use crate::validation::{MetadataValidator, ValidationConfig};
use crate::types::MintCertificateParams;
use crate::errors::CertificateError;
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env, String as SorobanString, Vec as SorobanVec,
};

#[test]
fn test_validate_mint_params_success() {
    let env = Env::default();
    let student = Address::generate(&env);
    let certificate_id = BytesN::from_array(&env, &[1u8; 32]);
    
    let params = MintCertificateParams {
        certificate_id,
        course_id: SorobanString::from_str(&env, "CS-101"),
        student,
        title: SorobanString::from_str(&env, "Introduction to Computer Science"),
        description: SorobanString::from_str(&env, "This certificate validates completion of CS-101 course covering fundamental programming concepts."),
        metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata/cert1.json"),
        expiry_date: env.ledger().timestamp() + 86400, // 1 day from now
    };
    
    assert!(MetadataValidator::validate_mint_params(&env, &params).is_ok());
}

#[test]
fn test_validate_title_empty() {
    let env = Env::default();
    let empty_title = SorobanString::from_str(&env, "");
    assert_eq!(
        MetadataValidator::validate_title(&empty_title),
        Err(CertificateError::InvalidMetadata)
    );
}

#[test]
fn test_validate_title_too_short() {
    let env = Env::default();
    let short_title = SorobanString::from_str(&env, "AB");
    assert_eq!(
        MetadataValidator::validate_title(&short_title),
        Err(CertificateError::InvalidMetadata)
    );
}

#[test]
fn test_validate_title_too_long() {
    let env = Env::default();
    let long_title = SorobanString::from_str(&env, &"A".repeat(ValidationConfig::MAX_TITLE_LENGTH as usize + 1));
    assert_eq!(
        MetadataValidator::validate_title(&long_title),
        Err(CertificateError::InvalidMetadata)
    );
}

#[test]
fn test_validate_title_xss_prevention() {
    let env = Env::default();
    
    // Test various XSS attack vectors
    let xss_vectors = vec![
        "<script>alert('xss')</script>",
        "Title with <img src=x onerror=alert(1)>",
        "Title with \"javascript:alert(1)\"",
        "Title with 'onload=alert(1)'",
        "Title with &lt;script&gt;",
    ];
    
    for xss in xss_vectors {
        let malicious_title = SorobanString::from_str(&env, xss);
        assert_eq!(
            MetadataValidator::validate_title(&malicious_title),
            Err(CertificateError::InvalidMetadata),
            "Failed to detect XSS in: {}", xss
        );
    }
}

#[test]
fn test_validate_description_success() {
    let env = Env::default();
    let valid_description = SorobanString::from_str(&env, "This is a valid certificate description that meets all requirements.");
    assert!(MetadataValidator::validate_description(&valid_description).is_ok());
}

#[test]
fn test_validate_description_too_short() {
    let env = Env::default();
    let short_desc = SorobanString::from_str(&env, "Short");
    assert_eq!(
        MetadataValidator::validate_description(&short_desc),
        Err(CertificateError::InvalidMetadata)
    );
}

#[test]
fn test_validate_description_too_long() {
    let env = Env::default();
    let long_desc = SorobanString::from_str(&env, &"A".repeat(ValidationConfig::MAX_DESCRIPTION_LENGTH as usize + 1));
    assert_eq!(
        MetadataValidator::validate_description(&long_desc),
        Err(CertificateError::InvalidMetadata)
    );
}

#[test]
fn test_validate_course_id_success() {
    let env = Env::default();
    
    let valid_course_ids = vec![
        "CS-101",
        "MATH_201",
        "ENG-101-Advanced",
        "PHY101",
        "BIO-201_Lab",
    ];
    
    for course_id in valid_course_ids {
        let course_id_str = SorobanString::from_str(&env, course_id);
        assert!(
            MetadataValidator::validate_course_id(&course_id_str).is_ok(),
            "Failed to validate valid course ID: {}", course_id
        );
    }
}

#[test]
fn test_validate_course_id_invalid_format() {
    let env = Env::default();
    
    let invalid_course_ids = vec![
        "CS@101",        // Invalid character @
        "CS 101",        // Space not allowed
        "CS#101",        // Invalid character #
        "-CS101",        // Starts with separator
        "CS101-",        // Ends with separator
        "_CS101",        // Starts with separator
        "CS101_",        // Ends with separator
        "CS..101",       // Invalid character .
    ];
    
    for course_id in invalid_course_ids {
        let course_id_str = SorobanString::from_str(&env, course_id);
        assert_eq!(
            MetadataValidator::validate_course_id(&course_id_str),
            Err(CertificateError::InvalidMetadata),
            "Failed to reject invalid course ID: {}", course_id
        );
    }
}

#[test]
fn test_validate_uri_https_success() {
    let env = Env::default();
    
    let valid_uris = vec![
        "https://example.com/metadata.json",
        "https://api.example.com/v1/certificates/metadata/123",
        "https://storage.googleapis.com/bucket/metadata.json",
        "https://cdn.example.com/assets/cert-metadata.json",
    ];
    
    for uri in valid_uris {
        let uri_str = SorobanString::from_str(&env, uri);
        assert!(
            MetadataValidator::validate_metadata_uri(&uri_str).is_ok(),
            "Failed to validate valid HTTPS URI: {}", uri
        );
    }
}

#[test]
fn test_validate_uri_ipfs_success() {
    let env = Env::default();
    
    let valid_ipfs_uris = vec![
        "ipfs://QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
        "ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    ];
    
    for uri in valid_ipfs_uris {
        let uri_str = SorobanString::from_str(&env, uri);
        assert!(
            MetadataValidator::validate_metadata_uri(&uri_str).is_ok(),
            "Failed to validate valid IPFS URI: {}", uri
        );
    }
}

#[test]
fn test_validate_uri_arweave_success() {
    let env = Env::default();
    
    let valid_ar_uris = vec![
        "ar://abc123def456ghi789jkl012mno345pqr678stu901v",
        "ar://XYZ789abc012DEF345ghi678JKL901mno234PQR567s",
    ];
    
    for uri in valid_ar_uris {
        let uri_str = SorobanString::from_str(&env, uri);
        assert!(
            MetadataValidator::validate_metadata_uri(&uri_str).is_ok(),
            "Failed to validate valid Arweave URI: {}", uri
        );
    }
}

#[test]
fn test_validate_uri_invalid_scheme() {
    let env = Env::default();
    
    let invalid_uris = vec![
        "http://example.com/metadata.json",  // HTTP not allowed
        "ftp://example.com/metadata.json",   // FTP not allowed
        "file:///path/to/metadata.json",     // File not allowed
        "data:application/json,{}",          // Data URI not allowed
    ];
    
    for uri in invalid_uris {
        let uri_str = SorobanString::from_str(&env, uri);
        assert_eq!(
            MetadataValidator::validate_metadata_uri(&uri_str),
            Err(CertificateError::InvalidUri),
            "Failed to reject invalid URI scheme: {}", uri
        );
    }
}

#[test]
fn test_validate_uri_malformed() {
    let env = Env::default();
    
    let malformed_uris = vec![
        "https://",                          // Incomplete
        "https:// example.com",              // Space in domain
        "https://example.com///path",        // Triple slash
        "https://.example.com",              // Domain starts with dot
        "https://example.com.",              // Domain ends with dot
        "ipfs://short",                      // IPFS hash too short
        "ar://short",                        // Arweave TX ID wrong length
    ];
    
    for uri in malformed_uris {
        let uri_str = SorobanString::from_str(&env, uri);
        assert_eq!(
            MetadataValidator::validate_metadata_uri(&uri_str),
            Err(CertificateError::InvalidUri),
            "Failed to reject malformed URI: {}", uri
        );
    }
}

#[test]
fn test_validate_expiry_date_future() {
    let env = Env::default();
    let future_date = env.ledger().timestamp() + 86400; // 1 day in future
    assert!(MetadataValidator::validate_expiry_date(&env, future_date).is_ok());
}

#[test]
fn test_validate_expiry_date_past() {
    let env = Env::default();
    let past_date = env.ledger().timestamp() - 1; // 1 second in past
    assert_eq!(
        MetadataValidator::validate_expiry_date(&env, past_date),
        Err(CertificateError::InvalidMetadata)
    );
}

#[test]
fn test_validate_expiry_date_too_far_future() {
    let env = Env::default();
    let too_far_future = env.ledger().timestamp() + (101 * 365 * 24 * 60 * 60); // 101 years
    assert_eq!(
        MetadataValidator::validate_expiry_date(&env, too_far_future),
        Err(CertificateError::InvalidMetadata)
    );
}

#[test]
fn test_validate_certificate_id_zero() {
    let env = Env::default();
    let zero_id = BytesN::from_array(&env, &[0u8; 32]);
    assert_eq!(
        MetadataValidator::validate_certificate_id(&zero_id),
        Err(CertificateError::InvalidMetadata)
    );
}

#[test]
fn test_validate_certificate_id_valid() {
    let env = Env::default();
    let valid_id = BytesN::from_array(&env, &[1u8; 32]);
    assert!(MetadataValidator::validate_certificate_id(&valid_id).is_ok());
}

#[test]
fn test_validate_text_quality_excessive_special_chars() {
    let env = Env::default();
    let bad_text = SorobanString::from_str(&env, "!!!@@@###$$$%%%^^^&&&***");
    assert_eq!(
        MetadataValidator::validate_title(&bad_text),
        Err(CertificateError::InvalidMetadata)
    );
}

#[test]
fn test_validate_text_quality_excessive_repetition() {
    let env = Env::default();
    let repeated_text = SorobanString::from_str(&env, "Aaaaaaaaa Certificate Title"); // 8 consecutive 'a's
    assert_eq!(
        MetadataValidator::validate_title(&repeated_text),
        Err(CertificateError::InvalidMetadata)
    );
}

#[test]
fn test_validate_text_quality_whitespace_only() {
    let env = Env::default();
    let whitespace_text = SorobanString::from_str(&env, "   \t\n   ");
    assert_eq!(
        MetadataValidator::validate_title(&whitespace_text),
        Err(CertificateError::InvalidMetadata)
    );
}

#[test]
fn test_sanitize_text() {
    let input = "Title with <script> and \"quotes\" and 'apostrophes'";
    let expected = "Title with  and  and ";
    let result = MetadataValidator::sanitize_text(input);
    assert_eq!(result, expected);
}

#[test]
fn test_validate_batch_params_success() {
    let env = Env::default();
    let student1 = Address::generate(&env);
    let student2 = Address::generate(&env);
    
    let params1 = MintCertificateParams {
        certificate_id: BytesN::from_array(&env, &[1u8; 32]),
        course_id: SorobanString::from_str(&env, "CS-101"),
        student: student1,
        title: SorobanString::from_str(&env, "Computer Science Fundamentals"),
        description: SorobanString::from_str(&env, "This certificate validates completion of CS-101 course."),
        metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata/cert1.json"),
        expiry_date: env.ledger().timestamp() + 86400,
    };
    
    let params2 = MintCertificateParams {
        certificate_id: BytesN::from_array(&env, &[2u8; 32]),
        course_id: SorobanString::from_str(&env, "CS-102"),
        student: student2,
        title: SorobanString::from_str(&env, "Advanced Computer Science"),
        description: SorobanString::from_str(&env, "This certificate validates completion of CS-102 course."),
        metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata/cert2.json"),
        expiry_date: env.ledger().timestamp() + 86400,
    };
    
    let params_list = vec![params1, params2];
    assert!(MetadataValidator::validate_batch_params(&env, &params_list).is_ok());
}

#[test]
fn test_validate_batch_params_empty() {
    let env = Env::default();
    let params_list: Vec<MintCertificateParams> = vec![];
    assert_eq!(
        MetadataValidator::validate_batch_params(&env, &params_list),
        Err(CertificateError::InvalidInput)
    );
}

#[test]
fn test_validate_batch_params_too_large() {
    let env = Env::default();
    let student = Address::generate(&env);
    
    // Create 101 certificates (exceeds limit of 100)
    let mut params_list = Vec::new();
    for i in 0..101 {
        let mut id_bytes = [0u8; 32];
        id_bytes[0] = i as u8;
        
        let params = MintCertificateParams {
            certificate_id: BytesN::from_array(&env, &id_bytes),
            course_id: SorobanString::from_str(&env, &format!("CS-{}", i)),
            student: student.clone(),
            title: SorobanString::from_str(&env, &format!("Certificate {}", i)),
            description: SorobanString::from_str(&env, &format!("This is certificate number {}", i)),
            metadata_uri: SorobanString::from_str(&env, &format!("https://example.com/cert{}.json", i)),
            expiry_date: env.ledger().timestamp() + 86400,
        };
        params_list.push(params);
    }
    
    assert_eq!(
        MetadataValidator::validate_batch_params(&env, &params_list),
        Err(CertificateError::InvalidInput)
    );
}

#[test]
fn test_validate_batch_params_duplicate_ids() {
    let env = Env::default();
    let student = Address::generate(&env);
    let duplicate_id = BytesN::from_array(&env, &[1u8; 32]);
    
    let params1 = MintCertificateParams {
        certificate_id: duplicate_id.clone(),
        course_id: SorobanString::from_str(&env, "CS-101"),
        student: student.clone(),
        title: SorobanString::from_str(&env, "Certificate 1"),
        description: SorobanString::from_str(&env, "This is the first certificate."),
        metadata_uri: SorobanString::from_str(&env, "https://example.com/cert1.json"),
        expiry_date: env.ledger().timestamp() + 86400,
    };
    
    let params2 = MintCertificateParams {
        certificate_id: duplicate_id, // Same ID as params1
        course_id: SorobanString::from_str(&env, "CS-102"),
        student: student,
        title: SorobanString::from_str(&env, "Certificate 2"),
        description: SorobanString::from_str(&env, "This is the second certificate."),
        metadata_uri: SorobanString::from_str(&env, "https://example.com/cert2.json"),
        expiry_date: env.ledger().timestamp() + 86400,
    };
    
    let params_list = vec![params1, params2];
    assert_eq!(
        MetadataValidator::validate_batch_params(&env, &params_list),
        Err(CertificateError::CertificateAlreadyExists)
    );
}

#[test]
fn test_validate_batch_params_invalid_individual() {
    let env = Env::default();
    let student = Address::generate(&env);
    
    let valid_params = MintCertificateParams {
        certificate_id: BytesN::from_array(&env, &[1u8; 32]),
        course_id: SorobanString::from_str(&env, "CS-101"),
        student: student.clone(),
        title: SorobanString::from_str(&env, "Valid Certificate"),
        description: SorobanString::from_str(&env, "This is a valid certificate."),
        metadata_uri: SorobanString::from_str(&env, "https://example.com/cert1.json"),
        expiry_date: env.ledger().timestamp() + 86400,
    };
    
    let invalid_params = MintCertificateParams {
        certificate_id: BytesN::from_array(&env, &[2u8; 32]),
        course_id: SorobanString::from_str(&env, "CS-102"),
        student: student,
        title: SorobanString::from_str(&env, "AB"), // Too short
        description: SorobanString::from_str(&env, "This is an invalid certificate."),
        metadata_uri: SorobanString::from_str(&env, "https://example.com/cert2.json"),
        expiry_date: env.ledger().timestamp() + 86400,
    };
    
    let params_list = vec![valid_params, invalid_params];
    assert_eq!(
        MetadataValidator::validate_batch_params(&env, &params_list),
        Err(CertificateError::InvalidMetadata)
    );
}

// Edge case tests
#[test]
fn test_validate_uri_edge_cases() {
    let env = Env::default();
    
    // Test minimum and maximum lengths
    let min_valid_uri = SorobanString::from_str(&env, "https://a.b"); // Exactly at minimum
    assert!(MetadataValidator::validate_metadata_uri(&min_valid_uri).is_ok());
    
    let max_valid_uri = SorobanString::from_str(&env, &format!("https://example.com/{}", "a".repeat(ValidationConfig::MAX_URI_LENGTH as usize - 19))); // 19 chars for "https://example.com/"
    assert!(MetadataValidator::validate_metadata_uri(&max_valid_uri).is_ok());
}

#[test]
fn test_validate_forbidden_characters_comprehensive() {
    let env = Env::default();
    
    // Test all forbidden characters
    for &forbidden_char in ValidationConfig::FORBIDDEN_CHARS {
        let text_with_forbidden = format!("Valid text{}", forbidden_char);
        let soroban_text = SorobanString::from_str(&env, &text_with_forbidden);
        
        assert_eq!(
            MetadataValidator::validate_title(&soroban_text),
            Err(CertificateError::InvalidMetadata),
            "Failed to detect forbidden character: {:?}", forbidden_char
        );
    }
}
