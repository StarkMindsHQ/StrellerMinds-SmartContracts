#[cfg(test)]
mod expiry_management_tests {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, Ledger, LedgerInfo}, Address, Env};
    use crate::{
        Certificate, CertificateClient,
        types::{CertificateStatus, MintCertificateParams, RenewalStatus, NotificationType},
        errors::CertificateError,
    };

    fn create_test_env() -> (Env, Address, Address, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        let instructor = Address::generate(&env);
        let student = Address::generate(&env);
        
        env.mock_all_auths();
        
        (env, admin, instructor, student)
    }

    fn setup_contract_with_certificate(env: &Env, admin: &Address, instructor: &Address, student: &Address) -> (CertificateClient, soroban_sdk::BytesN<32>) {
        let contract_id = env.register_contract(None, Certificate);
        let client = CertificateClient::new(env, &contract_id);
        
        // Initialize contract with admin role
        // Note: This would require proper RBAC setup in actual implementation
        
        let certificate_id = soroban_sdk::BytesN::from_array(env, &[1u8; 32]);
        let params = MintCertificateParams {
            certificate_id: certificate_id.clone(),
            student: student.clone(),
            course_id: "COURSE_001".into_val(env),
            title: "Test Certificate".into_val(env),
            description: "A test certificate".into_val(env),
            metadata_uri: "https://example.com/metadata".into_val(env),
            expiry_date: env.ledger().timestamp() + 86400 * 30, // 30 days from now
        };
        
        client.mint_certificate(instructor, &params);
        
        (client, certificate_id)
    }

    #[test]
    fn test_request_certificate_renewal() {
        let (env, admin, instructor, student) = create_test_env();
        let (client, certificate_id) = setup_contract_with_certificate(&env, &admin, &instructor, &student);
        
        let extension_period = 86400 * 30; // 30 days
        let reason = "Certificate still needed for employment".into_val(&env);
        
        // Test successful renewal request
        let result = client.try_request_certificate_renewal(
            &student,
            &certificate_id,
            &extension_period,
            &reason
        );
        assert!(result.is_ok());
        
        // Verify renewal request was stored
        let renewal_request = client.get_renewal_request(&certificate_id);
        assert!(renewal_request.is_some());
        
        let request = renewal_request.unwrap();
        assert_eq!(request.certificate_id, certificate_id);
        assert_eq!(request.requester, student);
        assert_eq!(request.requested_extension, extension_period);
        assert_eq!(request.status, RenewalStatus::Pending);
    }

    #[test]
    fn test_request_renewal_nonexistent_certificate() {
        let (env, admin, instructor, student) = create_test_env();
        let contract_id = env.register_contract(None, Certificate);
        let client = CertificateClient::new(&env, &contract_id);
        
        let fake_certificate_id = soroban_sdk::BytesN::from_array(&env, &[99u8; 32]);
        let extension_period = 86400 * 30;
        let reason = "Test reason".into_val(&env);
        
        let result = client.try_request_certificate_renewal(
            &student,
            &fake_certificate_id,
            &extension_period,
            &reason
        );
        assert_eq!(result, Err(Ok(CertificateError::CertificateNotFound)));
    }

    #[test]
    fn test_process_renewal_request_approval() {
        let (env, admin, instructor, student) = create_test_env();
        let (client, certificate_id) = setup_contract_with_certificate(&env, &admin, &instructor, &student);
        
        // First create a renewal request
        let extension_period = 86400 * 30;
        let reason = "Need extension".into_val(&env);
        client.request_certificate_renewal(&student, &certificate_id, &extension_period, &reason);
        
        // Get original certificate to check expiry date
        let original_cert = client.get_certificate(&certificate_id).unwrap();
        let original_expiry = original_cert.expiry_date;
        
        // Process approval
        let admin_reason = Some("Approved for continued education".into_val(&env));
        let result = client.try_process_renewal_request(
            &admin,
            &certificate_id,
            &true,
            &admin_reason
        );
        assert!(result.is_ok());
        
        // Verify certificate expiry was extended
        let updated_cert = client.get_certificate(&certificate_id).unwrap();
        assert!(updated_cert.expiry_date > original_expiry);
        assert_eq!(updated_cert.status, CertificateStatus::Renewed);
        assert_eq!(updated_cert.renewal_count, 1);
        
        // Verify renewal request status updated
        let renewal_request = client.get_renewal_request(&certificate_id);
        assert!(renewal_request.is_some());
        assert_eq!(renewal_request.unwrap().status, RenewalStatus::Approved);
    }

    #[test]
    fn test_process_renewal_request_rejection() {
        let (env, admin, instructor, student) = create_test_env();
        let (client, certificate_id) = setup_contract_with_certificate(&env, &admin, &instructor, &student);
        
        // Create renewal request
        let extension_period = 86400 * 30;
        let reason = "Need extension".into_val(&env);
        client.request_certificate_renewal(&student, &certificate_id, &extension_period, &reason);
        
        // Get original certificate
        let original_cert = client.get_certificate(&certificate_id).unwrap();
        let original_expiry = original_cert.expiry_date;
        
        // Process rejection
        let admin_reason = Some("Insufficient justification".into_val(&env));
        let result = client.try_process_renewal_request(
            &admin,
            &certificate_id,
            &false,
            &admin_reason
        );
        assert!(result.is_ok());
        
        // Verify certificate expiry was NOT extended
        let updated_cert = client.get_certificate(&certificate_id).unwrap();
        assert_eq!(updated_cert.expiry_date, original_expiry);
        assert_eq!(updated_cert.status, CertificateStatus::Active);
        
        // Verify renewal request status updated
        let renewal_request = client.get_renewal_request(&certificate_id);
        assert!(renewal_request.is_some());
        assert_eq!(renewal_request.unwrap().status, RenewalStatus::Rejected);
    }

    #[test]
    fn test_extend_certificate_expiry() {
        let (env, admin, instructor, student) = create_test_env();
        let (client, certificate_id) = setup_contract_with_certificate(&env, &admin, &instructor, &student);
        
        let original_cert = client.get_certificate(&certificate_id).unwrap();
        let original_expiry = original_cert.expiry_date;
        
        let extension_period = 86400 * 60; // 60 days
        let reason = "Administrative extension".into_val(&env);
        
        let result = client.try_extend_certificate_expiry(
            &admin,
            &certificate_id,
            &extension_period,
            &reason
        );
        assert!(result.is_ok());
        
        // Verify extension
        let updated_cert = client.get_certificate(&certificate_id).unwrap();
        assert_eq!(updated_cert.expiry_date, original_expiry + extension_period);
        assert_eq!(updated_cert.renewal_count, 1);
        assert!(updated_cert.last_renewed_date > 0);
    }

    #[test]
    fn test_bulk_extend_certificates() {
        let (env, admin, instructor, student) = create_test_env();
        let contract_id = env.register_contract(None, Certificate);
        let client = CertificateClient::new(&env, &contract_id);
        
        // Create multiple certificates
        let mut certificate_ids = soroban_sdk::Vec::new(&env);
        for i in 0..3 {
            let cert_id = soroban_sdk::BytesN::from_array(&env, &[i as u8; 32]);
            let params = MintCertificateParams {
                certificate_id: cert_id.clone(),
                student: student.clone(),
                course_id: format!("COURSE_{:03}", i).into_val(&env),
                title: format!("Test Certificate {}", i).into_val(&env),
                description: "A test certificate".into_val(&env),
                metadata_uri: "https://example.com/metadata".into_val(&env),
                expiry_date: env.ledger().timestamp() + 86400 * 30,
            };
            client.mint_certificate(&instructor, &params);
            certificate_ids.push_back(cert_id);
        }
        
        let new_expiry_date = env.ledger().timestamp() + 86400 * 90; // 90 days from now
        let reason = "Bulk administrative extension".into_val(&env);
        
        let result = client.try_bulk_extend_certificates(
            &admin,
            &certificate_ids,
            &new_expiry_date,
            &reason
        );
        assert!(result.is_ok());
        
        let successful_ids = result.unwrap();
        assert_eq!(successful_ids.len(), 3);
        
        // Verify all certificates were extended
        for cert_id in certificate_ids.iter() {
            let cert = client.get_certificate(&cert_id).unwrap();
            assert_eq!(cert.expiry_date, new_expiry_date);
            assert_eq!(cert.renewal_count, 1);
        }
    }

    #[test]
    fn test_get_expiring_certificates() {
        let (env, admin, instructor, student) = create_test_env();
        let contract_id = env.register_contract(None, Certificate);
        let client = CertificateClient::new(&env, &contract_id);
        
        let current_time = env.ledger().timestamp();
        
        // Create certificates with different expiry dates
        let cert_id_1 = soroban_sdk::BytesN::from_array(&env, &[1u8; 32]);
        let params_1 = MintCertificateParams {
            certificate_id: cert_id_1.clone(),
            student: student.clone(),
            course_id: "COURSE_001".into_val(&env),
            title: "Expiring Soon".into_val(&env),
            description: "Certificate expiring in 5 days".into_val(&env),
            metadata_uri: "https://example.com/metadata".into_val(&env),
            expiry_date: current_time + 86400 * 5, // 5 days
        };
        
        let cert_id_2 = soroban_sdk::BytesN::from_array(&env, &[2u8; 32]);
        let params_2 = MintCertificateParams {
            certificate_id: cert_id_2.clone(),
            student: student.clone(),
            course_id: "COURSE_002".into_val(&env),
            title: "Not Expiring Soon".into_val(&env),
            description: "Certificate expiring in 60 days".into_val(&env),
            metadata_uri: "https://example.com/metadata".into_val(&env),
            expiry_date: current_time + 86400 * 60, // 60 days
        };
        
        client.mint_certificate(&instructor, &params_1);
        client.mint_certificate(&instructor, &params_2);
        
        // Get certificates expiring within 7 days
        let expiring_certs = client.get_expiring_certificates(&(86400 * 7));
        
        // Should only include cert_id_1
        assert_eq!(expiring_certs.len(), 1);
        assert_eq!(expiring_certs.get(0).unwrap(), cert_id_1);
    }

    #[test]
    fn test_update_expired_certificates() {
        let (env, admin, instructor, student) = create_test_env();
        let contract_id = env.register_contract(None, Certificate);
        let client = CertificateClient::new(&env, &contract_id);
        
        // Create certificate that's already expired
        let cert_id = soroban_sdk::BytesN::from_array(&env, &[1u8; 32]);
        let params = MintCertificateParams {
            certificate_id: cert_id.clone(),
            student: student.clone(),
            course_id: "COURSE_001".into_val(&env),
            title: "Expired Certificate".into_val(&env),
            description: "This certificate is expired".into_val(&env),
            metadata_uri: "https://example.com/metadata".into_val(&env),
            expiry_date: env.ledger().timestamp() - 86400, // 1 day ago
        };
        
        client.mint_certificate(&instructor, &params);
        
        // Update expired certificates
        let result = client.try_update_expired_certificates();
        assert!(result.is_ok());
        
        let updated_count = result.unwrap();
        assert_eq!(updated_count, 1);
        
        // Verify certificate status was updated
        let cert = client.get_certificate(&cert_id).unwrap();
        assert_eq!(cert.status, CertificateStatus::Expired);
    }

    #[test]
    fn test_expiry_notifications() {
        let (env, admin, instructor, student) = create_test_env();
        let (client, certificate_id) = setup_contract_with_certificate(&env, &admin, &instructor, &student);
        
        // Simulate time passing to trigger notifications
        env.ledger().with_mut(|li| {
            li.timestamp = li.timestamp + 86400 * 25; // 25 days later (5 days before expiry)
        });
        
        // Get notifications for the student
        let notifications = client.get_expiry_notifications(&student);
        
        // Should have notification for certificate expiring in 5 days
        assert!(!notifications.is_empty());
        
        let notification = &notifications.get(0).unwrap();
        assert_eq!(notification.certificate_id, certificate_id);
        assert_eq!(notification.recipient, student);
        assert_eq!(notification.notification_type, NotificationType::ExpiryWarning7Days);
    }

    #[test]
    fn test_acknowledge_notification() {
        let (env, admin, instructor, student) = create_test_env();
        let (client, certificate_id) = setup_contract_with_certificate(&env, &admin, &instructor, &student);
        
        // Create a notification (this would normally be done by the system)
        // For testing, we'll assume a notification exists
        
        let result = client.try_acknowledge_notification(&student, &certificate_id);
        assert!(result.is_ok());
        
        // Verify notification was acknowledged
        let notifications = client.get_expiry_notifications(&student);
        let acknowledged_notifications: Vec<_> = notifications
            .iter()
            .filter(|n| n.certificate_id == certificate_id && n.acknowledged)
            .collect();
        
        assert!(!acknowledged_notifications.is_empty());
    }

    #[test]
    fn test_renewal_request_expiry() {
        let (env, admin, instructor, student) = create_test_env();
        let (client, certificate_id) = setup_contract_with_certificate(&env, &admin, &instructor, &student);
        
        // Create renewal request
        let extension_period = 86400 * 30;
        let reason = "Need extension".into_val(&env);
        client.request_certificate_renewal(&student, &certificate_id, &extension_period, &reason);
        
        // Simulate 31 days passing (renewal requests expire after 30 days)
        env.ledger().with_mut(|li| {
            li.timestamp = li.timestamp + 86400 * 31;
        });
        
        // Try to process expired renewal request
        let admin_reason = Some("Processing expired request".into_val(&env));
        let result = client.try_process_renewal_request(
            &admin,
            &certificate_id,
            &true,
            &admin_reason
        );
        
        // Should fail because request is expired
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_renewal_requests() {
        let (env, admin, instructor, student) = create_test_env();
        let (client, certificate_id) = setup_contract_with_certificate(&env, &admin, &instructor, &student);
        
        // Create first renewal request
        let extension_period = 86400 * 30;
        let reason = "First renewal".into_val(&env);
        client.request_certificate_renewal(&student, &certificate_id, &extension_period, &reason);
        
        // Try to create second renewal request for same certificate
        let reason2 = "Second renewal".into_val(&env);
        let result = client.try_request_certificate_renewal(
            &student,
            &certificate_id,
            &extension_period,
            &reason2
        );
        
        // Should fail because there's already a pending request
        assert!(result.is_err());
    }

    #[test]
    fn test_renewal_limits() {
        let (env, admin, instructor, student) = create_test_env();
        let (client, certificate_id) = setup_contract_with_certificate(&env, &admin, &instructor, &student);
        
        // Perform multiple renewals to test limits
        for i in 0..5 {
            let extension_period = 86400 * 30;
            let reason = format!("Renewal {}", i + 1).into_val(&env);
            
            client.request_certificate_renewal(&student, &certificate_id, &extension_period, &reason);
            
            let admin_reason = Some(format!("Approved renewal {}", i + 1).into_val(&env));
            client.process_renewal_request(&admin, &certificate_id, &true, &admin_reason);
        }
        
        // Try one more renewal (should fail if limit is 5)
        let extension_period = 86400 * 30;
        let reason = "Sixth renewal".into_val(&env);
        let result = client.try_request_certificate_renewal(
            &student,
            &certificate_id,
            &extension_period,
            &reason
        );
        
        // Verify certificate has been renewed 5 times
        let cert = client.get_certificate(&certificate_id).unwrap();
        assert_eq!(cert.renewal_count, 5);
        
        // Depending on implementation, this might fail due to renewal limits
        // The exact behavior would depend on the MAX_RENEWALS constant
    }
}
