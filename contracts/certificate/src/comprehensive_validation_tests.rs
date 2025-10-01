use soroban_sdk::{testutils::Address as _, Address, Env, String as SorobanString, BytesN};
use shared::validation::{CoreValidator, ValidationConfig, ValidationError};

/// Integration tests for comprehensive metadata validation
/// 
/// These tests ensure the validation system works end-to-end and covers
/// all critical edge cases and attack vectors.

#[cfg(test)]
mod integration_validation_tests {
    use super::*;
    
    /// Test the complete validation flow for valid certificate data
    #[test]
    fn test_end_to_end_validation_success() {
        let env = Env::default();
        
        // Test valid HTTPS URI
        let https_uri = "https://university.edu/certificates/metadata/cs101.json";
        assert!(CoreValidator::validate_uri(https_uri).is_ok());
        
        // Test valid IPFS URI
        let ipfs_uri = "ipfs://QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG";
        assert!(CoreValidator::validate_uri(ipfs_uri).is_ok());
        
        // Test valid Arweave URI
        let arweave_uri = "ar://ABC123abc456XYZ789xyz012DEF345def678GHI";
        assert!(CoreValidator::validate_uri(arweave_uri).is_ok());
        
        // Test all text fields
        let title = "Advanced Computer Science Certificate";
        let description = "This certificate validates completion of advanced computer science coursework including algorithms, data structures, and software engineering principles.";
        let course_id = "CS-401_Advanced";
        
        assert!(CoreValidator::validate_text_field(title, "title", ValidationConfig::MIN_TITLE_LENGTH, ValidationConfig::MAX_TITLE_LENGTH).is_ok());
        assert!(CoreValidator::validate_text_field(description, "description", ValidationConfig::MIN_DESCRIPTION_LENGTH, ValidationConfig::MAX_DESCRIPTION_LENGTH).is_ok());
        assert!(CoreValidator::validate_course_id(course_id).is_ok());
        
        // Test expiry date
        let future_date = env.ledger().timestamp() + 31536000; // 1 year
        assert!(CoreValidator::validate_expiry_date(&env, future_date).is_ok());
        
        // Test certificate ID
        let cert_id = BytesN::from_array(&env, &[1u8; 32]);
        assert!(CoreValidator::validate_certificate_id(&cert_id).is_ok());
    }
    
    /// Test XSS prevention across all input fields
    #[test]
    fn test_xss_prevention_comprehensive() {
        // Common XSS attack vectors
        let xss_vectors = vec![
            "<script>alert('xss')</script>",
            "javascript:alert('xss')",
            "<img src=x onerror=alert('xss')>",
            "<svg onload=alert('xss')>",
            "<iframe src=javascript:alert('xss')>",
            "';alert('xss');//",
            "\"><script>alert('xss')</script>",
            "<script>document.cookie</script>",
            "<body onload=alert('xss')>",
            "<div onclick=alert('xss')>click me</div>",
        ];
        
        for vector in xss_vectors {
            // Test in title field
            let result = CoreValidator::validate_text_field(
                vector, 
                "title", 
                ValidationConfig::MIN_TITLE_LENGTH, 
                ValidationConfig::MAX_TITLE_LENGTH
            );
            assert!(result.is_err(), "XSS vector should be rejected in title: {}", vector);
            
            // Test in description field  
            let result = CoreValidator::validate_text_field(
                vector, 
                "description", 
                ValidationConfig::MIN_DESCRIPTION_LENGTH, 
                ValidationConfig::MAX_DESCRIPTION_LENGTH
            );
            assert!(result.is_err(), "XSS vector should be rejected in description: {}", vector);
            
            // Test in course ID (if it passes length requirements)
            if vector.len() >= ValidationConfig::MIN_COURSE_ID_LENGTH as usize &&
               vector.len() <= ValidationConfig::MAX_COURSE_ID_LENGTH as usize {
                let result = CoreValidator::validate_course_id(vector);
                assert!(result.is_err(), "XSS vector should be rejected in course_id: {}", vector);
            }
        }
    }
    
    /// Test boundary conditions for all length limits
    #[test]
    fn test_boundary_conditions_comprehensive() {
        // Test minimum lengths - 1 (should fail)
        assert!(CoreValidator::validate_text_field(
            &"A".repeat((ValidationConfig::MIN_TITLE_LENGTH - 1) as usize),
            "title",
            ValidationConfig::MIN_TITLE_LENGTH,
            ValidationConfig::MAX_TITLE_LENGTH
        ).is_err());
        
        assert!(CoreValidator::validate_text_field(
            &"A".repeat((ValidationConfig::MIN_DESCRIPTION_LENGTH - 1) as usize),
            "description",
            ValidationConfig::MIN_DESCRIPTION_LENGTH,
            ValidationConfig::MAX_DESCRIPTION_LENGTH
        ).is_err());
        
        assert!(CoreValidator::validate_course_id(
            &"A".repeat((ValidationConfig::MIN_COURSE_ID_LENGTH - 1) as usize)
        ).is_err());
        
        // Test minimum lengths (should pass)
        assert!(CoreValidator::validate_text_field(
            &"A".repeat(ValidationConfig::MIN_TITLE_LENGTH as usize),
            "title",
            ValidationConfig::MIN_TITLE_LENGTH,
            ValidationConfig::MAX_TITLE_LENGTH
        ).is_ok());
        
        assert!(CoreValidator::validate_text_field(
            &"A".repeat(ValidationConfig::MIN_DESCRIPTION_LENGTH as usize),
            "description",
            ValidationConfig::MIN_DESCRIPTION_LENGTH,
            ValidationConfig::MAX_DESCRIPTION_LENGTH
        ).is_ok());
        
        assert!(CoreValidator::validate_course_id(
            &"A".repeat(ValidationConfig::MIN_COURSE_ID_LENGTH as usize)
        ).is_ok());
        
        // Test maximum lengths (should pass)
        assert!(CoreValidator::validate_text_field(
            &"A".repeat(ValidationConfig::MAX_TITLE_LENGTH as usize),
            "title",
            ValidationConfig::MIN_TITLE_LENGTH,
            ValidationConfig::MAX_TITLE_LENGTH
        ).is_ok());
        
        assert!(CoreValidator::validate_text_field(
            &"A".repeat(ValidationConfig::MAX_DESCRIPTION_LENGTH as usize),
            "description",
            ValidationConfig::MIN_DESCRIPTION_LENGTH,
            ValidationConfig::MAX_DESCRIPTION_LENGTH
        ).is_ok());
        
        assert!(CoreValidator::validate_course_id(
            &"A".repeat(ValidationConfig::MAX_COURSE_ID_LENGTH as usize)
        ).is_ok());
        
        // Test maximum lengths + 1 (should fail)
        assert!(CoreValidator::validate_text_field(
            &"A".repeat((ValidationConfig::MAX_TITLE_LENGTH + 1) as usize),
            "title",
            ValidationConfig::MIN_TITLE_LENGTH,
            ValidationConfig::MAX_TITLE_LENGTH
        ).is_err());
        
        assert!(CoreValidator::validate_text_field(
            &"A".repeat((ValidationConfig::MAX_DESCRIPTION_LENGTH + 1) as usize),
            "description",
            ValidationConfig::MIN_DESCRIPTION_LENGTH,
            ValidationConfig::MAX_DESCRIPTION_LENGTH
        ).is_err());
        
        assert!(CoreValidator::validate_course_id(
            &"A".repeat((ValidationConfig::MAX_COURSE_ID_LENGTH + 1) as usize)
        ).is_err());
    }
    
    /// Test URI validation with various edge cases
    #[test]
    fn test_uri_validation_edge_cases() {
        // Valid URIs
        let valid_uris = vec![
            "https://example.com/path/to/metadata.json",
            "https://sub.example.com/metadata.json",
            "https://example.com:8080/metadata.json",
            "https://example.com/path?query=value&param=test",
            "https://example.com/path#fragment",
            "ipfs://QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
            "ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
            "ar://ABC123abc456XYZ789xyz012DEF345def678GHI",
            "ar://1234567890abcdefghijklmnopqrstuvwxyzABCDEF",
        ];
        
        for uri in valid_uris {
            assert!(CoreValidator::validate_uri(uri).is_ok(), "Valid URI should pass: {}", uri);
        }
        
        // Invalid URIs
        let invalid_uris = vec![
            "http://example.com/metadata.json", // HTTP not allowed
            "ftp://example.com/file.json", // FTP not allowed
            "file:///local/path/file.json", // File protocol not allowed
            "https://", // Too short
            "https:// example.com", // Space in domain
            "https://example .com", // Space in domain
            "https://example.com///path", // Triple slash
            "ipfs://", // Too short
            "ipfs://invalid hash", // Invalid hash format
            "ipfs://QmTooShort", // Hash too short
            "ar://", // Too short
            "ar://TooShort", // Transaction ID too short
            "ar://TooLongTransactionIdThatExceedsThe43CharLimit", // Transaction ID too long
            "ar://Invalid@Chars!", // Invalid characters in transaction ID
        ];
        
        for uri in invalid_uris {
            assert!(CoreValidator::validate_uri(uri).is_err(), "Invalid URI should fail: {}", uri);
        }
    }
    
    /// Test course ID format validation with various patterns
    #[test]
    fn test_course_id_format_validation() {
        // Valid course IDs
        let valid_course_ids = vec![
            "CS101",
            "CS-101",
            "CS_101",
            "CS-101_Advanced",
            "MATH-201_Calculus",
            "PHYS301",
            "BIO-101_Intro",
            "CHEM_301_Organic",
            "ENG101_Writing",
            "HIST-202_Modern",
        ];
        
        for course_id in valid_course_ids {
            assert!(CoreValidator::validate_course_id(course_id).is_ok(), "Valid course ID should pass: {}", course_id);
        }
        
        // Invalid course IDs
        let invalid_course_ids = vec![
            "-CS101", // Starts with separator
            "CS101-", // Ends with separator
            "_CS101", // Starts with underscore
            "CS101_", // Ends with underscore
            "CS@101", // Invalid character
            "CS#101", // Invalid character
            "CS 101", // Space not allowed
            "CS.101", // Dot not allowed
            "CS/101", // Slash not allowed
            "CS\\101", // Backslash not allowed
            "CS+101", // Plus not allowed
            "CS=101", // Equals not allowed
            "CS!101", // Exclamation not allowed
            "CS?101", // Question mark not allowed
            "CS%101", // Percent not allowed
            "CS&101", // Ampersand not allowed
            "CS*101", // Asterisk not allowed
            "CS(101)", // Parentheses not allowed
            "CS[101]", // Brackets not allowed
            "CS{101}", // Braces not allowed
            "CS<101>", // Angle brackets not allowed
        ];
        
        for course_id in invalid_course_ids {
            assert!(CoreValidator::validate_course_id(course_id).is_err(), "Invalid course ID should fail: {}", course_id);
        }
    }
    
    /// Test content quality validation
    #[test]
    fn test_content_quality_validation() {
        // Valid content
        let valid_content = vec![
            "This is a normal certificate description.",
            "Certificate for Advanced Programming in Rust and Blockchain Development.",
            "Achievement award for completing the full-stack web development course.",
            "Certification of excellence in machine learning and artificial intelligence.",
            "Recognition for outstanding performance in data science fundamentals.",
        ];
        
        for content in valid_content {
            assert!(CoreValidator::validate_text_quality(content, "test").is_ok(), "Valid content should pass: {}", content);
        }
        
        // Invalid content - too many special characters
        let invalid_content = vec![
            "!@#$%^&*()_+{}|:<>?", // All special characters
            "Text with !@#$%^&*()_+ too many special chars", // High ratio of special chars
            "!!!!!!!!!!!!!!!!!!", // Repeated special characters
            "AAAAAAAAAAAAAAAAAAAAAA", // Too many repeated characters
            "   ", // Only whitespace
            "", // Empty string
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", // Too many repeated chars
        ];
        
        for content in invalid_content {
            assert!(CoreValidator::validate_text_quality(content, "test").is_err(), "Invalid content should fail: {}", content);
        }
    }
    
    /// Test sanitization functionality
    #[test]
    fn test_sanitization_functionality() {
        let test_cases = vec![
            ("Clean text", "Clean text"),
            ("Text with <script>", "Text with "),
            ("Text with 'quotes'", "Text with "),
            ("Text with \"double quotes\"", "Text with "),
            ("Text with & ampersand", "Text with  ampersand"),
            ("Text with \x00 null char", "Text with  null char"),
            ("Text with \x1F control char", "Text with  control char"),
            ("Normal text with numbers 123", "Normal text with numbers 123"),
            ("Text with\ttabs\nand\rlinebreaks", "Text withtabsandlinebreaks"),
        ];
        
        for (input, expected_pattern) in test_cases {
            let sanitized = CoreValidator::sanitize_text(input);
            if expected_pattern.contains("Text with ") && !expected_pattern.contains("<script>") {
                assert!(!sanitized.contains('<'), "Sanitized text should not contain < : {}", sanitized);
                assert!(!sanitized.contains('>'), "Sanitized text should not contain > : {}", sanitized);
            }
            if !expected_pattern.contains('\'') {
                assert!(!sanitized.contains('\''), "Sanitized text should not contain ' : {}", sanitized);
            }
            if !expected_pattern.contains('&') {
                assert!(!sanitized.contains('&'), "Sanitized text should not contain & : {}", sanitized);
            }
        }
    }
    
    /// Test certificate ID validation edge cases
    #[test]
    fn test_certificate_id_validation() {
        let env = Env::default();
        
        // Valid certificate IDs
        let valid_ids = vec![
            [1u8; 32],
            [255u8; 32],
            {
                let mut id = [0u8; 32];
                id[0] = 1;
                id
            },
            {
                let mut id = [0u8; 32];
                id[31] = 1;
                id
            },
        ];
        
        for id_bytes in valid_ids {
            let cert_id = BytesN::from_array(&env, &id_bytes);
            assert!(CoreValidator::validate_certificate_id(&cert_id).is_ok(), "Valid certificate ID should pass");
        }
        
        // Invalid certificate ID (all zeros)
        let zero_id = BytesN::from_array(&env, &[0u8; 32]);
        assert!(CoreValidator::validate_certificate_id(&zero_id).is_err(), "Zero certificate ID should fail");
    }
    
    /// Test expiry date validation with various edge cases
    #[test]
    fn test_expiry_date_validation_comprehensive() {
        let env = Env::default();
        let current_time = env.ledger().timestamp();
        
        // Valid expiry dates
        let valid_dates = vec![
            0, // Non-expiring certificate
            current_time + 1, // 1 second in future
            current_time + 86400, // 1 day in future
            current_time + 31536000, // 1 year in future
            current_time + (10 * 365 * 24 * 60 * 60), // 10 years in future
            current_time + (50 * 365 * 24 * 60 * 60), // 50 years in future
            current_time + ValidationConfig::MAX_FUTURE_EXPIRY, // Maximum allowed
        ];
        
        for date in valid_dates {
            assert!(CoreValidator::validate_expiry_date(&env, date).is_ok(), "Valid expiry date should pass: {}", date);
        }
        
        // Invalid expiry dates
        let invalid_dates = vec![
            current_time, // Current time (not future)
            current_time - 1, // 1 second in past
            current_time - 86400, // 1 day in past
            current_time - 31536000, // 1 year in past
            current_time + ValidationConfig::MAX_FUTURE_EXPIRY + 1, // Too far in future
            current_time + (200 * 365 * 24 * 60 * 60), // 200 years in future
        ];
        
        for date in invalid_dates {
            assert!(CoreValidator::validate_expiry_date(&env, date).is_err(), "Invalid expiry date should fail: {}", date);
        }
    }
    
    /// Test error message generation and types
    #[test]
    fn test_validation_error_messages() {
        // Test string length errors
        let result = CoreValidator::validate_string_length("AB", "title", 3, 20);
        assert!(matches!(result, Err(ValidationError::FieldTooShort { .. })));
        
        let result = CoreValidator::validate_string_length(&"A".repeat(21), "title", 3, 20);
        assert!(matches!(result, Err(ValidationError::FieldTooLong { .. })));
        
        // Test forbidden character errors
        let result = CoreValidator::validate_no_forbidden_chars("<script>", "title");
        assert!(matches!(result, Err(ValidationError::InvalidCharacters { .. })));
        
        // Test URI errors
        let result = CoreValidator::validate_uri_scheme("ftp://example.com");
        assert!(matches!(result, Err(ValidationError::InvalidUri { .. })));
        
        // Test format errors
        let result = CoreValidator::validate_course_id_format("CS@101");
        assert!(matches!(result, Err(ValidationError::InvalidFormat { .. })));
        
        // Test date errors
        let env = Env::default();
        let result = CoreValidator::validate_expiry_date(&env, env.ledger().timestamp() - 1);
        assert!(matches!(result, Err(ValidationError::InvalidDate { .. })));
        
        // Test content quality errors
        let result = CoreValidator::validate_text_quality("   ", "title");
        assert!(matches!(result, Err(ValidationError::EmptyField { .. })));
        
        let result = CoreValidator::validate_text_quality("!@#$%^&*()", "title");
        assert!(matches!(result, Err(ValidationError::ContentQuality { .. })));
    }
    
    /// Test performance with large inputs
    #[test]
    fn test_performance_with_large_inputs() {
        // Test with maximum allowed lengths
        let max_title = "A".repeat(ValidationConfig::MAX_TITLE_LENGTH as usize);
        let max_description = "B".repeat(ValidationConfig::MAX_DESCRIPTION_LENGTH as usize);
        let max_course_id = "C".repeat(ValidationConfig::MAX_COURSE_ID_LENGTH as usize);
        let max_uri = format!("https://example.com/{}", "D".repeat(ValidationConfig::MAX_URI_LENGTH as usize - 20));
        
        // These should complete quickly and pass validation
        assert!(CoreValidator::validate_text_field(&max_title, "title", ValidationConfig::MIN_TITLE_LENGTH, ValidationConfig::MAX_TITLE_LENGTH).is_ok());
        assert!(CoreValidator::validate_text_field(&max_description, "description", ValidationConfig::MIN_DESCRIPTION_LENGTH, ValidationConfig::MAX_DESCRIPTION_LENGTH).is_ok());
        assert!(CoreValidator::validate_course_id(&max_course_id).is_ok());
        assert!(CoreValidator::validate_uri(&max_uri).is_ok());
    }
}