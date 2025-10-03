#[cfg(test)]
mod multisig_tests {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, Address, Env};
    use crate::types::{
        MultiSigConfig, MintCertificateParams, CertificatePriority, MultiSigRequestStatus,
        ApprovalRecord, MultiSigCertificateRequest
    };
    use crate::multisig::MultiSigManager;
    use crate::Certificate;

    fn create_test_env() -> (Env, Address, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let instructor = Address::generate(&env);
        let student = Address::generate(&env);
        let approver1 = Address::generate(&env);
        let approver2 = Address::generate(&env);
        
        (env, admin, instructor, student, approver1, approver2)
    }

    #[test]
    fn test_create_multisig_request_invalid_metadata_rejected() {
        let (env, admin, instructor, student, approver1, approver2) = create_test_env();
        let contract = setup_contract(&env, &admin);

        // Grant instructor role
        contract.grant_role(env.clone(), instructor.clone(), 3).unwrap();

        // Configure multi-sig
        let config = create_test_config(
            &env,
            "COURSE_001",
            2,
            vec![approver1.clone(), approver2.clone()],
        );
        contract.configure_multisig(env.clone(), admin.clone(), config).unwrap();

        // Create invalid params (title too short)
        let mut params = create_test_mint_params(&env, &student, "COURSE_001");
        params.title = String::from_str(&env, "AB");

        let result = contract.create_multisig_request(
            env.clone(),
            instructor.clone(),
            params,
            String::from_str(&env, "Invalid title"),
        );
        assert!(result.is_err());
    }

    fn setup_contract(env: &Env, admin: &Address) -> Certificate {
        let contract = Certificate::new(env, admin.clone());
        contract.initialize(env.clone(), admin.clone()).unwrap();
        contract
    }

    fn create_test_config(
        env: &Env,
        course_id: &str,
        required_approvals: u32,
        approvers: Vec<Address>,
    ) -> MultiSigConfig {
        MultiSigConfig {
            course_id: String::from_str(env, course_id),
            required_approvals,
            authorized_approvers: approvers,
            timeout_duration: 86400, // 24 hours
            priority: CertificatePriority::Premium,
            auto_execute: true,
        }
    }

    fn create_test_mint_params(
        env: &Env,
        student: &Address,
        course_id: &str,
    ) -> MintCertificateParams {
        MintCertificateParams {
            certificate_id: BytesN::from_array(env, &[1u8; 32]),
            course_id: String::from_str(env, course_id),
            student: student.clone(),
            title: String::from_str(env, "Test Certificate"),
            description: String::from_str(env, "Test Description"),
            metadata_uri: String::from_str(env, "https://example.com/metadata"),
            expiry_date: env.ledger().timestamp() + 31536000, // 1 year
        }
    }

    #[test]
    fn test_configure_multisig_success() {
        let (env, admin, _, _, approver1, approver2) = create_test_env();
        let contract = setup_contract(&env, &admin);

        let config = create_test_config(
            &env,
            "COURSE_001",
            2,
            vec![approver1.clone(), approver2.clone()],
        );

        let result = contract.configure_multisig(env.clone(), admin.clone(), config.clone());
        assert!(result.is_ok());

        // Verify configuration was stored
        let stored_config = contract.get_multisig_config(env.clone(), String::from_str(&env, "COURSE_001"));
        assert!(stored_config.is_some());
        assert_eq!(stored_config.unwrap().required_approvals, 2);
    }

    #[test]
    fn test_configure_multisig_invalid_threshold() {
        let (env, admin, _, _, approver1, _) = create_test_env();
        let contract = setup_contract(&env, &admin);

        let mut config = create_test_config(&env, "COURSE_001", 3, vec![approver1.clone()]);
        config.required_approvals = 3; // More than available approvers

        let result = contract.configure_multisig(env.clone(), admin.clone(), config);
        assert!(result.is_err());
    }

    #[test]
    fn test_configure_multisig_timeout_too_short() {
        let (env, admin, _, _, approver1, approver2) = create_test_env();
        let contract = setup_contract(&env, &admin);

        let mut config = create_test_config(
            &env,
            "COURSE_001",
            2,
            vec![approver1.clone(), approver2.clone()],
        );
        config.timeout_duration = 1800; // 30 minutes - too short

        let result = contract.configure_multisig(env.clone(), admin.clone(), config);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_multisig_request_success() {
        let (env, admin, instructor, student, approver1, approver2) = create_test_env();
        let contract = setup_contract(&env, &admin);

        // Grant instructor role
        contract.grant_role(env.clone(), instructor.clone(), 3).unwrap(); // Instructor role

        // Configure multi-sig
        let config = create_test_config(
            &env,
            "COURSE_001",
            2,
            vec![approver1.clone(), approver2.clone()],
        );
        contract.configure_multisig(env.clone(), admin.clone(), config).unwrap();

        // Create request
        let params = create_test_mint_params(&env, &student, "COURSE_001");
        let reason = String::from_str(&env, "High-value certificate request");

        let result = contract.create_multisig_request(
            env.clone(),
            instructor.clone(),
            params,
            reason,
        );
        assert!(result.is_ok());

        let request_id = result.unwrap();
        let request = contract.get_multisig_request(env.clone(), request_id.clone());
        assert!(request.is_some());
        assert_eq!(request.unwrap().status, MultiSigRequestStatus::Pending);
    }

    #[test]
    fn test_create_multisig_request_no_config() {
        let (env, admin, instructor, student, _, _) = create_test_env();
        let contract = setup_contract(&env, &admin);

        // Grant instructor role
        contract.grant_role(env.clone(), instructor.clone(), 3).unwrap();

        // Try to create request without configuration
        let params = create_test_mint_params(&env, &student, "COURSE_001");
        let reason = String::from_str(&env, "Test request");

        let result = contract.create_multisig_request(
            env.clone(),
            instructor.clone(),
            params,
            reason,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_process_multisig_approval_success() {
        let (env, admin, instructor, student, approver1, approver2) = create_test_env();
        let contract = setup_contract(&env, &admin);

        // Setup
        contract.grant_role(env.clone(), instructor.clone(), 3).unwrap();
        let config = create_test_config(
            &env,
            "COURSE_001",
            2,
            vec![approver1.clone(), approver2.clone()],
        );
        contract.configure_multisig(env.clone(), admin.clone(), config).unwrap();

        // Create request
        let params = create_test_mint_params(&env, &student, "COURSE_001");
        let request_id = contract.create_multisig_request(
            env.clone(),
            instructor.clone(),
            params,
            String::from_str(&env, "Test request"),
        ).unwrap();

        // First approval
        let result = contract.process_multisig_approval(
            env.clone(),
            approver1.clone(),
            request_id.clone(),
            true,
            String::from_str(&env, "Approved by approver1"),
            None,
        );
        assert!(result.is_ok());

        // Check request status (should still be pending)
        let request = contract.get_multisig_request(env.clone(), request_id.clone()).unwrap();
        assert_eq!(request.status, MultiSigRequestStatus::Pending);
        assert_eq!(request.current_approvals, 1);

        // Second approval (should trigger auto-execution)
        let result = contract.process_multisig_approval(
            env.clone(),
            approver2.clone(),
            request_id.clone(),
            true,
            String::from_str(&env, "Approved by approver2"),
            None,
        );
        assert!(result.is_ok());

        // Check final status
        let request = contract.get_multisig_request(env.clone(), request_id.clone()).unwrap();
        assert_eq!(request.status, MultiSigRequestStatus::Executed);
        assert_eq!(request.current_approvals, 2);
    }

    #[test]
    fn test_process_multisig_approval_rejection() {
        let (env, admin, instructor, student, approver1, approver2) = create_test_env();
        let contract = setup_contract(&env, &admin);

        // Setup
        contract.grant_role(env.clone(), instructor.clone(), 3).unwrap();
        let config = create_test_config(
            &env,
            "COURSE_001",
            2,
            vec![approver1.clone(), approver2.clone()],
        );
        contract.configure_multisig(env.clone(), admin.clone(), config).unwrap();

        // Create request
        let params = create_test_mint_params(&env, &student, "COURSE_001");
        let request_id = contract.create_multisig_request(
            env.clone(),
            instructor.clone(),
            params,
            String::from_str(&env, "Test request"),
        ).unwrap();

        // Reject the request
        let result = contract.process_multisig_approval(
            env.clone(),
            approver1.clone(),
            request_id.clone(),
            false,
            String::from_str(&env, "Rejected - insufficient documentation"),
            None,
        );
        assert!(result.is_ok());

        // Check request status (should be rejected)
        let request = contract.get_multisig_request(env.clone(), request_id.clone()).unwrap();
        assert_eq!(request.status, MultiSigRequestStatus::Rejected);
    }

    #[test]
    fn test_process_multisig_approval_unauthorized_approver() {
        let (env, admin, instructor, student, approver1, approver2) = create_test_env();
        let contract = setup_contract(&env, &admin);
        let unauthorized_approver = Address::generate(&env);

        // Setup
        contract.grant_role(env.clone(), instructor.clone(), 3).unwrap();
        let config = create_test_config(
            &env,
            "COURSE_001",
            2,
            vec![approver1.clone(), approver2.clone()],
        );
        contract.configure_multisig(env.clone(), admin.clone(), config).unwrap();

        // Create request
        let params = create_test_mint_params(&env, &student, "COURSE_001");
        let request_id = contract.create_multisig_request(
            env.clone(),
            instructor.clone(),
            params,
            String::from_str(&env, "Test request"),
        ).unwrap();

        // Try to approve with unauthorized approver
        let result = contract.process_multisig_approval(
            env.clone(),
            unauthorized_approver,
            request_id.clone(),
            true,
            String::from_str(&env, "Unauthorized approval"),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_process_multisig_approval_duplicate_approval() {
        let (env, admin, instructor, student, approver1, approver2) = create_test_env();
        let contract = setup_contract(&env, &admin);

        // Setup
        contract.grant_role(env.clone(), instructor.clone(), 3).unwrap();
        let config = create_test_config(
            &env,
            "COURSE_001",
            2,
            vec![approver1.clone(), approver2.clone()],
        );
        contract.configure_multisig(env.clone(), admin.clone(), config).unwrap();

        // Create request
        let params = create_test_mint_params(&env, &student, "COURSE_001");
        let request_id = contract.create_multisig_request(
            env.clone(),
            instructor.clone(),
            params,
            String::from_str(&env, "Test request"),
        ).unwrap();

        // First approval
        contract.process_multisig_approval(
            env.clone(),
            approver1.clone(),
            request_id.clone(),
            true,
            String::from_str(&env, "First approval"),
            None,
        ).unwrap();

        // Try to approve again with same approver
        let result = contract.process_multisig_approval(
            env.clone(),
            approver1.clone(),
            request_id.clone(),
            true,
            String::from_str(&env, "Duplicate approval"),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_multisig_request_manual() {
        let (env, admin, instructor, student, approver1, approver2) = create_test_env();
        let contract = setup_contract(&env, &admin);

        // Setup with auto_execute = false
        contract.grant_role(env.clone(), instructor.clone(), 3).unwrap();
        let mut config = create_test_config(
            &env,
            "COURSE_001",
            2,
            vec![approver1.clone(), approver2.clone()],
        );
        config.auto_execute = false;
        contract.configure_multisig(env.clone(), admin.clone(), config).unwrap();

        // Create and approve request
        let params = create_test_mint_params(&env, &student, "COURSE_001");
        let request_id = contract.create_multisig_request(
            env.clone(),
            instructor.clone(),
            params,
            String::from_str(&env, "Test request"),
        ).unwrap();

        // Get approvals
        contract.process_multisig_approval(
            env.clone(),
            approver1.clone(),
            request_id.clone(),
            true,
            String::from_str(&env, "Approved"),
            None,
        ).unwrap();

        contract.process_multisig_approval(
            env.clone(),
            approver2.clone(),
            request_id.clone(),
            true,
            String::from_str(&env, "Approved"),
            None,
        ).unwrap();

        // Manually execute
        let result = contract.execute_multisig_request(
            env.clone(),
            instructor.clone(),
            request_id.clone(),
        );
        assert!(result.is_ok());

        // Verify certificate was created
        let cert_id = BytesN::from_array(&env, &[1u8; 32]);
        let certificate = contract.get_certificate(env.clone(), cert_id);
        assert!(certificate.is_some());
    }

    #[test]
    fn test_get_pending_approvals() {
        let (env, admin, instructor, student, approver1, approver2) = create_test_env();
        let contract = setup_contract(&env, &admin);

        // Setup
        contract.grant_role(env.clone(), instructor.clone(), 3).unwrap();
        let config = create_test_config(
            &env,
            "COURSE_001",
            2,
            vec![approver1.clone(), approver2.clone()],
        );
        contract.configure_multisig(env.clone(), admin.clone(), config).unwrap();

        // Create request
        let params = create_test_mint_params(&env, &student, "COURSE_001");
        let request_id = contract.create_multisig_request(
            env.clone(),
            instructor.clone(),
            params,
            String::from_str(&env, "Test request"),
        ).unwrap();

        // Check pending approvals
        let pending1 = contract.get_pending_approvals(env.clone(), approver1.clone());
        let pending2 = contract.get_pending_approvals(env.clone(), approver2.clone());

        assert_eq!(pending1.len(), 1);
        assert_eq!(pending2.len(), 1);
        assert_eq!(pending1.get(0).unwrap(), request_id);
        assert_eq!(pending2.get(0).unwrap(), request_id);
    }

    #[test]
    fn test_multisig_audit_trail() {
        let (env, admin, instructor, student, approver1, approver2) = create_test_env();
        let contract = setup_contract(&env, &admin);

        // Setup
        contract.grant_role(env.clone(), instructor.clone(), 3).unwrap();
        let config = create_test_config(
            &env,
            "COURSE_001",
            2,
            vec![approver1.clone(), approver2.clone()],
        );
        contract.configure_multisig(env.clone(), admin.clone(), config).unwrap();

        // Create request
        let params = create_test_mint_params(&env, &student, "COURSE_001");
        let request_id = contract.create_multisig_request(
            env.clone(),
            instructor.clone(),
            params,
            String::from_str(&env, "Test request"),
        ).unwrap();

        // Process approvals
        contract.process_multisig_approval(
            env.clone(),
            approver1.clone(),
            request_id.clone(),
            true,
            String::from_str(&env, "Approved by approver1"),
            None,
        ).unwrap();

        contract.process_multisig_approval(
            env.clone(),
            approver2.clone(),
            request_id.clone(),
            true,
            String::from_str(&env, "Approved by approver2"),
            None,
        ).unwrap();

        // Check audit trail
        let audit_trail = contract.get_multisig_audit_trail(env.clone(), request_id);
        assert!(audit_trail.len() >= 3); // Created + 2 approvals + executed
    }

    #[test]
    fn test_certificate_priority_levels() {
        let (env, admin, instructor, student, approver1, approver2) = create_test_env();
        let contract = setup_contract(&env, &admin);

        // Test different priority levels
        let priorities = vec![
            (CertificatePriority::Standard, 1),
            (CertificatePriority::Premium, 2),
            (CertificatePriority::Enterprise, 3),
            (CertificatePriority::Institutional, 5),
        ];

        for (priority, expected_approvals) in priorities {
            let course_id = format!("COURSE_{:?}", priority);
            let mut config = create_test_config(
                &env,
                &course_id,
                expected_approvals,
                vec![approver1.clone(), approver2.clone()],
            );
            config.priority = priority.clone();

            let result = contract.configure_multisig(env.clone(), admin.clone(), config);
            
            if expected_approvals <= 2 {
                assert!(result.is_ok());
            } else {
                // Should fail because we only have 2 approvers
                assert!(result.is_err());
            }
        }
    }
}
