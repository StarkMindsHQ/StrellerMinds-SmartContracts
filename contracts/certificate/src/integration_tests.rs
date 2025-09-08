#[cfg(test)]
mod integration_tests {
    use crate::{Certificate, CertificateClient};
    use crate::types::{MintCertificateParams, CertificateStatus};
    use crate::errors::CertificateError;
    use soroban_sdk::{
        testutils::{Address as _, MockAuth, MockAuthInvoke},
        Address, BytesN, Env, String as SorobanString, Vec as SorobanVec,
    };
    use shared::roles::{Permission, RoleLevel};

    fn setup_test_env() -> (Env, CertificateClient<'static>, Address, Address) {
        let env = Env::default();
        let contract_id = env.register(Certificate, ());
        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);

        env.mock_all_auths();
        let client = CertificateClient::new(&env, &contract_id);
        client.initialize(&admin);

        // Grant issuer role
        client.grant_role(&issuer, RoleLevel::Instructor as u32);

        (env, client, admin, issuer)
    }

    #[test]
    fn test_comprehensive_metadata_validation_integration() {
        let (env, client, _admin, issuer) = setup_test_env();
        let student = Address::generate(&env);

        // Test 1: Valid certificate with all validation checks
        let valid_params = MintCertificateParams {
            certificate_id: BytesN::from_array(&env, &[1u8; 32]),
            course_id: SorobanString::from_str(&env, "CS-101"),
            student: student.clone(),
            title: SorobanString::from_str(&env, "Introduction to Computer Science"),
            description: SorobanString::from_str(&env, "This certificate validates successful completion of CS-101 covering fundamental programming concepts, data structures, and algorithms."),
            metadata_uri: SorobanString::from_str(&env, "https://certificates.university.edu/metadata/cs101.json"),
            expiry_date: env.ledger().timestamp() + 31536000, // 1 year from now
        };

        let result = client.try_mint_certificate(&issuer, &valid_params);
        assert!(result.is_ok(), "Valid certificate should mint successfully");

        // Verify certificate was created with correct metadata
        let certificate = client.get_certificate(&valid_params.certificate_id);
        assert!(certificate.is_some());
        let cert_metadata = certificate.unwrap();
        assert_eq!(cert_metadata.title, valid_params.title);
        assert_eq!(cert_metadata.status, CertificateStatus::Active);
    }

    #[test]
    fn test_xss_prevention_integration() {
        let (env, client, _admin, issuer) = setup_test_env();
        let student = Address::generate(&env);

        // Test various XSS attack vectors
        let xss_attacks = vec![
            ("<script>alert('xss')</script>", "Script tag injection"),
            ("Title with <img src=x onerror=alert(1)>", "Image tag with onerror"),
            ("Certificate \"javascript:alert(1)\"", "JavaScript protocol"),
            ("Title with 'onload=alert(1)'", "Event handler injection"),
            ("Title & <iframe src=javascript:alert(1)>", "Iframe injection"),
        ];

        for (malicious_content, attack_type) in xss_attacks {
            let malicious_params = MintCertificateParams {
                certificate_id: BytesN::random(&env),
                course_id: SorobanString::from_str(&env, "CS-101"),
                student: student.clone(),
                title: SorobanString::from_str(&env, malicious_content),
                description: SorobanString::from_str(&env, "Valid description for testing XSS prevention."),
                metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata.json"),
                expiry_date: env.ledger().timestamp() + 86400,
            };

            let result = client.try_mint_certificate(&issuer, &malicious_params);
            assert_eq!(
                result,
                Err(Ok(CertificateError::InvalidMetadata)),
                "XSS attack should be prevented: {}", attack_type
            );
        }
    }

    #[test]
    fn test_uri_validation_integration() {
        let (env, client, _admin, issuer) = setup_test_env();
        let student = Address::generate(&env);

        // Test valid URI schemes
        let valid_uris = vec![
            ("https://example.com/metadata.json", "HTTPS URI"),
            ("ipfs://QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG", "IPFS URI"),
            ("ar://abc123def456ghi789jkl012mno345pqr678stu901v", "Arweave URI"),
        ];

        for (uri, uri_type) in valid_uris {
            let params = MintCertificateParams {
                certificate_id: BytesN::random(&env),
                course_id: SorobanString::from_str(&env, "CS-101"),
                student: student.clone(),
                title: SorobanString::from_str(&env, "Valid Certificate Title"),
                description: SorobanString::from_str(&env, "This certificate validates completion of the course."),
                metadata_uri: SorobanString::from_str(&env, uri),
                expiry_date: env.ledger().timestamp() + 86400,
            };

            let result = client.try_mint_certificate(&issuer, &params);
            assert!(result.is_ok(), "Valid {} should be accepted", uri_type);
        }

        // Test invalid URI schemes
        let invalid_uris = vec![
            ("http://example.com/metadata.json", "HTTP not allowed"),
            ("ftp://example.com/metadata.json", "FTP not allowed"),
            ("file:///path/to/metadata.json", "File protocol not allowed"),
            ("data:application/json,{}", "Data URI not allowed"),
        ];

        for (uri, reason) in invalid_uris {
            let params = MintCertificateParams {
                certificate_id: BytesN::random(&env),
                course_id: SorobanString::from_str(&env, "CS-101"),
                student: student.clone(),
                title: SorobanString::from_str(&env, "Certificate Title"),
                description: SorobanString::from_str(&env, "This certificate validates completion of the course."),
                metadata_uri: SorobanString::from_str(&env, uri),
                expiry_date: env.ledger().timestamp() + 86400,
            };

            let result = client.try_mint_certificate(&issuer, &params);
            assert_eq!(
                result,
                Err(Ok(CertificateError::InvalidUri)),
                "Invalid URI should be rejected: {}", reason
            );
        }
    }

    #[test]
    fn test_size_limits_integration() {
        let (env, client, _admin, issuer) = setup_test_env();
        let student = Address::generate(&env);

        // Test title length limits
        let too_long_title = "A".repeat(201); // Exceeds MAX_TITLE_LENGTH (200)
        let params = MintCertificateParams {
            certificate_id: BytesN::random(&env),
            course_id: SorobanString::from_str(&env, "CS-101"),
            student: student.clone(),
            title: SorobanString::from_str(&env, &too_long_title),
            description: SorobanString::from_str(&env, "Valid description for testing size limits."),
            metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata.json"),
            expiry_date: env.ledger().timestamp() + 86400,
        };

        let result = client.try_mint_certificate(&issuer, &params);
        assert_eq!(result, Err(Ok(CertificateError::InvalidMetadata)));

        // Test description length limits
        let too_long_description = "A".repeat(1001); // Exceeds MAX_DESCRIPTION_LENGTH (1000)
        let params = MintCertificateParams {
            certificate_id: BytesN::random(&env),
            course_id: SorobanString::from_str(&env, "CS-101"),
            student: student.clone(),
            title: SorobanString::from_str(&env, "Valid Certificate Title"),
            description: SorobanString::from_str(&env, &too_long_description),
            metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata.json"),
            expiry_date: env.ledger().timestamp() + 86400,
        };

        let result = client.try_mint_certificate(&issuer, &params);
        assert_eq!(result, Err(Ok(CertificateError::InvalidMetadata)));
    }

    #[test]
    fn test_batch_certificate_validation_integration() {
        let (env, client, _admin, issuer) = setup_test_env();
        let student1 = Address::generate(&env);
        let student2 = Address::generate(&env);

        // Test valid batch minting
        let params1 = MintCertificateParams {
            certificate_id: BytesN::from_array(&env, &[1u8; 32]),
            course_id: SorobanString::from_str(&env, "CS-101"),
            student: student1,
            title: SorobanString::from_str(&env, "Computer Science Fundamentals"),
            description: SorobanString::from_str(&env, "This certificate validates completion of CS-101 course covering programming fundamentals."),
            metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata/cert1.json"),
            expiry_date: env.ledger().timestamp() + 86400,
        };

        let params2 = MintCertificateParams {
            certificate_id: BytesN::from_array(&env, &[2u8; 32]),
            course_id: SorobanString::from_str(&env, "CS-102"),
            student: student2,
            title: SorobanString::from_str(&env, "Advanced Computer Science"),
            description: SorobanString::from_str(&env, "This certificate validates completion of CS-102 course covering advanced programming concepts."),
            metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata/cert2.json"),
            expiry_date: env.ledger().timestamp() + 86400,
        };

        let batch_params = SorobanVec::from_array(&env, [params1.clone(), params2.clone()]);
        let result = client.try_mint_certificates_batch(&issuer, &batch_params);
        assert!(result.is_ok(), "Valid batch should mint successfully");

        // Verify both certificates were created
        assert!(client.get_certificate(&params1.certificate_id).is_some());
        assert!(client.get_certificate(&params2.certificate_id).is_some());

        // Test batch with duplicate IDs
        let duplicate_params = MintCertificateParams {
            certificate_id: BytesN::from_array(&env, &[1u8; 32]), // Same as params1
            course_id: SorobanString::from_str(&env, "CS-103"),
            student: student1,
            title: SorobanString::from_str(&env, "Duplicate Certificate"),
            description: SorobanString::from_str(&env, "This should fail due to duplicate ID."),
            metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata/cert3.json"),
            expiry_date: env.ledger().timestamp() + 86400,
        };

        let invalid_batch = SorobanVec::from_array(&env, [params1, duplicate_params]);
        let result = client.try_mint_certificates_batch(&issuer, &invalid_batch);
        assert_eq!(result, Err(Ok(CertificateError::CertificateAlreadyExists)));
    }

    #[test]
    fn test_uri_update_validation_integration() {
        let (env, client, _admin, issuer) = setup_test_env();
        let student = Address::generate(&env);

        // First, mint a certificate
        let cert_id = BytesN::from_array(&env, &[1u8; 32]);
        let params = MintCertificateParams {
            certificate_id: cert_id.clone(),
            course_id: SorobanString::from_str(&env, "CS-101"),
            student: student.clone(),
            title: SorobanString::from_str(&env, "Computer Science Certificate"),
            description: SorobanString::from_str(&env, "This certificate validates completion of CS-101."),
            metadata_uri: SorobanString::from_str(&env, "https://example.com/original.json"),
            expiry_date: env.ledger().timestamp() + 86400,
        };

        client.mint_certificate(&issuer, &params);

        // Test valid URI update
        let new_valid_uri = SorobanString::from_str(&env, "https://updated.example.com/metadata.json");
        let result = client.try_update_certificate_uri(&issuer, &cert_id, &new_valid_uri);
        assert!(result.is_ok(), "Valid URI update should succeed");

        // Test invalid URI update
        let invalid_uri = SorobanString::from_str(&env, "http://insecure.example.com/metadata.json");
        let result = client.try_update_certificate_uri(&issuer, &cert_id, &invalid_uri);
        assert_eq!(result, Err(Ok(CertificateError::InvalidUri)));

        // Test empty URI update
        let empty_uri = SorobanString::from_str(&env, "");
        let result = client.try_update_certificate_uri(&issuer, &cert_id, &empty_uri);
        assert_eq!(result, Err(Ok(CertificateError::InvalidUri)));
    }

    #[test]
    fn test_expiry_date_validation_integration() {
        let (env, client, _admin, issuer) = setup_test_env();
        let student = Address::generate(&env);

        // Test certificate with past expiry date
        let past_expiry = env.ledger().timestamp() - 1000; // 1000 seconds ago
        let params = MintCertificateParams {
            certificate_id: BytesN::random(&env),
            course_id: SorobanString::from_str(&env, "CS-101"),
            student: student.clone(),
            title: SorobanString::from_str(&env, "Expired Certificate"),
            description: SorobanString::from_str(&env, "This certificate should not be created with past expiry."),
            metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata.json"),
            expiry_date: past_expiry,
        };

        let result = client.try_mint_certificate(&issuer, &params);
        assert_eq!(result, Err(Ok(CertificateError::InvalidMetadata)));

        // Test certificate with too far future expiry date
        let too_far_future = env.ledger().timestamp() + (101 * 365 * 24 * 60 * 60); // 101 years
        let params = MintCertificateParams {
            certificate_id: BytesN::random(&env),
            course_id: SorobanString::from_str(&env, "CS-101"),
            student: student.clone(),
            title: SorobanString::from_str(&env, "Far Future Certificate"),
            description: SorobanString::from_str(&env, "This certificate should not be created with too far future expiry."),
            metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata.json"),
            expiry_date: too_far_future,
        };

        let result = client.try_mint_certificate(&issuer, &params);
        assert_eq!(result, Err(Ok(CertificateError::InvalidMetadata)));
    }

    #[test]
    fn test_course_id_format_validation_integration() {
        let (env, client, _admin, issuer) = setup_test_env();
        let student = Address::generate(&env);

        // Test valid course ID formats
        let valid_course_ids = vec![
            "CS-101",
            "MATH_201", 
            "ENG-101-Advanced",
            "PHY101",
            "BIO-201_Lab",
        ];

        for course_id in valid_course_ids {
            let params = MintCertificateParams {
                certificate_id: BytesN::random(&env),
                course_id: SorobanString::from_str(&env, course_id),
                student: student.clone(),
                title: SorobanString::from_str(&env, "Valid Course Certificate"),
                description: SorobanString::from_str(&env, "This certificate validates course completion."),
                metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata.json"),
                expiry_date: env.ledger().timestamp() + 86400,
            };

            let result = client.try_mint_certificate(&issuer, &params);
            assert!(result.is_ok(), "Valid course ID '{}' should be accepted", course_id);
        }

        // Test invalid course ID formats
        let invalid_course_ids = vec![
            "CS@101",     // Invalid character @
            "CS 101",     // Space not allowed
            "-CS101",     // Starts with separator
            "CS101-",     // Ends with separator
            "CS..101",    // Invalid character .
        ];

        for course_id in invalid_course_ids {
            let params = MintCertificateParams {
                certificate_id: BytesN::random(&env),
                course_id: SorobanString::from_str(&env, course_id),
                student: student.clone(),
                title: SorobanString::from_str(&env, "Invalid Course Certificate"),
                description: SorobanString::from_str(&env, "This certificate should be rejected."),
                metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata.json"),
                expiry_date: env.ledger().timestamp() + 86400,
            };

            let result = client.try_mint_certificate(&issuer, &params);
            assert_eq!(
                result,
                Err(Ok(CertificateError::InvalidMetadata)),
                "Invalid course ID '{}' should be rejected", course_id
            );
        }
    }

    #[test]
    fn test_content_quality_validation_integration() {
        let (env, client, _admin, issuer) = setup_test_env();
        let student = Address::generate(&env);

        // Test excessive special characters
        let spammy_title = "!!!@@@###$$$%%%^^^&&&***((()))";
        let params = MintCertificateParams {
            certificate_id: BytesN::random(&env),
            course_id: SorobanString::from_str(&env, "CS-101"),
            student: student.clone(),
            title: SorobanString::from_str(&env, spammy_title),
            description: SorobanString::from_str(&env, "Valid description for testing content quality."),
            metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata.json"),
            expiry_date: env.ledger().timestamp() + 86400,
        };

        let result = client.try_mint_certificate(&issuer, &params);
        assert_eq!(result, Err(Ok(CertificateError::InvalidMetadata)));

        // Test excessive character repetition
        let repeated_title = "Aaaaaaaaa Certificate Title"; // 8 consecutive 'a's (exceeds limit of 5)
        let params = MintCertificateParams {
            certificate_id: BytesN::random(&env),
            course_id: SorobanString::from_str(&env, "CS-101"),
            student: student.clone(),
            title: SorobanString::from_str(&env, repeated_title),
            description: SorobanString::from_str(&env, "Valid description for testing repetition."),
            metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata.json"),
            expiry_date: env.ledger().timestamp() + 86400,
        };

        let result = client.try_mint_certificate(&issuer, &params);
        assert_eq!(result, Err(Ok(CertificateError::InvalidMetadata)));
    }
}
