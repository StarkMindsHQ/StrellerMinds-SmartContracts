#[cfg(test)]
mod multisig_integration_tests {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, Events}, Address, Env};
    use crate::types::{
        MultiSigConfig, MintCertificateParams, CertificatePriority, MultiSigRequestStatus,
        CertificateStatus
    };
    use crate::Certificate;

    fn setup_full_multisig_scenario() -> (
        Env,
        Certificate,
        Address, // admin
        Address, // instructor
        Address, // student
        Vec<Address>, // approvers
    ) {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let instructor = Address::generate(&env);
        let student = Address::generate(&env);
        let approvers = vec![
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];
        
        let contract = Certificate::new(&env, admin.clone());
        contract.initialize(env.clone(), admin.clone()).unwrap();
        
        // Grant instructor role
        contract.grant_role(env.clone(), instructor.clone(), 3).unwrap();
        
        (env, contract, admin, instructor, student, approvers)
    }

    #[test]
    fn test_end_to_end_multisig_workflow() {
        let (env, contract, admin, instructor, student, approvers) = setup_full_multisig_scenario();

        // Step 1: Configure multi-signature for enterprise course
        let config = MultiSigConfig {
            course_id: String::from_str(&env, "ENTERPRISE_COURSE_001"),
            required_approvals: 3,
            authorized_approvers: approvers.clone(),
            timeout_duration: 86400, // 24 hours
            priority: CertificatePriority::Enterprise,
            auto_execute: true,
        };

        contract.configure_multisig(env.clone(), admin.clone(), config).unwrap();

        // Step 2: Create multi-signature certificate request
        let params = MintCertificateParams {
            certificate_id: BytesN::from_array(&env, &[1u8; 32]),
            course_id: String::from_str(&env, "ENTERPRISE_COURSE_001"),
            student: student.clone(),
            title: String::from_str(&env, "Enterprise Blockchain Certification"),
            description: String::from_str(&env, "Advanced enterprise blockchain development certification"),
            metadata_uri: String::from_str(&env, "https://enterprise.example.com/cert/metadata"),
            expiry_date: env.ledger().timestamp() + 31536000, // 1 year
        };

        let request_id = contract.create_multisig_request(
            env.clone(),
            instructor.clone(),
            params.clone(),
            String::from_str(&env, "High-value enterprise certification requiring multiple approvals"),
        ).unwrap();

        // Verify request was created
        let request = contract.get_multisig_request(env.clone(), request_id.clone()).unwrap();
        assert_eq!(request.status, MultiSigRequestStatus::Pending);
        assert_eq!(request.required_approvals, 3);
        assert_eq!(request.current_approvals, 0);

        // Step 3: Process approvals sequentially
        // First approval
        contract.process_multisig_approval(
            env.clone(),
            approvers[0].clone(),
            request_id.clone(),
            true,
            String::from_str(&env, "Approved - student demonstrated excellent technical skills"),
            Some(BytesN::from_array(&env, &[0xA1u8; 32])),
        ).unwrap();

        let request = contract.get_multisig_request(env.clone(), request_id.clone()).unwrap();
        assert_eq!(request.current_approvals, 1);
        assert_eq!(request.status, MultiSigRequestStatus::Pending);

        // Second approval
        contract.process_multisig_approval(
            env.clone(),
            approvers[1].clone(),
            request_id.clone(),
            true,
            String::from_str(&env, "Approved - comprehensive project portfolio reviewed"),
            Some(BytesN::from_array(&env, &[0xB2u8; 32])),
        ).unwrap();

        let request = contract.get_multisig_request(env.clone(), request_id.clone()).unwrap();
        assert_eq!(request.current_approvals, 2);
        assert_eq!(request.status, MultiSigRequestStatus::Pending);

        // Third approval (should trigger auto-execution)
        contract.process_multisig_approval(
            env.clone(),
            approvers[2].clone(),
            request_id.clone(),
            true,
            String::from_str(&env, "Final approval - all requirements met"),
            Some(BytesN::from_array(&env, &[0xC3u8; 32])),
        ).unwrap();

        // Step 4: Verify final state
        let request = contract.get_multisig_request(env.clone(), request_id.clone()).unwrap();
        assert_eq!(request.current_approvals, 3);
        assert_eq!(request.status, MultiSigRequestStatus::Executed);

        // Verify certificate was actually created
        let certificate = contract.get_certificate(env.clone(), params.certificate_id).unwrap();
        assert_eq!(certificate.status, CertificateStatus::Active);
        assert_eq!(certificate.student_id, student);
        assert_eq!(certificate.instructor_id, instructor);

        // Verify audit trail
        let audit_trail = contract.get_multisig_audit_trail(env.clone(), request_id);
        assert!(audit_trail.len() >= 5); // Created + 3 approvals + executed

        // Verify events were emitted
        let events = env.events().all();
        assert!(events.len() > 0);
    }

    #[test]
    fn test_multisig_rejection_workflow() {
        let (env, contract, admin, instructor, student, approvers) = setup_full_multisig_scenario();

        // Configure multi-signature
        let config = MultiSigConfig {
            course_id: String::from_str(&env, "PREMIUM_COURSE_001"),
            required_approvals: 2,
            authorized_approvers: approvers[0..2].to_vec(),
            timeout_duration: 86400,
            priority: CertificatePriority::Premium,
            auto_execute: false,
        };

        contract.configure_multisig(env.clone(), admin.clone(), config).unwrap();

        // Create request
        let params = MintCertificateParams {
            certificate_id: BytesN::from_array(&env, &[2u8; 32]),
            course_id: String::from_str(&env, "PREMIUM_COURSE_001"),
            student: student.clone(),
            title: String::from_str(&env, "Premium Certification"),
            description: String::from_str(&env, "Premium level certification"),
            metadata_uri: String::from_str(&env, "https://premium.example.com/metadata"),
            expiry_date: env.ledger().timestamp() + 31536000,
        };

        let request_id = contract.create_multisig_request(
            env.clone(),
            instructor.clone(),
            params.clone(),
            String::from_str(&env, "Premium certification request"),
        ).unwrap();

        // First approver rejects
        contract.process_multisig_approval(
            env.clone(),
            approvers[0].clone(),
            request_id.clone(),
            false,
            String::from_str(&env, "Rejected - insufficient project quality"),
            None,
        ).unwrap();

        // Verify request was rejected
        let request = contract.get_multisig_request(env.clone(), request_id.clone()).unwrap();
        assert_eq!(request.status, MultiSigRequestStatus::Rejected);

        // Verify certificate was NOT created
        let certificate = contract.get_certificate(env.clone(), params.certificate_id);
        assert!(certificate.is_none());
    }

    #[test]
    fn test_multisig_timeout_scenario() {
        let (env, contract, admin, instructor, student, approvers) = setup_full_multisig_scenario();

        // Configure with short timeout for testing
        let config = MultiSigConfig {
            course_id: String::from_str(&env, "TIMEOUT_COURSE"),
            required_approvals: 2,
            authorized_approvers: approvers[0..2].to_vec(),
            timeout_duration: 3600, // 1 hour
            priority: CertificatePriority::Premium,
            auto_execute: false,
        };

        contract.configure_multisig(env.clone(), admin.clone(), config).unwrap();

        // Create request
        let params = MintCertificateParams {
            certificate_id: BytesN::from_array(&env, &[3u8; 32]),
            course_id: String::from_str(&env, "TIMEOUT_COURSE"),
            student: student.clone(),
            title: String::from_str(&env, "Timeout Test Cert"),
            description: String::from_str(&env, "Testing timeout behavior"),
            metadata_uri: String::from_str(&env, "https://timeout.example.com/metadata"),
            expiry_date: env.ledger().timestamp() + 31536000,
        };

        let request_id = contract.create_multisig_request(
            env.clone(),
            instructor.clone(),
            params.clone(),
            String::from_str(&env, "Timeout test request"),
        ).unwrap();

        // Simulate time passing beyond timeout
        env.ledger().with_mut(|li| {
            li.timestamp = li.timestamp + 7200; // 2 hours later
        });

        // Try to approve after timeout
        let result = contract.process_multisig_approval(
            env.clone(),
            approvers[0].clone(),
            request_id.clone(),
            true,
            String::from_str(&env, "Late approval"),
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_concurrent_requests() {
        let (env, contract, admin, instructor, student, approvers) = setup_full_multisig_scenario();

        // Configure multi-signature
        let config = MultiSigConfig {
            course_id: String::from_str(&env, "CONCURRENT_COURSE"),
            required_approvals: 2,
            authorized_approvers: approvers[0..2].to_vec(),
            timeout_duration: 86400,
            priority: CertificatePriority::Premium,
            auto_execute: true,
        };

        contract.configure_multisig(env.clone(), admin.clone(), config).unwrap();

        // Create multiple requests
        let mut request_ids = Vec::new();
        for i in 0..3 {
            let params = MintCertificateParams {
                certificate_id: BytesN::from_array(&env, &[(i + 10) as u8; 32]),
                course_id: String::from_str(&env, "CONCURRENT_COURSE"),
                student: student.clone(),
                title: String::from_str(&env, &format!("Concurrent Cert {}", i)),
                description: String::from_str(&env, &format!("Concurrent test {}", i)),
                metadata_uri: String::from_str(&env, &format!("https://concurrent{}.example.com", i)),
                expiry_date: env.ledger().timestamp() + 31536000,
            };

            let request_id = contract.create_multisig_request(
                env.clone(),
                instructor.clone(),
                params,
                String::from_str(&env, &format!("Concurrent request {}", i)),
            ).unwrap();

            request_ids.push(request_id);
        }

        // Verify all requests are pending
        for request_id in &request_ids {
            let request = contract.get_multisig_request(env.clone(), request_id.clone()).unwrap();
            assert_eq!(request.status, MultiSigRequestStatus::Pending);
        }

        // Approve all requests
        for request_id in &request_ids {
            contract.process_multisig_approval(
                env.clone(),
                approvers[0].clone(),
                request_id.clone(),
                true,
                String::from_str(&env, "Batch approval 1"),
                None,
            ).unwrap();

            contract.process_multisig_approval(
                env.clone(),
                approvers[1].clone(),
                request_id.clone(),
                true,
                String::from_str(&env, "Batch approval 2"),
                None,
            ).unwrap();
        }

        // Verify all requests were executed
        for request_id in &request_ids {
            let request = contract.get_multisig_request(env.clone(), request_id.clone()).unwrap();
            assert_eq!(request.status, MultiSigRequestStatus::Executed);
        }
    }

    #[test]
    fn test_multisig_with_different_priority_levels() {
        let (env, contract, admin, instructor, student, approvers) = setup_full_multisig_scenario();

        let test_cases = vec![
            (CertificatePriority::Premium, 2, true),
            (CertificatePriority::Enterprise, 3, true),
        ];

        for (priority, required_approvals, should_succeed) in test_cases {
            let course_id = format!("PRIORITY_{:?}", priority);
            let config = MultiSigConfig {
                course_id: String::from_str(&env, &course_id),
                required_approvals,
                authorized_approvers: approvers.clone(),
                timeout_duration: 86400,
                priority: priority.clone(),
                auto_execute: true,
            };

            let config_result = contract.configure_multisig(env.clone(), admin.clone(), config);
            assert_eq!(config_result.is_ok(), should_succeed);

            if should_succeed {
                // Test creating and approving a request
                let params = MintCertificateParams {
                    certificate_id: BytesN::from_array(&env, &[priority as u8; 32]),
                    course_id: String::from_str(&env, &course_id),
                    student: student.clone(),
                    title: String::from_str(&env, &format!("{:?} Certificate", priority)),
                    description: String::from_str(&env, &format!("{:?} level cert", priority)),
                    metadata_uri: String::from_str(&env, "https://priority.example.com"),
                    expiry_date: env.ledger().timestamp() + 31536000,
                };

                let request_id = contract.create_multisig_request(
                    env.clone(),
                    instructor.clone(),
                    params,
                    String::from_str(&env, &format!("{:?} priority request", priority)),
                ).unwrap();

                // Provide required approvals
                for i in 0..required_approvals {
                    contract.process_multisig_approval(
                        env.clone(),
                        approvers[i as usize].clone(),
                        request_id.clone(),
                        true,
                        String::from_str(&env, &format!("Approval {}", i + 1)),
                        None,
                    ).unwrap();
                }

                // Verify execution
                let request = contract.get_multisig_request(env.clone(), request_id).unwrap();
                assert_eq!(request.status, MultiSigRequestStatus::Executed);
            }
        }
    }

    #[test]
    fn test_multisig_audit_trail_completeness() {
        let (env, contract, admin, instructor, student, approvers) = setup_full_multisig_scenario();

        // Configure multi-signature
        let config = MultiSigConfig {
            course_id: String::from_str(&env, "AUDIT_COURSE"),
            required_approvals: 2,
            authorized_approvers: approvers[0..2].to_vec(),
            timeout_duration: 86400,
            priority: CertificatePriority::Premium,
            auto_execute: true,
        };

        contract.configure_multisig(env.clone(), admin.clone(), config).unwrap();

        // Create request
        let params = MintCertificateParams {
            certificate_id: BytesN::from_array(&env, &[99u8; 32]),
            course_id: String::from_str(&env, "AUDIT_COURSE"),
            student: student.clone(),
            title: String::from_str(&env, "Audit Trail Test"),
            description: String::from_str(&env, "Testing audit trail"),
            metadata_uri: String::from_str(&env, "https://audit.example.com"),
            expiry_date: env.ledger().timestamp() + 31536000,
        };

        let request_id = contract.create_multisig_request(
            env.clone(),
            instructor.clone(),
            params,
            String::from_str(&env, "Audit trail test request"),
        ).unwrap();

        // Process approvals
        contract.process_multisig_approval(
            env.clone(),
            approvers[0].clone(),
            request_id.clone(),
            true,
            String::from_str(&env, "First approval with detailed comments"),
            Some(BytesN::from_array(&env, &[0xABu8; 32])),
        ).unwrap();

        contract.process_multisig_approval(
            env.clone(),
            approvers[1].clone(),
            request_id.clone(),
            true,
            String::from_str(&env, "Second approval completing the process"),
            Some(BytesN::from_array(&env, &[0xCDu8; 32])),
        ).unwrap();

        // Verify comprehensive audit trail
        let audit_trail = contract.get_multisig_audit_trail(env.clone(), request_id);
        
        // Should have: RequestCreated + ApprovalGranted + ApprovalGranted + CertificateIssued
        assert_eq!(audit_trail.len(), 4);

        // Verify audit entry details
        let creation_entry = &audit_trail.get(0).unwrap();
        assert_eq!(creation_entry.actor, instructor);
        
        let first_approval = &audit_trail.get(1).unwrap();
        assert_eq!(first_approval.actor, approvers[0]);
        
        let second_approval = &audit_trail.get(2).unwrap();
        assert_eq!(second_approval.actor, approvers[1]);
        
        let execution_entry = &audit_trail.get(3).unwrap();
        assert_eq!(execution_entry.actor, instructor);
    }
}
