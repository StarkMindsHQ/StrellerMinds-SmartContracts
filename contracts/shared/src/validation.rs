use soroban_sdk::{BytesN, Env};

/// Configuration constants for metadata validation that can be reused across contracts
pub struct ValidationConfig;

impl ValidationConfig {
    // Size limits (in bytes)
    pub const MAX_TITLE_LENGTH: u32 = 200;
    pub const MAX_DESCRIPTION_LENGTH: u32 = 1000;
    pub const MAX_COURSE_ID_LENGTH: u32 = 100;
    pub const MAX_URI_LENGTH: u32 = 500;
    pub const MAX_BATCH_SIZE: u32 = 100;

    // Minimum lengths
    pub const MIN_TITLE_LENGTH: u32 = 3;
    pub const MIN_DESCRIPTION_LENGTH: u32 = 10;
    pub const MIN_COURSE_ID_LENGTH: u32 = 3;
    pub const MIN_URI_LENGTH: u32 = 10;

    // URI validation patterns
    pub const VALID_URI_SCHEMES: &'static [&'static str] = &["https://", "ipfs://", "ar://"];

    // Forbidden characters for XSS prevention
    pub const FORBIDDEN_CHARS: &'static [char] = &[
        '<', '>', '"', '\'', '&', '\0', '\x01', '\x02', '\x03', '\x04', '\x05', '\x06', '\x07',
        '\x08', '\x0B', '\x0C', '\x0E', '\x0F', '\x10', '\x11', '\x12', '\x13', '\x14', '\x15',
        '\x16', '\x17', '\x18', '\x19', '\x1A', '\x1B', '\x1C', '\x1D', '\x1E', '\x1F', '\x7F',
    ];

    // Maximum allowed special characters ratio (to prevent spam/malformed content)
    pub const MAX_SPECIAL_CHAR_RATIO: f32 = 0.3;

    // Maximum consecutive identical characters
    pub const MAX_CONSECUTIVE_CHARS: usize = 5;

    // Maximum future time for expiry dates (100 years in seconds)
    pub const MAX_FUTURE_EXPIRY: u64 = 100 * 365 * 24 * 60 * 60;
}

/// Validation error types for enhanced error reporting
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    FieldTooShort {
        field: &'static str,
        min_length: u32,
        actual_length: usize,
    },
    FieldTooLong {
        field: &'static str,
        max_length: u32,
        actual_length: usize,
    },
    InvalidCharacters {
        field: &'static str,
        forbidden_char: char,
    },
    InvalidFormat {
        field: &'static str,
        reason: &'static str,
    },
    InvalidUri {
        reason: &'static str,
    },
    InvalidDate {
        reason: &'static str,
    },
    ContentQuality {
        reason: &'static str,
    },
    EmptyField {
        field: &'static str,
    },
}

/// Core validation utilities that can be reused across different contracts
pub struct CoreValidator;

impl CoreValidator {
    /// Validates string field length constraints
    pub fn validate_string_length(
        text: &str,
        field_name: &'static str,
        min_length: u32,
        max_length: u32,
    ) -> Result<(), ValidationError> {
        let len = text.len();

        if len < min_length as usize {
            return Err(ValidationError::FieldTooShort {
                field: field_name,
                min_length,
                actual_length: len,
            });
        }

        if len > max_length as usize {
            return Err(ValidationError::FieldTooLong {
                field: field_name,
                max_length,
                actual_length: len,
            });
        }

        Ok(())
    }

    /// Validates that string contains no forbidden characters
    pub fn validate_no_forbidden_chars(
        text: &str,
        field_name: &'static str,
    ) -> Result<(), ValidationError> {
        for &forbidden_char in ValidationConfig::FORBIDDEN_CHARS {
            if text.contains(forbidden_char) {
                return Err(ValidationError::InvalidCharacters {
                    field: field_name,
                    forbidden_char,
                });
            }
        }
        Ok(())
    }

    /// Validates text quality (prevents spam and malformed content)
    pub fn validate_text_quality(
        text: &str,
        field_name: &'static str,
    ) -> Result<(), ValidationError> {
        // Check for excessive whitespace
        if text.trim().is_empty() {
            return Err(ValidationError::EmptyField { field: field_name });
        }

        // Check for excessive special characters
        let special_char_count = text
            .chars()
            .filter(|&ch| !ch.is_alphanumeric() && !ch.is_whitespace())
            .count();

        let special_char_ratio = special_char_count as f32 / text.len() as f32;
        if special_char_ratio > ValidationConfig::MAX_SPECIAL_CHAR_RATIO {
            return Err(ValidationError::ContentQuality {
                reason: "Too many special characters",
            });
        }

        // Check for repeated characters (potential spam)
        Self::validate_no_excessive_repetition(text, field_name)?;

        Ok(())
    }

    /// Validates no excessive character repetition
    fn validate_no_excessive_repetition(
        text: &str,
        field_name: &'static str,
    ) -> Result<(), ValidationError> {
        let chars: Vec<char> = text.chars().collect();
        let mut consecutive_count = 1;

        for i in 1..chars.len() {
            if chars[i] == chars[i - 1] {
                consecutive_count += 1;
                if consecutive_count > ValidationConfig::MAX_CONSECUTIVE_CHARS {
                    return Err(ValidationError::ContentQuality {
                        reason: "Too many consecutive identical characters",
                    });
                }
            } else {
                consecutive_count = 1;
            }
        }

        Ok(())
    }

    /// Validates course ID format (alphanumeric with hyphens and underscores)
    pub fn validate_course_id_format(course_id: &str) -> Result<(), ValidationError> {
        // Course ID should contain only alphanumeric characters, hyphens, and underscores
        for ch in course_id.chars() {
            if !ch.is_alphanumeric() && ch != '-' && ch != '_' {
                return Err(ValidationError::InvalidFormat {
                    field: "course_id",
                    reason: "Only alphanumeric, hyphens, and underscores allowed",
                });
            }
        }

        // Should not start or end with separator
        if course_id.starts_with('-')
            || course_id.starts_with('_')
            || course_id.ends_with('-')
            || course_id.ends_with('_')
        {
            return Err(ValidationError::InvalidFormat {
                field: "course_id",
                reason: "Cannot start or end with separator",
            });
        }

        Ok(())
    }

    /// Validates URI scheme is allowed
    pub fn validate_uri_scheme(uri: &str) -> Result<(), ValidationError> {
        let uri_lower = uri.to_lowercase();

        let has_valid_scheme = ValidationConfig::VALID_URI_SCHEMES
            .iter()
            .any(|&scheme| uri_lower.starts_with(scheme));

        if !has_valid_scheme {
            return Err(ValidationError::InvalidUri {
                reason: "URI scheme must be https://, ipfs://, or ar://",
            });
        }

        Ok(())
    }

    /// Validates URI format structure
    pub fn validate_uri_format(uri: &str) -> Result<(), ValidationError> {
        // Should not contain spaces
        if uri.contains(' ') {
            return Err(ValidationError::InvalidUri {
                reason: "URI cannot contain spaces",
            });
        }

        // Should not have consecutive slashes after scheme
        if uri.contains("///") {
            return Err(ValidationError::InvalidUri {
                reason: "URI cannot have consecutive slashes",
            });
        }

        // For HTTPS URIs, validate domain structure
        if uri.starts_with("https://") {
            Self::validate_https_uri(&uri[8..])?;
        }

        // For IPFS URIs, validate hash format
        if uri.starts_with("ipfs://") {
            Self::validate_ipfs_uri(&uri[7..])?;
        }

        // For Arweave URIs, validate transaction ID format
        if uri.starts_with("ar://") {
            Self::validate_arweave_uri(&uri[5..])?;
        }

        Ok(())
    }

    /// Validates HTTPS URI domain structure
    fn validate_https_uri(domain_path: &str) -> Result<(), ValidationError> {
        if domain_path.is_empty() {
            return Err(ValidationError::InvalidUri {
                reason: "HTTPS URI must have domain",
            });
        }

        // Should contain at least a domain
        let parts: Vec<&str> = domain_path.split('/').collect();
        if parts.is_empty() || parts[0].is_empty() {
            return Err(ValidationError::InvalidUri {
                reason: "HTTPS URI must have valid domain",
            });
        }

        // Basic domain validation
        let domain = parts[0];
        if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
            return Err(ValidationError::InvalidUri {
                reason: "Invalid domain format",
            });
        }

        Ok(())
    }

    /// Validates IPFS URI hash format
    fn validate_ipfs_uri(hash: &str) -> Result<(), ValidationError> {
        // IPFS hash should be alphanumeric and of appropriate length
        if hash.len() < 40 || hash.len() > 100 {
            return Err(ValidationError::InvalidUri {
                reason: "IPFS hash must be 40-100 characters",
            });
        }

        // Should contain only alphanumeric characters
        if !hash.chars().all(|c| c.is_alphanumeric()) {
            return Err(ValidationError::InvalidUri {
                reason: "IPFS hash must be alphanumeric",
            });
        }

        Ok(())
    }

    /// Validates Arweave URI transaction ID format
    fn validate_arweave_uri(tx_id: &str) -> Result<(), ValidationError> {
        // Arweave transaction ID should be 43 characters, base64url encoded
        if tx_id.len() != 43 {
            return Err(ValidationError::InvalidUri {
                reason: "Arweave transaction ID must be 43 characters",
            });
        }

        // Should contain only valid base64url characters
        for ch in tx_id.chars() {
            if !ch.is_alphanumeric() && ch != '-' && ch != '_' {
                return Err(ValidationError::InvalidUri {
                    reason: "Arweave transaction ID must be base64url encoded",
                });
            }
        }

        Ok(())
    }

    /// Validates expiry date
    pub fn validate_expiry_date(env: &Env, expiry_date: u64) -> Result<(), ValidationError> {
        let current_time = env.ledger().timestamp();

        // Allow non-expiring certificates when expiry_date == 0
        if expiry_date == 0 {
            return Ok(());
        }

        // Otherwise, expiry date must be in the future
        if expiry_date <= current_time {
            return Err(ValidationError::InvalidDate {
                reason: "Expiry date must be in the future",
            });
        }

        // Expiry date should not be too far in the future
        let max_future_time = current_time + ValidationConfig::MAX_FUTURE_EXPIRY;
        if expiry_date > max_future_time {
            return Err(ValidationError::InvalidDate {
                reason: "Expiry date too far in the future (max 100 years)",
            });
        }

        Ok(())
    }

    /// Validates certificate ID format and requirements
    pub fn validate_certificate_id(certificate_id: &BytesN<32>) -> Result<(), ValidationError> {
        // Check if all bytes are zero (invalid certificate ID)
        let bytes = certificate_id.to_array();
        if bytes.iter().all(|&b| b == 0) {
            return Err(ValidationError::EmptyField {
                field: "certificate_id",
            });
        }

        Ok(())
    }

    /// Sanitizes text content for safe storage and display
    pub fn sanitize_text(text: &str) -> String {
        text.chars()
            .filter(|&ch| !ValidationConfig::FORBIDDEN_CHARS.contains(&ch))
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Validates complete text field with all checks
    pub fn validate_text_field(
        text: &str,
        field_name: &'static str,
        min_length: u32,
        max_length: u32,
    ) -> Result<(), ValidationError> {
        Self::validate_string_length(text, field_name, min_length, max_length)?;
        Self::validate_no_forbidden_chars(text, field_name)?;
        Self::validate_text_quality(text, field_name)?;
        Ok(())
    }

    /// Validates complete URI with all checks
    pub fn validate_uri(uri: &str) -> Result<(), ValidationError> {
        Self::validate_string_length(
            uri,
            "uri",
            ValidationConfig::MIN_URI_LENGTH,
            ValidationConfig::MAX_URI_LENGTH,
        )?;
        Self::validate_no_forbidden_chars(uri, "uri")?;
        Self::validate_uri_scheme(uri)?;
        Self::validate_uri_format(uri)?;
        Ok(())
    }

    /// Validates complete course ID with all checks
    pub fn validate_course_id(course_id: &str) -> Result<(), ValidationError> {
        Self::validate_string_length(
            course_id,
            "course_id",
            ValidationConfig::MIN_COURSE_ID_LENGTH,
            ValidationConfig::MAX_COURSE_ID_LENGTH,
        )?;
        Self::validate_no_forbidden_chars(course_id, "course_id")?;
        Self::validate_course_id_format(course_id)?;
        Ok(())
    }
}

#[cfg(test)]
use soroban_sdk::testutils::Ledger;
mod tests {
    use super::*;
    use soroban_sdk::{BytesN, Env};

    #[test]
    fn test_validate_string_length_success() {
        let result = CoreValidator::validate_string_length("Valid text", "test_field", 3, 20);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_string_length_too_short() {
        let result = CoreValidator::validate_string_length("AB", "test_field", 3, 20);
        assert!(matches!(result, Err(ValidationError::FieldTooShort { .. })));
    }

    #[test]
    fn test_validate_string_length_too_long() {
        let long_text = "A".repeat(21);
        let result = CoreValidator::validate_string_length(&long_text, "test_field", 3, 20);
        assert!(matches!(result, Err(ValidationError::FieldTooLong { .. })));
    }

    #[test]
    fn test_validate_forbidden_chars() {
        let result = CoreValidator::validate_no_forbidden_chars("Text with <script>", "test_field");
        assert!(matches!(
            result,
            Err(ValidationError::InvalidCharacters { .. })
        ));
    }

    #[test]
    fn test_validate_text_quality_empty() {
        let result = CoreValidator::validate_text_quality("   ", "test_field");
        assert!(matches!(result, Err(ValidationError::EmptyField { .. })));
    }

    #[test]
    fn test_validate_text_quality_too_many_special_chars() {
        let result = CoreValidator::validate_text_quality("!@#$%^&*()", "test_field");
        assert!(matches!(
            result,
            Err(ValidationError::ContentQuality { .. })
        ));
    }

    #[test]
    fn test_validate_uri_scheme_valid() {
        assert!(CoreValidator::validate_uri_scheme("https://example.com").is_ok());
        assert!(CoreValidator::validate_uri_scheme("ipfs://QmHash").is_ok());
        assert!(CoreValidator::validate_uri_scheme("ar://TxId").is_ok());
    }

    #[test]
    fn test_validate_uri_scheme_invalid() {
        let result = CoreValidator::validate_uri_scheme("http://example.com");
        assert!(matches!(result, Err(ValidationError::InvalidUri { .. })));
    }

    #[test]
    fn test_validate_course_id_format_valid() {
        assert!(CoreValidator::validate_course_id_format("CS-101_Advanced").is_ok());
    }

    #[test]
    fn test_validate_course_id_format_invalid_chars() {
        let result = CoreValidator::validate_course_id_format("CS@101");
        assert!(matches!(result, Err(ValidationError::InvalidFormat { .. })));
    }

    #[test]
    fn test_validate_course_id_format_invalid_start() {
        let result = CoreValidator::validate_course_id_format("-CS101");
        assert!(matches!(result, Err(ValidationError::InvalidFormat { .. })));
    }

    #[test]
    fn test_validate_expiry_date_future() {
        let env = Env::default();
        let future_date = env.ledger().timestamp() + 86400; // 1 day in future
        assert!(CoreValidator::validate_expiry_date(&env, future_date).is_ok());
    }

    #[test]
    fn test_validate_expiry_date_past() {
        let env = Env::default();
        // Set a specific ledger timestamp to ensure consistency
        env.ledger().set_timestamp(1000000);
        let past_date = 500000; // Explicitly in the past
        let result = CoreValidator::validate_expiry_date(&env, past_date);
        assert!(matches!(result, Err(ValidationError::InvalidDate { .. })));
    }

    #[test]
    fn test_validate_certificate_id_valid() {
        let env = Env::default();
        let valid_id = BytesN::from_array(&env, &[1u8; 32]);
        assert!(CoreValidator::validate_certificate_id(&valid_id).is_ok());
    }

    #[test]
    fn test_validate_certificate_id_zero() {
        let env = Env::default();
        let zero_id = BytesN::from_array(&env, &[0u8; 32]);
        let result = CoreValidator::validate_certificate_id(&zero_id);
        assert!(matches!(result, Err(ValidationError::EmptyField { .. })));
    }

    #[test]
    fn test_sanitize_text() {
        let dirty_text = "Clean text with <script> and 'quotes'";
        let clean_text = CoreValidator::sanitize_text(dirty_text);
        assert!(!clean_text.contains('<'));
        assert!(!clean_text.contains('>'));
        assert!(!clean_text.contains('\''));
    }
}
