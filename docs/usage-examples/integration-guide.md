# StrellerMinds Smart Contracts Integration Guide

## Overview

This comprehensive guide provides detailed examples and best practices for integrating with the StrellerMinds smart contracts. It covers common integration patterns, code examples, and practical implementations for educational platforms.

## Prerequisites

### Development Environment Setup

```bash
# Install Rust and Soroban CLI
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
cargo install soroban-cli

# Clone the repository
git clone https://github.com/olaleyeolajide81-sketch/StrellerMinds-SmartContracts.git
cd StrellerMinds-SmartContracts

# Build contracts
cargo build --target wasm32-unknown-unknown --release
```

### Required Dependencies

```toml
# Cargo.toml
[dependencies]
soroban-sdk = "20.0.0"
stellarminds-shared = { path = "../contracts/shared" }
stellarminds-analytics = { path = "../contracts/analytics" }
stellarminds-token = { path = "../contracts/token" }
```

## Basic Integration Patterns

### 1. Contract Client Setup

```rust
use soroban_sdk::{Address, Env, Symbol};
use stellarminds_shared::AccessControl;
use stellarminds_analytics::Analytics;
use stellarminds_token::Token;

pub struct StrellerMindsClient {
    env: Env,
    shared_address: Address,
    analytics_address: Address,
    token_address: Address,
}

impl StrellerMindsClient {
    pub fn new(
        env: Env,
        shared_address: Address,
        analytics_address: Address,
        token_address: Address,
    ) -> Self {
        Self {
            env,
            shared_address,
            analytics_address,
            token_address,
        }
    }
    
    // Initialize all contracts
    pub fn initialize_platform(&self, admin: Address) -> Result<(), soroban_sdk::Error> {
        // Initialize shared contract
        let shared_client = AccessControlClient::new(&self.env, &self.shared_address);
        shared_client.initialize(&admin)?;
        
        // Initialize analytics contract
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        analytics_client.initialize(&admin)?;
        
        // Initialize token contract
        let token_client = TokenClient::new(&self.env, &self.token_address);
        token_client.initialize(&admin)?;
        
        Ok(())
    }
}
```

### 2. User Management Integration

```rust
impl StrellerMindsClient {
    /// Register a new student with appropriate roles and initial setup
    pub fn register_student(
        &self,
        admin: Address,
        student: Address,
        student_info: StudentInfo,
    ) -> Result<(), soroban_sdk::Error> {
        // 1. Grant student role
        let shared_client = AccessControlClient::new(&self.env, &self.shared_address);
        shared_client.grant_role(&admin, Role::Student, &student)?;
        
        // 2. Initialize student analytics
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        analytics_client.initialize_student_profile(&student, &student_info)?;
        
        // 3. Award welcome tokens
        let token_client = TokenClient::new(&self.env, &self.token_address);
        token_client.mint(&admin, &student, 100)?; // Welcome bonus
        
        // 4. Create welcome achievement
        let welcome_achievement = Achievement {
            id: "welcome".to_string(),
            name: "Welcome to StrellerMinds".to_string(),
            description: "Joined the platform".to_string(),
            category: AchievementCategory::Special,
            achievement_type: AchievementType::Milestone,
            criteria: AchievementCriteria::CourseCompletion { course_count: 0 },
            reward_amount: 100,
            // ... other fields
        };
        token_client.create_achievement(&admin, welcome_achievement)?;
        token_client.award_achievement(&student, "welcome")?;
        
        Ok(())
    }
    
    /// Register an instructor with teaching permissions
    pub fn register_instructor(
        &self,
        admin: Address,
        instructor: Address,
        instructor_info: InstructorInfo,
    ) -> Result<(), soroban_sdk::Error> {
        // Grant instructor role
        let shared_client = AccessControlClient::new(&self.env, &self.shared_address);
        shared_client.grant_role(&admin, Role::Instructor, &instructor)?;
        
        // Initialize instructor analytics
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        analytics_client.initialize_instructor_profile(&instructor, &instructor_info)?;
        
        // Award instructor tokens
        let token_client = TokenClient::new(&self.env, &self.token_address);
        token_client.mint(&admin, &instructor, 500)?; // Instructor bonus
        
        Ok(())
    }
}
```

## Course Management Integration

### 1. Course Creation and Setup

```rust
impl StrellerMindsClient {
    /// Create a new course with comprehensive setup
    pub fn create_course(
        &self,
        instructor: Address,
        course_data: CourseCreationData,
    ) -> Result<CourseId, soroban_sdk::Error> {
        // 1. Verify instructor permissions
        let shared_client = AccessControlClient::new(&self.env, &self.shared_address);
        shared_client.require_role(&instructor, Role::Instructor)?;
        
        // 2. Create course in analytics system
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        let course_id = analytics_client.create_course(&instructor, &course_data)?;
        
        // 3. Set up course analytics tracking
        analytics_client.initialize_course_analytics(&course_id)?;
        
        // 4. Create course-specific achievements
        let token_client = TokenClient::new(&self.env, &self.token_address);
        self.create_course_achievements(&token_client, &instructor, &course_id)?;
        
        // 5. Set up course staking pool (if applicable)
        if course_data.enable_staking {
            let staking_pool = StakingPool {
                id: format!("course_{}", course_id),
                name: format!("{} Staking Pool", course_data.name),
                apy_percentage: 5.0,
                min_stake_amount: 100,
                lock_period_seconds: course_data.duration_seconds,
                max_total_stake: Some(10000),
            };
            token_client.create_staking_pool(&instructor, staking_pool)?;
        }
        
        Ok(course_id)
    }
    
    /// Create course-specific achievements
    fn create_course_achievements(
        &self,
        token_client: &TokenClient,
        instructor: &Address,
        course_id: &CourseId,
    ) -> Result<(), soroban_sdk::Error> {
        // Course completion achievement
        let completion_achievement = Achievement {
            id: format!("{}_complete", course_id),
            name: format!("{} Master", course_id),
            description: format!("Complete the {} course", course_id),
            category: AchievementCategory::Completion,
            achievement_type: AchievementType::Milestone,
            criteria: AchievementCriteria::FullCourseCompletion { 
                course_id: Symbol::new(&self.env, course_id) 
            },
            reward_amount: 200,
            // ... other fields
        };
        token_client.create_achievement(instructor, completion_achievement)?;
        
        // Perfect score achievement
        let perfect_achievement = Achievement {
            id: format!("{}_perfect", course_id),
            name: format!("{} Perfect Score", course_id),
            description: format!("Achieve perfect score in {}", course_id),
            category: AchievementCategory::Performance,
            achievement_type: AchievementType::Milestone,
            criteria: AchievementCriteria::PerfectScore { 
                assessment_id: Symbol::new(&self.env, format!("{}_final", course_id)) 
            },
            reward_amount: 150,
            // ... other fields
        };
        token_client.create_achievement(instructor, perfect_achievement)?;
        
        Ok(())
    }
}
```

### 2. Student Enrollment and Progress Tracking

```rust
impl StrellerMindsClient {
    /// Enroll a student in a course
    pub fn enroll_student(
        &self,
        student: Address,
        course_id: CourseId,
    ) -> Result<(), soroban_sdk::Error> {
        // 1. Verify student role
        let shared_client = AccessControlClient::new(&self.env, &self.shared_address);
        shared_client.require_role(&student, Role::Student)?;
        
        // 2. Enroll in analytics system
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        analytics_client.enroll_student(&student, &course_id)?;
        
        // 3. Initialize progress tracking
        analytics_client.initialize_progress_tracking(&student, &course_id)?;
        
        // 4. Award enrollment tokens (if configured)
        let token_client = TokenClient::new(&self.env, &self.token_address);
        if let Some(enrollment_reward) = self.get_enrollment_reward(&course_id) {
            token_client.mint(&self.get_admin_address(), &student, enrollment_reward)?;
        }
        
        Ok(())
    }
    
    /// Track learning session with comprehensive metrics
    pub fn track_learning_session(
        &self,
        student: Address,
        course_id: CourseId,
        session_data: LearningSessionData,
    ) -> Result<SessionId, soroban_sdk::Error> {
        // 1. Start session
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        let session_id = analytics_client.record_session(&student, &course_id)?;
        
        // 2. Track session events in real-time
        for event in session_data.events {
            analytics_client.record_session_event(&session_id, &event)?;
        }
        
        // 3. Update session metrics
        analytics_client.update_session_metrics(&session_id, &session_data.metrics)?;
        
        // 4. Complete session
        analytics_client.complete_session(&session_id, &session_data.final_metrics)?;
        
        // 5. Check for achievements
        self.check_session_achievements(&student, &course_id, &session_data)?;
        
        // 6. Update progress analytics
        analytics_client.update_progress_analytics(&student, &course_id)?;
        
        Ok(session_id)
    }
    
    /// Check and award achievements based on session performance
    fn check_session_achievements(
        &self,
        student: &Address,
        course_id: &CourseId,
        session_data: &LearningSessionData,
    ) -> Result<(), soroban_sdk::Error> {
        let token_client = TokenClient::new(&self.env, &self.token_address);
        
        // Check for streak achievements
        if session_data.learning_streak >= 7 {
            token_client.award_achievement(student, "week_streak")?;
        }
        
        if session_data.learning_streak >= 30 {
            token_client.award_achievement(student, "month_streak")?;
        }
        
        // Check for performance achievements
        if let Some(score) = session_data.final_metrics.score {
            if score >= 95.0 {
                token_client.award_achievement(student, "excellent_performance")?;
            }
        }
        
        // Check for speed achievements
        if session_data.completion_time < self.get_fast_completion_threshold(course_id) {
            token_client.award_achievement(student, "speed_learner")?;
        }
        
        Ok(())
    }
}
```

## Assessment and Grading Integration

### 1. Assessment Creation and Management

```rust
impl StrellerMindsClient {
    /// Create a new assessment with analytics integration
    pub fn create_assessment(
        &self,
        instructor: Address,
        course_id: CourseId,
        assessment_data: AssessmentCreationData,
    ) -> Result<AssessmentId, soroban_sdk::Error> {
        // 1. Verify instructor permissions
        let shared_client = AccessControlClient::new(&self.env, &self.shared_address);
        shared_client.require_role(&instructor, Role::Instructor)?;
        
        // 2. Create assessment in analytics system
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        let assessment_id = analytics_client.create_assessment(
            &instructor,
            &course_id,
            &assessment_data
        )?;
        
        // 3. Set up assessment analytics tracking
        analytics_client.initialize_assessment_analytics(&assessment_id)?;
        
        // 4. Create assessment-specific achievements
        if assessment_data.is_final_exam {
            self.create_assessment_achievements(&instructor, &course_id, &assessment_id)?;
        }
        
        Ok(assessment_id)
    }
    
    /// Submit and process student assessment
    pub fn submit_assessment(
        &self,
        student: Address,
        assessment_id: AssessmentId,
        submission_data: AssessmentSubmission,
    ) -> Result<AssessmentResult, soroban_sdk::Error> {
        // 1. Record submission
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        let result = analytics_client.submit_assessment(
            &student,
            &assessment_id,
            &submission_data
        )?;
        
        // 2. Update student progress
        let course_id = analytics_client.get_assessment_course(&assessment_id)?;
        analytics_client.update_progress_analytics(&student, &course_id)?;
        
        // 3. Award performance-based tokens
        let token_client = TokenClient::new(&self.env, &self.token_address);
        let token_reward = self.calculate_assessment_token_reward(&result);
        if token_reward > 0 {
            token_client.mint(&self.get_admin_address(), &student, token_reward)?;
        }
        
        // 4. Check for assessment achievements
        self.check_assessment_achievements(&student, &assessment_id, &result)?;
        
        // 5. Update course analytics
        analytics_client.update_course_analytics(&course_id)?;
        
        Ok(result)
    }
    
    /// Calculate token rewards based on assessment performance
    fn calculate_assessment_token_reward(&self, result: &AssessmentResult) -> u64 {
        let base_reward = 50u64;
        let score_bonus = (result.score * 0.5) as u64; // 0.5 tokens per point
        let streak_bonus = if result.learning_streak >= 7 { 20 } else { 0 };
        let perfect_bonus = if result.score >= 100.0 { 50 } else { 0 };
        
        base_reward + score_bonus + streak_bonus + perfect_bonus
    }
}
```

## Token Economy Integration

### 1. Token Rewards and Incentives

```rust
impl StrellerMindsClient {
    /// Implement comprehensive reward system
    pub fn process_learning_rewards(
        &self,
        student: Address,
        learning_activity: LearningActivity,
    ) -> Result<RewardResult, soroban_sdk::Error> {
        let token_client = TokenClient::new(&self.env, &self.token_address);
        let mut total_rewards = 0u64;
        let mut awarded_achievements = Vec::new(&self.env);
        
        match learning_activity {
            LearningActivity::CourseCompletion { course_id, score, time_spent } => {
                // Course completion reward
                let completion_reward = self.calculate_course_completion_reward(
                    &course_id, score, time_spent
                )?;
                token_client.mint(&self.get_admin_address(), &student, completion_reward)?;
                total_rewards += completion_reward;
                
                // Award completion achievement
                let achievement_id = format!("{}_complete", course_id);
                if token_client.award_achievement(&student, &achievement_id).is_ok() {
                    awarded_achievements.push_back(achievement_id);
                }
            }
            
            LearningActivity::StreakMaintenance { days } => {
                // Streak maintenance rewards
                if days >= 7 {
                    let streak_reward = 10 * days;
                    token_client.mint(&self.get_admin_address(), &student, streak_reward)?;
                    total_rewards += streak_reward;
                    
                    // Award streak achievement
                    if days == 7 {
                        token_client.award_achievement(&student, "week_streak")?;
                        awarded_achievements.push_back("week_streak".to_string());
                    } else if days == 30 {
                        token_client.award_achievement(&student, "month_streak")?;
                        awarded_achievements.push_back("month_streak".to_string());
                    }
                }
            }
            
            LearningActivity::PeerAssistance { helped_students, quality_score } => {
                // Peer assistance rewards
                let assistance_reward = (helped_students as u64) * (quality_score * 10) as u64;
                token_client.mint(&self.get_admin_address(), &student, assistance_reward)?;
                total_rewards += assistance_reward;
                
                // Award collaboration achievement
                if helped_students >= 10 {
                    token_client.award_achievement(&student, "helpful_peer")?;
                    awarded_achievements.push_back("helpful_peer".to_string());
                }
            }
        }
        
        Ok(RewardResult {
            total_tokens_rewarded: total_rewards,
            achievements_awarded: awarded_achievements,
            new_balance: token_client.balance(&student)?,
        })
    }
    
    /// Handle token staking for learning commitments
    pub fn stake_learning_commitment(
        &self,
        student: Address,
        course_id: CourseId,
        stake_amount: u64,
        commitment_period: u64, // seconds
    ) -> Result<StakeId, soroban_sdk::Error> {
        let token_client = TokenClient::new(&self.env, &self.token_address);
        
        // 1. Create learning commitment staking pool
        let pool_id = format!("learning_commitment_{}", course_id);
        let staking_pool = StakingPool {
            id: pool_id.clone(),
            name: format!("{} Learning Commitment", course_id),
            apy_percentage: 15.0, // Higher APY for learning commitments
            min_stake_amount: 100,
            lock_period_seconds: commitment_period,
            max_total_stake: Some(50000),
        };
        
        // Create pool if it doesn't exist
        if token_client.get_staking_pool(&pool_id).is_err() {
            token_client.create_staking_pool(&self.get_admin_address(), staking_pool)?;
        }
        
        // 2. Stake tokens
        token_client.stake_tokens(&student, &pool_id, stake_amount as i128)?;
        
        // 3. Create commitment tracking
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        let stake_id = analytics_client.create_learning_commitment(
            &student,
            &course_id,
            stake_amount,
            commitment_period
        )?;
        
        Ok(stake_id)
    }
}
```

### 2. Certificate and Upgrade System

```rust
impl StrellerMindsClient {
    /// Issue certificate with upgrade options
    pub fn issue_certificate(
        &self,
        student: Address,
        course_id: CourseId,
        performance_data: CoursePerformance,
    ) -> Result<CertificateId, soroban_sdk::Error> {
        // 1. Generate base certificate
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        let certificate_id = analytics_client.issue_certificate(
            &student,
            &course_id,
            &performance_data
        )?;
        
        // 2. Determine certificate level based on performance
        let certificate_level = self.determine_certificate_level(&performance_data);
        
        // 3. Set upgrade options based on level
        let upgrade_options = self.get_certificate_upgrade_options(&certificate_level);
        analytics_client.set_certificate_upgrades(
            &certificate_id,
            &upgrade_options
        )?;
        
        // 4. Award certificate achievement
        let token_client = TokenClient::new(&self.env, &self.token_address);
        let achievement_id = format!("certificate_{}", certificate_level);
        token_client.award_achievement(&student, &achievement_id)?;
        
        Ok(certificate_id)
    }
    
    /// Upgrade certificate using tokens
    pub fn upgrade_certificate(
        &self,
        student: Address,
        certificate_id: CertificateId,
        upgrade_type: CertificateUpgradeType,
    ) -> Result<(), soroban_sdk::Error> {
        let token_client = TokenClient::new(&self.env, &self.token_address);
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        
        // 1. Get upgrade cost
        let upgrade_cost = analytics_client.get_certificate_upgrade_cost(
            &certificate_id,
            &upgrade_type
        )?;
        
        // 2. Check student balance
        let student_balance = token_client.balance(&student)?;
        if student_balance < upgrade_cost {
            return Err(soroban_sdk::Error::from_contract_error(4001)); // Insufficient balance
        }
        
        // 3. Burn tokens for upgrade
        token_client.burn_for_upgrade(
            &student,
            upgrade_cost as i128,
            certificate_id.to_string(),
            upgrade_type.to_string()
        )?;
        
        // 4. Apply upgrade to certificate
        analytics_client.upgrade_certificate(&certificate_id, &upgrade_type)?;
        
        // 5. Award upgrade achievement
        token_client.award_achievement(&student, "certificate_upgraded")?;
        
        Ok(())
    }
}
```

## Analytics and Reporting Integration

### 1. Real-time Analytics Dashboard

```rust
impl StrellerMindsClient {
    /// Get comprehensive student dashboard data
    pub fn get_student_dashboard(
        &self,
        student: Address,
    ) -> Result<StudentDashboard, soroban_sdk::Error> {
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        let token_client = TokenClient::new(&self.env, &self.token_address);
        
        // 1. Get progress analytics for all active courses
        let active_courses = analytics_client.get_student_active_courses(&student)?;
        let mut course_progress = Vec::new(&self.env);
        
        for course_id in active_courses {
            let progress = analytics_client.get_progress_analytics(&student, &course_id)?;
            course_progress.push_back(CourseProgress {
                course_id,
                completion_percentage: progress.completion_percentage,
                average_score: progress.average_score,
                learning_streak: progress.learning_streak,
                predicted_completion: progress.predicted_completion,
            });
        }
        
        // 2. Get recent achievements
        let recent_achievements = analytics_client.get_recent_achievements(&student, 10)?;
        
        // 3. Get token balance and staking info
        let token_balance = token_client.balance(&student)?;
        let staking_positions = token_client.get_staking_positions(&student)?;
        
        // 4. Get leaderboard positions
        let leaderboard_positions = analytics_client.get_student_leaderboard_positions(&student)?;
        
        // 5. Get ML insights and recommendations
        let ml_insights = analytics_client.get_ml_insights(&student)?;
        let recommendations = analytics_client.get_recommendations(&student)?;
        
        Ok(StudentDashboard {
            student,
            course_progress,
            recent_achievements,
            token_balance,
            staking_positions,
            leaderboard_positions,
            ml_insights,
            recommendations,
            last_updated: self.env.ledger().timestamp(),
        })
    }
    
    /// Get instructor analytics dashboard
    pub fn get_instructor_dashboard(
        &self,
        instructor: Address,
    ) -> Result<InstructorDashboard, soroban_sdk::Error> {
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        
        // 1. Get instructor's courses
        let courses = analytics_client.get_instructor_courses(&instructor)?;
        let mut course_analytics = Vec::new(&self.env);
        
        for course_id in courses {
            let analytics = analytics_client.get_course_analytics(&course_id)?;
            course_analytics.push_back(CourseAnalyticsSummary {
                course_id,
                total_enrollments: analytics.total_enrollments,
                active_students: analytics.active_students,
                completion_rate: analytics.completion_rate,
                average_score: analytics.average_score,
            });
        }
        
        // 2. Get student performance insights
        let student_insights = analytics_client.get_student_performance_insights(&instructor)?;
        
        // 3. Get course engagement metrics
        let engagement_metrics = analytics_client.get_course_engagement_metrics(&instructor)?;
        
        Ok(InstructorDashboard {
            instructor,
            course_analytics,
            student_insights,
            engagement_metrics,
            last_updated: self.env.ledger().timestamp(),
        })
    }
}
```

### 2. Automated Report Generation

```rust
impl StrellerMindsClient {
    /// Generate comprehensive progress report
    pub fn generate_progress_report(
        &self,
        student: Address,
        course_id: CourseId,
        period: ReportPeriod,
    ) -> Result<ProgressReport, soroban_sdk::Error> {
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        
        // 1. Generate base report
        let report = analytics_client.generate_progress_report(
            &student,
            &course_id,
            &period
        )?;
        
        // 2. Add ML-powered insights
        let ml_insights = analytics_client.generate_ml_insights_for_period(
            &student,
            &course_id,
            &period
        )?;
        report.ml_insights = ml_insights;
        
        // 3. Add peer comparison data
        let peer_comparison = analytics_client.generate_peer_comparison(
            &student,
            &course_id,
            &period
        )?;
        report.peer_comparison = peer_comparison;
        
        // 4. Add personalized recommendations
        let recommendations = analytics_client.generate_personalized_recommendations(
            &student,
            &course_id,
            &report
        )?;
        report.recommendations = recommendations;
        
        // 5. Store report for future reference
        analytics_client.store_progress_report(&report)?;
        
        Ok(report)
    }
    
    /// Schedule automated report generation
    pub fn schedule_automated_reports(
        &self,
        admin: Address,
        schedule: ReportSchedule,
    ) -> Result<(), soroban_sdk::Error> {
        let analytics_client = AnalyticsClient::new(&self.env, &self.analytics_address);
        
        // 1. Validate admin permissions
        let shared_client = AccessControlClient::new(&self.env, &self.shared_address);
        shared_client.require_role(&admin, Role::Admin)?;
        
        // 2. Create report generation schedule
        analytics_client.create_report_schedule(&admin, &schedule)?;
        
        // 3. Set up automated triggers
        for trigger in schedule.triggers {
            analytics_client.create_report_trigger(&admin, &trigger)?;
        }
        
        Ok(())
    }
}
```

## Frontend Integration Examples

### 1. React/JavaScript Integration

```javascript
// frontend/src/services/strellerminds.js
import { 
  Address, 
  SorobanClient,
  Contract 
} from '@stellar/stellar-sdk';

class StrellerMindsService {
  constructor(config) {
    this.client = new SorobanClient(config.rpcUrl);
    this.contracts = {
      shared: new Contract(config.sharedContractAddress),
      analytics: new Contract(config.analyticsContractAddress),
      token: new Contract(config.tokenContractAddress),
    };
  }

  // Initialize user session
  async initializeUser(userAddress, userRole) {
    try {
      // Check if user exists
      const userExists = await this.checkUserExists(userAddress);
      
      if (!userExists) {
        // Register new user
        await this.registerUser(userAddress, userRole);
      }

      // Get user profile
      const profile = await this.getUserProfile(userAddress);
      return profile;
    } catch (error) {
      console.error('Failed to initialize user:', error);
      throw error;
    }
  }

  // Start learning session
  async startLearningSession(studentAddress, courseId) {
    const sessionData = {
      student: new Address(studentAddress),
      course_id: courseId,
      start_time: Date.now(),
      session_type: 'lecture',
    };

    const result = await this.invokeContract(
      'analytics',
      'record_session',
      sessionData
    );

    return result.session_id;
  }

  // Update session progress
  async updateSessionProgress(sessionId, progressData) {
    const updateData = {
      session_id: sessionId,
      metrics: {
        time_spent: progressData.timeSpent,
        pages_viewed: progressData.pagesViewed,
        exercises_completed: progressData.exercisesCompleted,
        interaction_count: progressData.interactions,
      },
      events: progressData.events,
    };

    return await this.invokeContract(
      'analytics',
      'update_session',
      updateData
    );
  }

  // Complete learning session
  async completeLearningSession(sessionId, finalMetrics) {
    const completionData = {
      session_id: sessionId,
      end_time: Date.now(),
      final_metrics: {
        score: finalMetrics.score,
        completion_percentage: finalMetrics.completionPercentage,
        engagement_score: finalMetrics.engagementScore,
      },
    };

    return await this.invokeContract(
      'analytics',
      'complete_session',
      completionData
    );
  }

  // Get student dashboard
  async getStudentDashboard(studentAddress) {
    return await this.invokeContract(
      'analytics',
      'get_student_dashboard',
      { student: new Address(studentAddress) }
    );
  }

  // Helper method to invoke contracts
  async invokeContract(contractName, method, params) {
    const contract = this.contracts[contractName];
    const operation = contract.call(method, ...Object.values(params));
    
    const result = await this.client.sendTransaction(operation);
    return result.result;
  }
}

// React Hook for StrellerMinds integration
export function useStrellerMinds(config) {
  const [service] = useState(() => new StrellerMindsService(config));
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const initializeUser = useCallback(async (address, role) => {
    setLoading(true);
    setError(null);
    
    try {
      const userProfile = await service.initializeUser(address, role);
      setUser(userProfile);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }, [service]);

  const startSession = useCallback(async (courseId) => {
    if (!user) throw new Error('User not initialized');
    
    return await service.startLearningSession(user.address, courseId);
  }, [service, user]);

  return {
    user,
    loading,
    error,
    initializeUser,
    startSession,
    service,
  };
}
```

### 2. Vue.js Integration

```javascript
// frontend/src/composables/useStrellerMinds.js
import { ref, computed } from 'vue';
import { StrellerMindsService } from '@/services/strellerminds';

export function useStrellerMinds(config) {
  const service = new StrellerMindsService(config);
  
  // Reactive state
  const user = ref(null);
  const loading = ref(false);
  const error = ref(null);
  const dashboard = ref(null);

  // Computed properties
  const isAuthenticated = computed(() => user.value !== null);
  const userRole = computed(() => user.value?.role);
  const tokenBalance = computed(() => user.value?.token_balance || 0);

  // Methods
  const connectWallet = async () => {
    loading.value = true;
    error.value = null;
    
    try {
      // Connect to wallet (e.g., Freighter, Albedo)
      const wallet = await connectWalletService();
      const address = await wallet.getPublicKey();
      
      // Initialize user
      const userProfile = await service.initializeUser(address, 'student');
      user.value = userProfile;
      
      // Load dashboard
      await loadDashboard();
    } catch (err) {
      error.value = err.message;
    } finally {
      loading.value = false;
    }
  };

  const loadDashboard = async () => {
    if (!user.value) return;
    
    try {
      dashboard.value = await service.getStudentDashboard(user.value.address);
    } catch (err) {
      console.error('Failed to load dashboard:', err);
    }
  };

  const startLearningSession = async (courseId) => {
    loading.value = true;
    
    try {
      const sessionId = await service.startLearningSession(user.value.address, courseId);
      
      // Store session ID for tracking
      localStorage.setItem('currentSessionId', sessionId);
      
      return sessionId;
    } catch (err) {
      error.value = err.message;
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const trackProgress = async (progressData) => {
    const sessionId = localStorage.getItem('currentSessionId');
    if (!sessionId) return;
    
    try {
      await service.updateSessionProgress(sessionId, progressData);
      await loadDashboard(); // Refresh dashboard
    } catch (err) {
      console.error('Failed to track progress:', err);
    }
  };

  return {
    // State
    user,
    loading,
    error,
    dashboard,
    
    // Computed
    isAuthenticated,
    userRole,
    tokenBalance,
    
    // Methods
    connectWallet,
    loadDashboard,
    startLearningSession,
    trackProgress,
  };
}
```

## Mobile App Integration

### 1. React Native Integration

```javascript
// mobile/src/services/StrellerMindsService.js
import { 
  Address,
  SorobanClient 
} from '@stellar/stellar-sdk';

class StrellerMindsMobileService {
  constructor(config) {
    this.client = new SorobanClient(config.rpcUrl);
    this.config = config;
  }

  // Initialize mobile user with device-specific features
  async initializeMobileUser(userAddress, deviceInfo) {
    try {
      // Register user with mobile-specific data
      const userProfile = await this.registerMobileUser(userAddress, deviceInfo);
      
      // Enable push notifications
      await this.enablePushNotifications(userAddress);
      
      // Set up offline sync
      await this.setupOfflineSync(userAddress);
      
      return userProfile;
    } catch (error) {
      console.error('Failed to initialize mobile user:', error);
      throw error;
    }
  }

  // Track offline learning activity
  async trackOfflineActivity(sessionData) {
    // Store activity locally when offline
    await this.storeOfflineActivity(sessionData);
    
    // Sync when online
    if (this.isOnline()) {
      await this.syncOfflineActivities();
    }
  }

  // Enable location-based learning (if permitted)
  async enableLocationBasedLearning(userAddress) {
    const hasPermission = await this.requestLocationPermission();
    
    if (hasPermission) {
      // Track location for campus-based learning
      this.startLocationTracking(userAddress);
    }
  }

  // Handle app background/foreground transitions
  async handleAppStateChange(newState, userAddress) {
    if (newState === 'background') {
      // Pause active sessions
      await this.pauseActiveSessions(userAddress);
    } else if (newState === 'active') {
      // Resume sessions and sync data
      await this.resumeSessions(userAddress);
      await this.syncOfflineActivities();
    }
  }

  // Get mobile-optimized dashboard
  async getMobileDashboard(userAddress) {
    const dashboard = await this.getStudentDashboard(userAddress);
    
    // Optimize for mobile viewing
    return {
      ...dashboard,
      mobile_optimized: true,
      quick_actions: this.getMobileQuickActions(dashboard),
      notifications: await this.getMobileNotifications(userAddress),
    };
  }

  private async registerMobileUser(address, deviceInfo) {
    const registrationData = {
      user_address: new Address(address),
      device_info: deviceInfo,
      platform: 'mobile',
      app_version: deviceInfo.appVersion,
    };

    return await this.invokeContract('analytics', 'register_mobile_user', registrationData);
  }

  private async enablePushNotifications(address) {
    const token = await this.getPushNotificationToken();
    
    return await this.invokeContract('analytics', 'register_push_token', {
      user_address: new Address(address),
      push_token: token,
    });
  }
}

// React Native Hook
export function useStrellerMindsMobile(config) {
  const [service] = useState(() => new StrellerMindsMobileService(config));
  const [user, setUser] = useState(null);
  const [isOnline, setIsOnline] = useState(true);

  useEffect(() => {
    // Handle network state changes
    const unsubscribe = NetInfo.addEventListener(state => {
      setIsOnline(state.isConnected);
    });

    // Handle app state changes
    const appStateSubscription = AppState.addEventListener('change', handleAppStateChange);

    return () => {
      unsubscribe();
      appStateSubscription.remove();
    };
  }, []);

  const handleAppStateChange = useCallback(async (nextAppState) => {
    if (user) {
      await service.handleAppStateChange(nextAppState, user.address);
    }
  }, [service, user]);

  return {
    service,
    user,
    isOnline,
    setUser,
  };
}
```

## Testing Integration

### 1. Unit Testing Examples

```rust
// tests/integration_test.rs
use soroban_sdk::Env;
use stellarminds_shared::AccessControlClient;
use stellarminds_analytics::AnalyticsClient;
use stellarminds_token::TokenClient;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_student_journey() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let student = Address::generate(&env);
        let instructor = Address::generate(&env);

        // Deploy contracts
        let shared_contract = deploy_shared_contract(&env);
        let analytics_contract = deploy_analytics_contract(&env);
        let token_contract = deploy_token_contract(&env);

        let shared_client = AccessControlClient::new(&env, &shared_contract);
        let analytics_client = AnalyticsClient::new(&env, &analytics_contract);
        let token_client = TokenClient::new(&env, &token_contract);

        // Initialize platform
        shared_client.initialize(&admin).unwrap();
        analytics_client.initialize(&admin).unwrap();
        token_client.initialize(&admin).unwrap();

        // Register instructor
        shared_client.grant_role(&admin, Role::Instructor, &instructor).unwrap();

        // Create course
        let course_id = analytics_client.create_course(
            &instructor,
            &CourseCreationData {
                name: "Rust Programming".to_string(),
                description: "Learn Rust programming".to_string(),
                duration_seconds: 2592000, // 30 days
                max_students: 100,
            }
        ).unwrap();

        // Register student
        shared_client.grant_role(&admin, Role::Student, &student).unwrap();
        analytics_client.enroll_student(&student, &course_id).unwrap();

        // Start learning session
        let session_id = analytics_client.record_session(&student, &course_id).unwrap();

        // Complete session with good performance
        let final_metrics = SessionMetrics {
            score: Some(95.0),
            completion_percentage: 100,
            engagement_score: 90,
            time_spent: 3600,
        };
        analytics_client.complete_session(&session_id, &final_metrics).unwrap();

        // Verify rewards
        let final_balance = token_client.balance(&student).unwrap();
        assert!(final_balance > 0);

        // Check achievements
        let achievements = analytics_client.get_student_achievements(&student).unwrap();
        assert!(!achievements.is_empty());

        // Verify progress analytics
        let progress = analytics_client.get_progress_analytics(&student, &course_id).unwrap();
        assert_eq!(progress.completion_percentage, 100);
        assert!(progress.average_score >= 95.0);
    }

    #[test]
    fn test_token_economy_integration() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let student = Address::generate(&env);

        // Setup contracts
        let (shared_client, analytics_client, token_client) = setup_test_contracts(&env, &admin);

        // Register student
        shared_client.grant_role(&admin, Role::Student, &student).unwrap();
        token_client.mint(&admin, &student, 1000).unwrap();

        // Test staking
        let pool_id = "test_pool".to_string();
        let staking_pool = StakingPool {
            id: pool_id.clone(),
            name: "Test Pool".to_string(),
            apy_percentage: 10.0,
            min_stake_amount: 100,
            lock_period_seconds: 86400, // 1 day
            max_total_stake: Some(10000),
        };

        token_client.create_staking_pool(&admin, staking_pool).unwrap();
        token_client.stake_tokens(&student, &pool_id, 500).unwrap();

        // Verify staking position
        let positions = token_client.get_staking_positions(&student).unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].amount, 500);

        // Test certificate upgrade
        let certificate_id = "test_cert".to_string();
        analytics_client.issue_certificate(&student, Symbol::new(&env, "TEST"), 
            &CoursePerformance { score: 90.0 }).unwrap();

        token_client.burn_for_upgrade(&student, 200, certificate_id.clone(), "premium".to_string()).unwrap();

        // Verify upgrade
        let certificate = analytics_client.get_certificate(&certificate_id).unwrap();
        assert!(certificate.upgrades.contains(&"premium".to_string()));

        // Verify token balance decreased
        let final_balance = token_client.balance(&student).unwrap();
        assert_eq!(final_balance, 300); // 1000 - 500 (staked) - 200 (burned)
    }
}
```

### 2. Frontend Testing

```javascript
// frontend/src/services/__tests__/strellerminds.test.js
import { StrellerMindsService } from '../strellerminds';
import { mockContractResponses } from './mocks';

describe('StrellerMindsService', () => {
  let service;
  let mockClient;

  beforeEach(() => {
    mockClient = {
      sendTransaction: jest.fn(),
    };
    
    service = new StrellerMindsService({
      rpcUrl: 'https://test-rpc.stellar.org',
      sharedContractAddress: 'test-shared-address',
      analyticsContractAddress: 'test-analytics-address',
      tokenContractAddress: 'test-token-address',
    });
    
    service.client = mockClient;
  });

  describe('User Management', () => {
    test('should initialize new user successfully', async () => {
      const userAddress = 'test-user-address';
      const userRole = 'student';
      
      mockClient.sendTransaction
        .mockResolvedValueOnce({ result: { exists: false } })
        .mockResolvedValueOnce({ result: { registered: true } })
        .mockResolvedValueOnce({ 
          result: { 
            address: userAddress,
            role: userRole,
            token_balance: 100,
            achievements: []
          } 
        });

      const result = await service.initializeUser(userAddress, userRole);

      expect(result.address).toBe(userAddress);
      expect(result.role).toBe(userRole);
      expect(result.token_balance).toBe(100);
    });

    test('should load existing user profile', async () => {
      const userAddress = 'existing-user-address';
      
      mockClient.sendTransaction
        .mockResolvedValueOnce({ result: { exists: true } })
        .mockResolvedValueOnce({ 
          result: { 
            address: userAddress,
            role: 'instructor',
            token_balance: 500,
            achievements: ['first_course']
          } 
        });

      const result = await service.initializeUser(userAddress);

      expect(result.address).toBe(userAddress);
      expect(result.role).toBe('instructor');
      expect(result.achievements).toContain('first_course');
    });
  });

  describe('Learning Sessions', () => {
    test('should start learning session correctly', async () => {
      const studentAddress = 'test-student';
      const courseId = 'rust-101';
      
      mockClient.sendTransaction.mockResolvedValue({
        result: { 
          session_id: 'test-session-123',
          start_time: Date.now()
        }
      });

      const result = await service.startLearningSession(studentAddress, courseId);

      expect(result.session_id).toBe('test-session-123');
      expect(mockClient.sendTransaction).toHaveBeenCalledWith(
        expect.objectContaining({
          method: 'record_session',
          params: expect.objectContaining({
            student: expect.any(Address),
            course_id: courseId,
          })
        })
      );
    });

    test('should update session progress', async () => {
      const sessionId = 'test-session-123';
      const progressData = {
        timeSpent: 1800,
        pagesViewed: 5,
        exercisesCompleted: 3,
        interactions: 25,
      };
      
      mockClient.sendTransaction.mockResolvedValue({
        result: { updated: true }
      });

      await service.updateSessionProgress(sessionId, progressData);

      expect(mockClient.sendTransaction).toHaveBeenCalledWith(
        expect.objectContaining({
          method: 'update_session',
          params: expect.objectContaining({
            session_id: sessionId,
            metrics: expect.objectContaining({
              time_spent: progressData.timeSpent,
              pages_viewed: progressData.pagesViewed,
            })
          })
        })
      );
    });
  });

  describe('Dashboard Data', () => {
    test('should fetch student dashboard', async () => {
      const studentAddress = 'test-student';
      
      mockClient.sendTransaction.mockResolvedValue({
        result: {
          student: studentAddress,
          course_progress: [
            {
              course_id: 'rust-101',
              completion_percentage: 75,
              average_score: 85.5,
              learning_streak: 5,
            }
          ],
          recent_achievements: ['week_streak'],
          token_balance: 250,
          recommendations: ['practice_more_exercises'],
        }
      });

      const dashboard = await service.getStudentDashboard(studentAddress);

      expect(dashboard.student).toBe(studentAddress);
      expect(dashboard.course_progress).toHaveLength(1);
      expect(dashboard.token_balance).toBe(250);
      expect(dashboard.recommendations).toContain('practice_more_exercises');
    });
  });
});
```

## Deployment and Production Integration

### 1. Production Configuration

```typescript
// config/production.ts
export const productionConfig = {
  network: 'public',
  rpcUrl: 'https://horizon.stellar.org',
  contracts: {
    shared: process.env.SHARED_CONTRACT_ADDRESS,
    analytics: process.env.ANALYTICS_CONTRACT_ADDRESS,
    token: process.env.TOKEN_CONTRACT_ADDRESS,
  },
  features: {
    enableMLInsights: true,
    enableRealTimeAnalytics: true,
    enablePushNotifications: true,
    enableOfflineSync: true,
  },
  limits: {
    maxSessionsPerDay: 50,
    maxUploadSize: 10 * 1024 * 1024, // 10MB
    cacheTimeout: 300000, // 5 minutes
  },
  monitoring: {
    enableErrorTracking: true,
    enablePerformanceMonitoring: true,
    enableUserAnalytics: true,
  },
};
```

### 2. Error Handling and Monitoring

```typescript
// services/monitoring.ts
export class MonitoringService {
  private static instance: MonitoringService;
  
  static getInstance(): MonitoringService {
    if (!MonitoringService.instance) {
      MonitoringService.instance = new MonitoringService();
    }
    return MonitoringService.instance;
  }

  trackError(error: Error, context: any) {
    // Send to error tracking service
    console.error('Contract Error:', error, context);
    
    // Report to monitoring dashboard
    this.reportToDashboard({
      type: 'error',
      message: error.message,
      stack: error.stack,
      context,
      timestamp: Date.now(),
    });
  }

  trackPerformance(operation: string, duration: number) {
    if (duration > 5000) { // 5 seconds threshold
      console.warn(`Slow operation: ${operation} took ${duration}ms`);
    }
    
    this.reportToDashboard({
      type: 'performance',
      operation,
      duration,
      timestamp: Date.now(),
    });
  }

  trackUserActivity(activity: UserActivity) {
    this.reportToDashboard({
      type: 'user_activity',
      ...activity,
      timestamp: Date.now(),
    });
  }

  private reportToDashboard(data: any) {
    // Send to monitoring service
    fetch('/api/monitoring', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    }).catch(console.error);
  }
}

// Enhanced service with monitoring
export class MonitoredStrellerMindsService extends StrellerMindsService {
  private monitoring = MonitoringService.getInstance();

  async initializeUser(address: string, role: string) {
    const startTime = Date.now();
    
    try {
      const result = await super.initializeUser(address, role);
      
      this.monitoring.trackPerformance('initialize_user', Date.now() - startTime);
      this.monitoring.trackUserActivity({
        action: 'user_initialized',
        address,
        role,
      });
      
      return result;
    } catch (error) {
      this.monitoring.trackError(error, { address, role, operation: 'initialize_user' });
      throw error;
    }
  }
}
```

## Conclusion

This integration guide provides comprehensive examples and patterns for integrating with the StrellerMinds smart contracts. The examples cover:

- **Basic Setup**: Contract initialization and client configuration
- **User Management**: Registration, role management, and profiles
- **Learning Activities**: Session tracking, progress monitoring, and assessments
- **Token Economy**: Rewards, staking, and certificate upgrades
- **Analytics**: Real-time dashboards and reporting
- **Frontend Integration**: React, Vue.js, and React Native examples
- **Testing**: Comprehensive unit and integration tests
- **Production**: Deployment configuration and monitoring

Key integration benefits:
- **Comprehensive API**: Full access to all contract features
- **Type Safety**: Strong typing for reliable integration
- **Error Handling**: Robust error management and recovery
- **Performance**: Optimized patterns for efficiency
- **Monitoring**: Built-in tracking and analytics
- **Scalability**: Patterns for high-volume applications

These integration patterns enable developers to build sophisticated educational platforms on the StrellerMinds blockchain infrastructure.
