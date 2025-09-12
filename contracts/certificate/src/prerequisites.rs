use soroban_sdk::{Address, Env, String, Vec};
use crate::errors::CertificateError;
use crate::types::{
    CoursePrerequisite, PrerequisiteCourse, PrerequisiteCheckResult, MissingPrerequisite,
    CompletedPrerequisite, PrerequisiteOverride, PrerequisiteViolation, CourseDependencyNode,
    PrerequisitePolicy, LearningPath, DataKey
};
use crate::storage::CertificateStorage;
use crate::events::CertificateEvents;
use shared::access_control::AccessControl;
use shared::roles::Permission;

/// External progress contract interface for checking completion
pub trait ProgressContract {
    fn get_completion_percentage(env: Env, user: Address, course_id: String) -> Result<u32, u32>;
    fn get_progress(env: Env, user: Address, course_id: String) -> Result<Vec<bool>, u32>;
}

/// Prerequisites management system
pub struct PrerequisiteManager;

impl PrerequisiteManager {
    /// Define prerequisites for a course
    pub fn define_prerequisites(
        env: &Env,
        admin: &Address,
        course_prerequisite: CoursePrerequisite,
    ) -> Result<(), CertificateError> {
        // Validate admin permissions
        AccessControl::require_permission(env, admin, &Permission::UpdateCertificateMetadata)
            .map_err(|_| CertificateError::Unauthorized)?;

        // Validate prerequisite configuration
        Self::validate_prerequisite_config(&course_prerequisite)?;

        // Check for circular dependencies
        Self::check_circular_dependencies(env, &course_prerequisite)?;

        // Store prerequisite definition
        env.storage()
            .persistent()
            .set(&DataKey::CoursePrerequisites(course_prerequisite.course_id.clone()), &course_prerequisite);

        // Update dependency graph
        Self::update_dependency_graph(env, &course_prerequisite)?;

        // Emit event
        CertificateEvents::emit_prerequisite_defined(
            env,
            &course_prerequisite.course_id,
            admin,
            course_prerequisite.prerequisite_courses.len() as u32,
        );

        Ok(())
    }

    /// Check if student meets prerequisites for a course
    pub fn check_prerequisites(
        env: &Env,
        student: &Address,
        course_id: &String,
        progress_contract: &Address,
    ) -> Result<PrerequisiteCheckResult, CertificateError> {
        // Get prerequisite definition
        let prerequisites = Self::get_prerequisites(env, course_id)?;

        // Check for active override
        let override_applied = Self::get_active_override(env, student, course_id);

        // If override exists and is valid, student is eligible
        if let Some(ref override_data) = override_applied {
            if Self::is_override_valid(env, override_data) {
                return Ok(PrerequisiteCheckResult {
                    student: student.clone(),
                    course_id: course_id.clone(),
                    eligible: true,
                    missing_prerequisites: Vec::new(env),
                    completed_prerequisites: Vec::new(env),
                    check_timestamp: env.ledger().timestamp(),
                    override_applied: override_applied,
                });
            }
        }

        let mut missing_prerequisites = Vec::new(env);
        let mut completed_prerequisites = Vec::new(env);
        let mut eligible = true;

        // Check each prerequisite course
        for prereq_course in prerequisites.prerequisite_courses.iter() {
            let completion_result = Self::check_course_completion(
                env,
                student,
                &prereq_course.course_id,
                &prereq_course,
                progress_contract,
            );

            match completion_result {
                Ok(completed_prereq) => {
                    completed_prerequisites.push_back(completed_prereq);
                }
                Err(missing_prereq) => {
                    missing_prerequisites.push_back(missing_prereq);
                    eligible = false;
                }
            }
        }

        // Apply enforcement policy
        eligible = Self::apply_enforcement_policy(
            env,
            &prerequisites,
            eligible,
            &missing_prerequisites,
            &completed_prerequisites,
        );

        let result = PrerequisiteCheckResult {
            student: student.clone(),
            course_id: course_id.clone(),
            eligible,
            missing_prerequisites,
            completed_prerequisites,
            check_timestamp: env.ledger().timestamp(),
            override_applied,
        };

        // Cache the result for performance
        Self::cache_check_result(env, &result);

        Ok(result)
    }

    /// Grant prerequisite override for a student
    pub fn grant_prerequisite_override(
        env: &Env,
        admin: &Address,
        override_data: PrerequisiteOverride,
    ) -> Result<(), CertificateError> {
        // Validate admin permissions
        AccessControl::require_permission(env, admin, &Permission::UpdateCertificateMetadata)
            .map_err(|_| CertificateError::Unauthorized)?;

        // Validate override data
        Self::validate_override(&override_data)?;

        // Store override
        env.storage()
            .persistent()
            .set(
                &DataKey::PrerequisiteOverride(override_data.student.clone(), override_data.course_id.clone()),
                &override_data,
            );

        // Log the override
        Self::log_prerequisite_action(
            env,
            &override_data.student,
            &override_data.course_id,
            "Override granted",
            admin,
        );

        // Emit event
        CertificateEvents::emit_prerequisite_override_granted(
            env,
            &override_data.student,
            &override_data.course_id,
            admin,
            &override_data.override_reason,
        );

        Ok(())
    }

    /// Revoke prerequisite override
    pub fn revoke_prerequisite_override(
        env: &Env,
        admin: &Address,
        student: &Address,
        course_id: &String,
        reason: String,
    ) -> Result<(), CertificateError> {
        // Validate admin permissions
        AccessControl::require_permission(env, admin, &Permission::UpdateCertificateMetadata)
            .map_err(|_| CertificateError::Unauthorized)?;

        // Check if override exists
        let key = DataKey::PrerequisiteOverride(student.clone(), course_id.clone());
        if !env.storage().persistent().has(&key) {
            return Err(CertificateError::PrerequisiteOverrideNotFound);
        }

        // Remove override
        env.storage().persistent().remove(&key);

        // Log the revocation
        Self::log_prerequisite_action(env, student, course_id, &reason, admin);

        // Emit event
        CertificateEvents::emit_prerequisite_override_revoked(env, student, course_id, admin, &reason);

        Ok(())
    }

    /// Generate learning path for a student to reach target course
    pub fn generate_learning_path(
        env: &Env,
        student: &Address,
        target_course: &String,
        progress_contract: &Address,
    ) -> Result<LearningPath, CertificateError> {
        // Get all prerequisites for target course (including transitive)
        let all_prerequisites = Self::get_all_prerequisites(env, target_course)?;

        // Check current progress for each prerequisite
        let mut missing_courses = Vec::new(env);
        for course_id in all_prerequisites.iter() {
            let check_result = Self::check_prerequisites(env, student, &course_id, progress_contract)?;
            if !check_result.eligible {
                missing_courses.push_back(course_id);
            }
        }

        // Generate optimal sequence
        let recommended_sequence = Self::optimize_learning_sequence(env, &missing_courses)?;

        // Estimate total time
        let estimated_total_time = Self::estimate_learning_time(env, &recommended_sequence);

        let learning_path = LearningPath {
            student: student.clone(),
            target_course: target_course.clone(),
            recommended_sequence,
            estimated_total_time,
            current_position: 0,
            generated_at: env.ledger().timestamp(),
            last_updated: env.ledger().timestamp(),
        };

        // Store learning path
        env.storage()
            .persistent()
            .set(&DataKey::LearningPath(student.clone(), target_course.clone()), &learning_path);

        Ok(learning_path)
    }

    /// Get course dependency graph
    pub fn get_dependency_graph(env: &Env, course_id: &String) -> Option<CourseDependencyNode> {
        env.storage()
            .persistent()
            .get(&DataKey::CourseDependencies(course_id.clone()))
    }

    /// Validate course enrollment against prerequisites
    pub fn validate_enrollment(
        env: &Env,
        student: &Address,
        course_id: &String,
        enrolled_by: &Address,
        progress_contract: &Address,
    ) -> Result<(), CertificateError> {
        let check_result = Self::check_prerequisites(env, student, course_id, progress_contract)?;

        if !check_result.eligible {
            // Log violation
            let violation = PrerequisiteViolation {
                student: student.clone(),
                attempted_course: course_id.clone(),
                missing_prerequisites: check_result.missing_prerequisites
                    .iter()
                    .map(|mp| mp.course_id.clone())
                    .collect(),
                violation_timestamp: env.ledger().timestamp(),
                attempted_by: enrolled_by.clone(),
            };

            Self::log_violation(env, &violation);

            // Emit violation event
            CertificateEvents::emit_prerequisite_violation(
                env,
                student,
                course_id,
                enrolled_by,
                check_result.missing_prerequisites.len() as u32,
            );

            return Err(CertificateError::PrerequisiteNotMet);
        }

        Ok(())
    }

    // Private helper methods

    fn validate_prerequisite_config(config: &CoursePrerequisite) -> Result<(), CertificateError> {
        if config.prerequisite_courses.is_empty() {
            return Err(CertificateError::InvalidPrerequisiteConfig);
        }

        if config.minimum_completion_percentage > 100 {
            return Err(CertificateError::InvalidPrerequisiteConfig);
        }

        for prereq in config.prerequisite_courses.iter() {
            if prereq.minimum_percentage > 100 {
                return Err(CertificateError::InvalidPrerequisiteConfig);
            }
            if prereq.weight == 0 || prereq.weight > 10 {
                return Err(CertificateError::InvalidPrerequisiteConfig);
            }
        }

        Ok(())
    }

    fn check_circular_dependencies(
        env: &Env,
        new_prerequisite: &CoursePrerequisite,
    ) -> Result<(), CertificateError> {
        // Simple circular dependency check - more sophisticated algorithm could be implemented
        for prereq_course in new_prerequisite.prerequisite_courses.iter() {
            if let Some(existing_prereqs) = Self::get_prerequisites(env, &prereq_course.course_id).ok() {
                for existing_prereq in existing_prereqs.prerequisite_courses.iter() {
                    if existing_prereq.course_id == new_prerequisite.course_id {
                        return Err(CertificateError::CircularDependency);
                    }
                }
            }
        }
        Ok(())
    }

    fn check_course_completion(
        env: &Env,
        student: &Address,
        course_id: &String,
        prereq_course: &PrerequisiteCourse,
        progress_contract: &Address,
    ) -> Result<CompletedPrerequisite, MissingPrerequisite> {
        // Mock progress check - in real implementation, this would call the progress contract
        let completion_percentage = 75u32; // Mock data
        let has_certificate = false; // Mock data

        let required_percentage = if prereq_course.minimum_percentage > 0 {
            prereq_course.minimum_percentage
        } else {
            80u32 // Default minimum
        };

        if completion_percentage >= required_percentage && 
           (!prereq_course.required_certificate || has_certificate) {
            Ok(CompletedPrerequisite {
                course_id: course_id.clone(),
                completion_percentage,
                has_certificate,
                completion_date: Some(env.ledger().timestamp() - 86400), // Mock completion date
                certificate_id: None,
            })
        } else {
            Err(MissingPrerequisite {
                course_id: course_id.clone(),
                current_percentage: completion_percentage,
                required_percentage,
                has_certificate,
                requires_certificate: prereq_course.required_certificate,
                estimated_completion_time: Some(3600 * 10), // Mock 10 hours
            })
        }
    }

    fn apply_enforcement_policy(
        env: &Env,
        prerequisites: &CoursePrerequisite,
        base_eligible: bool,
        missing_prerequisites: &Vec<MissingPrerequisite>,
        completed_prerequisites: &Vec<CompletedPrerequisite>,
    ) -> bool {
        // For now, use strict policy - could be made configurable
        base_eligible
    }

    fn get_prerequisites(env: &Env, course_id: &String) -> Result<CoursePrerequisite, CertificateError> {
        env.storage()
            .persistent()
            .get(&DataKey::CoursePrerequisites(course_id.clone()))
            .ok_or(CertificateError::PrerequisiteNotFound)
    }

    fn get_active_override(
        env: &Env,
        student: &Address,
        course_id: &String,
    ) -> Option<PrerequisiteOverride> {
        env.storage()
            .persistent()
            .get(&DataKey::PrerequisiteOverride(student.clone(), course_id.clone()))
    }

    fn is_override_valid(env: &Env, override_data: &PrerequisiteOverride) -> bool {
        if let Some(expires_at) = override_data.expires_at {
            env.ledger().timestamp() <= expires_at
        } else {
            true // No expiration
        }
    }

    fn validate_override(override_data: &PrerequisiteOverride) -> Result<(), CertificateError> {
        if override_data.override_reason.is_empty() {
            return Err(CertificateError::InvalidInput);
        }
        if override_data.overridden_prerequisites.is_empty() {
            return Err(CertificateError::InvalidInput);
        }
        Ok(())
    }

    fn update_dependency_graph(
        env: &Env,
        prerequisite: &CoursePrerequisite,
    ) -> Result<(), CertificateError> {
        let direct_prerequisites: Vec<String> = prerequisite.prerequisite_courses
            .iter()
            .map(|pc| pc.course_id.clone())
            .collect();

        let dependency_node = CourseDependencyNode {
            course_id: prerequisite.course_id.clone(),
            level: Self::calculate_dependency_level(env, &direct_prerequisites),
            direct_prerequisites,
            all_prerequisites: Self::get_all_prerequisites(env, &prerequisite.course_id)
                .unwrap_or_else(|_| Vec::new(env)),
            dependent_courses: Vec::new(env), // Would be populated by reverse lookup
        };

        env.storage()
            .persistent()
            .set(&DataKey::CourseDependencies(prerequisite.course_id.clone()), &dependency_node);

        Ok(())
    }

    fn calculate_dependency_level(env: &Env, prerequisites: &Vec<String>) -> u32 {
        let mut max_level = 0u32;
        for prereq_id in prerequisites.iter() {
            if let Some(prereq_node) = Self::get_dependency_graph(env, &prereq_id) {
                max_level = max_level.max(prereq_node.level);
            }
        }
        max_level + 1
    }

    fn get_all_prerequisites(env: &Env, course_id: &String) -> Result<Vec<String>, CertificateError> {
        let mut all_prerequisites = Vec::new(env);
        let mut to_process = Vec::new(env);
        to_process.push_back(course_id.clone());

        while !to_process.is_empty() {
            let current_course = to_process.pop_front().unwrap();
            if let Ok(prereqs) = Self::get_prerequisites(env, &current_course) {
                for prereq_course in prereqs.prerequisite_courses.iter() {
                    if !all_prerequisites.contains(&prereq_course.course_id) {
                        all_prerequisites.push_back(prereq_course.course_id.clone());
                        to_process.push_back(prereq_course.course_id.clone());
                    }
                }
            }
        }

        Ok(all_prerequisites)
    }

    fn optimize_learning_sequence(env: &Env, courses: &Vec<String>) -> Result<Vec<String>, CertificateError> {
        // Simple topological sort - could be enhanced with more sophisticated algorithms
        let mut sequence = Vec::new(env);
        let mut remaining_courses = courses.clone();

        while !remaining_courses.is_empty() {
            let mut added_any = false;
            let mut i = 0;
            
            while i < remaining_courses.len() {
                let course = remaining_courses.get(i as u32).unwrap();
                let can_add = if let Ok(prereqs) = Self::get_prerequisites(env, &course) {
                    prereqs.prerequisite_courses.iter().all(|pc| sequence.contains(&pc.course_id))
                } else {
                    true // No prerequisites
                };

                if can_add {
                    sequence.push_back(course.clone());
                    remaining_courses.remove(i as u32);
                    added_any = true;
                } else {
                    i += 1;
                }
            }

            if !added_any {
                return Err(CertificateError::CircularDependency);
            }
        }

        Ok(sequence)
    }

    fn estimate_learning_time(env: &Env, courses: &Vec<String>) -> u64 {
        // Mock implementation - would use actual course duration data
        courses.len() as u64 * 3600 * 40 // 40 hours per course
    }

    fn cache_check_result(env: &Env, result: &PrerequisiteCheckResult) {
        env.storage()
            .temporary()
            .set(&DataKey::PrerequisiteCheckCache(result.student.clone(), result.course_id.clone()), result);
    }

    fn log_prerequisite_action(
        env: &Env,
        student: &Address,
        course_id: &String,
        action: &str,
        admin: &Address,
    ) {
        // Log to events for audit trail
        env.events().publish(
            ("prerequisite_action", student, course_id),
            (action, admin, env.ledger().timestamp()),
        );
    }

    fn log_violation(env: &Env, violation: &PrerequisiteViolation) {
        let mut violations = env.storage()
            .persistent()
            .get(&DataKey::PrerequisiteViolations(violation.student.clone()))
            .unwrap_or_else(|| Vec::new(env));
        
        violations.push_back(violation.clone());
        
        env.storage()
            .persistent()
            .set(&DataKey::PrerequisiteViolations(violation.student.clone()), &violations);
    }
}
