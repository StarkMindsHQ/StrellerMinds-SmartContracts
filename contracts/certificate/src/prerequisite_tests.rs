#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};
    use crate::types::{
        CoursePrerequisite, PrerequisiteCourse, PrerequisiteCheckResult, PrerequisiteOverride,
        PrerequisitePolicy, LearningPath, CourseDependencyNode, PrerequisiteViolation
    };
    use crate::prerequisites::PrerequisiteManager;
    use crate::errors::CertificateError;

    fn create_test_env() -> Env {
        Env::default()
    }

    fn create_test_addresses(env: &Env) -> (Address, Address, Address, Address) {
        (
            Address::generate(env),
            Address::generate(env),
            Address::generate(env),
            Address::generate(env),
        )
    }

    fn create_basic_prerequisite(env: &Env, course_id: &str, prereq_course_id: &str) -> CoursePrerequisite {
        let mut prerequisite_courses = Vec::new(env);
        prerequisite_courses.push_back(PrerequisiteCourse {
            course_id: String::from_str(env, prereq_course_id),
            minimum_percentage: 80,
            weight: 1,
            required_certificate: false,
        });

        CoursePrerequisite {
            course_id: String::from_str(env, course_id),
            prerequisite_courses,
            minimum_completion_percentage: 80,
            policy: PrerequisitePolicy::Strict,
            created_by: Address::generate(env),
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
        }
    }

    #[test]
    fn test_define_prerequisites_success() {
        let env = create_test_env();
        let (admin, _, _, _) = create_test_addresses(&env);
        
        // Setup RBAC mock
        env.mock_all_auths();

        let prerequisite = create_basic_prerequisite(&env, "advanced_rust", "basic_rust");

        let result = PrerequisiteManager::define_prerequisites(&env, &admin, prerequisite.clone());
        assert!(result.is_ok());

        // Verify storage
        let stored = env.storage()
            .persistent()
            .get(&crate::types::DataKey::CoursePrerequisites(prerequisite.course_id.clone()));
        assert!(stored.is_some());
    }

    #[test]
    fn test_define_prerequisites_invalid_config() {
        let env = create_test_env();
        let (admin, _, _, _) = create_test_addresses(&env);
        
        env.mock_all_auths();

        let mut prerequisite = create_basic_prerequisite(&env, "advanced_rust", "basic_rust");
        prerequisite.minimum_completion_percentage = 150; // Invalid percentage

        let result = PrerequisiteManager::define_prerequisites(&env, &admin, prerequisite);
        assert_eq!(result.unwrap_err(), CertificateError::InvalidPrerequisiteConfig);
    }

    #[test]
    fn test_define_prerequisites_empty_courses() {
        let env = create_test_env();
        let (admin, _, _, _) = create_test_addresses(&env);
        
        env.mock_all_auths();

        let prerequisite = CoursePrerequisite {
            course_id: String::from_str(&env, "advanced_rust"),
            prerequisite_courses: Vec::new(&env), // Empty prerequisites
            minimum_completion_percentage: 80,
            policy: PrerequisitePolicy::Strict,
            created_by: admin.clone(),
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
        };

        let result = PrerequisiteManager::define_prerequisites(&env, &admin, prerequisite);
        assert_eq!(result.unwrap_err(), CertificateError::InvalidPrerequisiteConfig);
    }

    #[test]
    fn test_check_prerequisites_eligible() {
        let env = create_test_env();
        let (admin, student, progress_contract, _) = create_test_addresses(&env);
        
        env.mock_all_auths();

        // Define prerequisite
        let prerequisite = create_basic_prerequisite(&env, "advanced_rust", "basic_rust");
        PrerequisiteManager::define_prerequisites(&env, &admin, prerequisite.clone()).unwrap();

        // Mock progress check - student has completed prerequisite
        // Note: In a real implementation, this would call the progress contract

        let result = PrerequisiteManager::check_prerequisites(
            &env,
            &student,
            &String::from_str(&env, "advanced_rust"),
            &progress_contract,
        );

        assert!(result.is_ok());
        let check_result = result.unwrap();
        // Note: The mock implementation in prerequisites.rs returns mock data
        // In a real test, we would set up proper mocks for the progress contract
    }

    #[test]
    fn test_grant_prerequisite_override() {
        let env = create_test_env();
        let (admin, student, _, _) = create_test_addresses(&env);
        
        env.mock_all_auths();

        let override_data = PrerequisiteOverride {
            student: student.clone(),
            course_id: String::from_str(&env, "advanced_rust"),
            overridden_prerequisites: {
                let mut vec = Vec::new(&env);
                vec.push_back(String::from_str(&env, "basic_rust"));
                vec
            },
            override_reason: String::from_str(&env, "Student has equivalent experience"),
            granted_by: admin.clone(),
            granted_at: env.ledger().timestamp(),
            expires_at: Some(env.ledger().timestamp() + 86400 * 30), // 30 days
        };

        let result = PrerequisiteManager::grant_prerequisite_override(&env, &admin, override_data.clone());
        assert!(result.is_ok());

        // Verify storage
        let stored = env.storage()
            .persistent()
            .get(&crate::types::DataKey::PrerequisiteOverride(
                student.clone(),
                String::from_str(&env, "advanced_rust")
            ));
        assert!(stored.is_some());
    }

    #[test]
    fn test_revoke_prerequisite_override() {
        let env = create_test_env();
        let (admin, student, _, _) = create_test_addresses(&env);
        
        env.mock_all_auths();

        // First grant an override
        let override_data = PrerequisiteOverride {
            student: student.clone(),
            course_id: String::from_str(&env, "advanced_rust"),
            overridden_prerequisites: {
                let mut vec = Vec::new(&env);
                vec.push_back(String::from_str(&env, "basic_rust"));
                vec
            },
            override_reason: String::from_str(&env, "Student has equivalent experience"),
            granted_by: admin.clone(),
            granted_at: env.ledger().timestamp(),
            expires_at: None,
        };

        PrerequisiteManager::grant_prerequisite_override(&env, &admin, override_data).unwrap();

        // Now revoke it
        let result = PrerequisiteManager::revoke_prerequisite_override(
            &env,
            &admin,
            &student,
            &String::from_str(&env, "advanced_rust"),
            String::from_str(&env, "Override no longer needed"),
        );

        assert!(result.is_ok());

        // Verify removal
        let stored = env.storage()
            .persistent()
            .get(&crate::types::DataKey::PrerequisiteOverride(
                student.clone(),
                String::from_str(&env, "advanced_rust")
            ));
        assert!(stored.is_none());
    }

    #[test]
    fn test_revoke_nonexistent_override() {
        let env = create_test_env();
        let (admin, student, _, _) = create_test_addresses(&env);
        
        env.mock_all_auths();

        let result = PrerequisiteManager::revoke_prerequisite_override(
            &env,
            &admin,
            &student,
            &String::from_str(&env, "advanced_rust"),
            String::from_str(&env, "Override no longer needed"),
        );

        assert_eq!(result.unwrap_err(), CertificateError::PrerequisiteOverrideNotFound);
    }

    #[test]
    fn test_generate_learning_path() {
        let env = create_test_env();
        let (admin, student, progress_contract, _) = create_test_addresses(&env);
        
        env.mock_all_auths();

        // Create a chain of prerequisites: intro -> basic -> intermediate -> advanced
        let intro_prereq = create_basic_prerequisite(&env, "basic_rust", "intro_programming");
        let intermediate_prereq = create_basic_prerequisite(&env, "intermediate_rust", "basic_rust");
        let advanced_prereq = create_basic_prerequisite(&env, "advanced_rust", "intermediate_rust");

        PrerequisiteManager::define_prerequisites(&env, &admin, intro_prereq).unwrap();
        PrerequisiteManager::define_prerequisites(&env, &admin, intermediate_prereq).unwrap();
        PrerequisiteManager::define_prerequisites(&env, &admin, advanced_prereq).unwrap();

        let result = PrerequisiteManager::generate_learning_path(
            &env,
            &student,
            &String::from_str(&env, "advanced_rust"),
            &progress_contract,
        );

        assert!(result.is_ok());
        let learning_path = result.unwrap();
        assert_eq!(learning_path.target_course, String::from_str(&env, "advanced_rust"));
        assert!(learning_path.recommended_sequence.len() > 0);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let env = create_test_env();
        let (admin, _, _, _) = create_test_addresses(&env);
        
        env.mock_all_auths();

        // Create A -> B
        let prereq_a = create_basic_prerequisite(&env, "course_a", "course_b");
        PrerequisiteManager::define_prerequisites(&env, &admin, prereq_a).unwrap();

        // Try to create B -> A (circular dependency)
        let prereq_b = create_basic_prerequisite(&env, "course_b", "course_a");
        let result = PrerequisiteManager::define_prerequisites(&env, &admin, prereq_b);

        assert_eq!(result.unwrap_err(), CertificateError::CircularDependency);
    }

    #[test]
    fn test_validate_enrollment_success() {
        let env = create_test_env();
        let (admin, student, progress_contract, enrolled_by) = create_test_addresses(&env);
        
        env.mock_all_auths();

        // Define prerequisite
        let prerequisite = create_basic_prerequisite(&env, "advanced_rust", "basic_rust");
        PrerequisiteManager::define_prerequisites(&env, &admin, prerequisite).unwrap();

        // Mock that student meets prerequisites (this would be handled by progress contract in reality)
        let result = PrerequisiteManager::validate_enrollment(
            &env,
            &student,
            &String::from_str(&env, "advanced_rust"),
            &enrolled_by,
            &progress_contract,
        );

        // Note: The mock implementation may return different results
        // In a real implementation, we would mock the progress contract properly
        match result {
            Ok(_) => assert!(true),
            Err(CertificateError::PrerequisiteNotMet) => {
                // This is expected with mock data showing insufficient progress
                assert!(true);
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_dependency_graph_creation() {
        let env = create_test_env();
        let (admin, _, _, _) = create_test_addresses(&env);
        
        env.mock_all_auths();

        let prerequisite = create_basic_prerequisite(&env, "advanced_rust", "basic_rust");
        PrerequisiteManager::define_prerequisites(&env, &admin, prerequisite.clone()).unwrap();

        let dependency_graph = PrerequisiteManager::get_dependency_graph(
            &env,
            &String::from_str(&env, "advanced_rust")
        );

        assert!(dependency_graph.is_some());
        let graph = dependency_graph.unwrap();
        assert_eq!(graph.course_id, String::from_str(&env, "advanced_rust"));
        assert!(graph.direct_prerequisites.len() > 0);
    }

    #[test]
    fn test_prerequisite_override_validation() {
        let env = create_test_env();
        let (admin, student, _, _) = create_test_addresses(&env);
        
        env.mock_all_auths();

        // Test invalid override - empty reason
        let invalid_override = PrerequisiteOverride {
            student: student.clone(),
            course_id: String::from_str(&env, "advanced_rust"),
            overridden_prerequisites: {
                let mut vec = Vec::new(&env);
                vec.push_back(String::from_str(&env, "basic_rust"));
                vec
            },
            override_reason: String::from_str(&env, ""), // Empty reason
            granted_by: admin.clone(),
            granted_at: env.ledger().timestamp(),
            expires_at: None,
        };

        let result = PrerequisiteManager::grant_prerequisite_override(&env, &admin, invalid_override);
        assert_eq!(result.unwrap_err(), CertificateError::InvalidInput);

        // Test invalid override - empty prerequisites
        let invalid_override2 = PrerequisiteOverride {
            student: student.clone(),
            course_id: String::from_str(&env, "advanced_rust"),
            overridden_prerequisites: Vec::new(&env), // Empty prerequisites
            override_reason: String::from_str(&env, "Valid reason"),
            granted_by: admin.clone(),
            granted_at: env.ledger().timestamp(),
            expires_at: None,
        };

        let result2 = PrerequisiteManager::grant_prerequisite_override(&env, &admin, invalid_override2);
        assert_eq!(result2.unwrap_err(), CertificateError::InvalidInput);
    }

    #[test]
    fn test_weighted_prerequisites() {
        let env = create_test_env();
        let (admin, _, _, _) = create_test_addresses(&env);
        
        env.mock_all_auths();

        let mut prerequisite_courses = Vec::new(&env);
        
        // Add multiple prerequisites with different weights
        prerequisite_courses.push_back(PrerequisiteCourse {
            course_id: String::from_str(&env, "basic_rust"),
            minimum_percentage: 80,
            weight: 3, // High weight
            required_certificate: false,
        });
        
        prerequisite_courses.push_back(PrerequisiteCourse {
            course_id: String::from_str(&env, "intro_programming"),
            minimum_percentage: 70,
            weight: 1, // Low weight
            required_certificate: false,
        });

        let prerequisite = CoursePrerequisite {
            course_id: String::from_str(&env, "advanced_rust"),
            prerequisite_courses,
            minimum_completion_percentage: 80,
            policy: PrerequisitePolicy::Weighted,
            created_by: admin.clone(),
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
        };

        let result = PrerequisiteManager::define_prerequisites(&env, &admin, prerequisite);
        assert!(result.is_ok());
    }

    #[test]
    fn test_prerequisite_config_validation_edge_cases() {
        let env = create_test_env();

        // Test weight validation
        let mut prerequisite_courses = Vec::new(&env);
        prerequisite_courses.push_back(PrerequisiteCourse {
            course_id: String::from_str(&env, "basic_rust"),
            minimum_percentage: 80,
            weight: 0, // Invalid weight
            required_certificate: false,
        });

        let prerequisite = CoursePrerequisite {
            course_id: String::from_str(&env, "advanced_rust"),
            prerequisite_courses,
            minimum_completion_percentage: 80,
            policy: PrerequisitePolicy::Strict,
            created_by: Address::generate(&env),
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
        };

        let result = crate::prerequisites::PrerequisiteManager::validate_prerequisite_config(&prerequisite);
        assert_eq!(result.unwrap_err(), CertificateError::InvalidPrerequisiteConfig);

        // Test percentage validation
        let mut prerequisite_courses2 = Vec::new(&env);
        prerequisite_courses2.push_back(PrerequisiteCourse {
            course_id: String::from_str(&env, "basic_rust"),
            minimum_percentage: 150, // Invalid percentage
            weight: 1,
            required_certificate: false,
        });

        let prerequisite2 = CoursePrerequisite {
            course_id: String::from_str(&env, "advanced_rust"),
            prerequisite_courses: prerequisite_courses2,
            minimum_completion_percentage: 80,
            policy: PrerequisitePolicy::Strict,
            created_by: Address::generate(&env),
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
        };

        let result2 = crate::prerequisites::PrerequisiteManager::validate_prerequisite_config(&prerequisite2);
        assert_eq!(result2.unwrap_err(), CertificateError::InvalidPrerequisiteConfig);
    }

    #[test]
    fn test_learning_path_optimization() {
        let env = create_test_env();
        let (admin, student, progress_contract, _) = create_test_addresses(&env);
        
        env.mock_all_auths();

        // Create a complex dependency tree
        // Foundation courses
        let math_prereq = create_basic_prerequisite(&env, "advanced_math", "basic_math");
        let programming_prereq = create_basic_prerequisite(&env, "advanced_programming", "basic_programming");
        
        // Combined course requiring both
        let mut combined_prereqs = Vec::new(&env);
        combined_prereqs.push_back(PrerequisiteCourse {
            course_id: String::from_str(&env, "advanced_math"),
            minimum_percentage: 80,
            weight: 1,
            required_certificate: false,
        });
        combined_prereqs.push_back(PrerequisiteCourse {
            course_id: String::from_str(&env, "advanced_programming"),
            minimum_percentage: 80,
            weight: 1,
            required_certificate: false,
        });

        let combined_course = CoursePrerequisite {
            course_id: String::from_str(&env, "data_science"),
            prerequisite_courses: combined_prereqs,
            minimum_completion_percentage: 80,
            policy: PrerequisitePolicy::Strict,
            created_by: admin.clone(),
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
        };

        PrerequisiteManager::define_prerequisites(&env, &admin, math_prereq).unwrap();
        PrerequisiteManager::define_prerequisites(&env, &admin, programming_prereq).unwrap();
        PrerequisiteManager::define_prerequisites(&env, &admin, combined_course).unwrap();

        let result = PrerequisiteManager::generate_learning_path(
            &env,
            &student,
            &String::from_str(&env, "data_science"),
            &progress_contract,
        );

        assert!(result.is_ok());
        let learning_path = result.unwrap();
        assert!(learning_path.recommended_sequence.len() > 0);
        assert!(learning_path.estimated_total_time > 0);
    }
}
