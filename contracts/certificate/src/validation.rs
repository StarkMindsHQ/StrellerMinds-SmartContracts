use soroban_sdk::{String, Env, BytesN, Address};
use crate::errors::CertificateError;
use crate::types::MintCertificateParams;

/// Configuration constants for metadata validation
pub struct ValidationConfig;

impl ValidationConfig {
    // Size limits (in bytes)
    pub const MAX_TITLE_LENGTH: u32 = 200;
    pub const MAX_DESCRIPTION_LENGTH: u32 = 1000;
    pub const MAX_COURSE_ID_LENGTH: u32 = 100;
    pub const MAX_URI_LENGTH: u32 = 500;
    
    // Minimum lengths
    pub const MIN_TITLE_LENGTH: u32 = 3;
    pub const MIN_DESCRIPTION_LENGTH: u32 = 10;
    pub const MIN_COURSE_ID_LENGTH: u32 = 3;
    pub const MIN_URI_LENGTH: u32 = 10;
    
    // URI validation patterns
    pub const VALID_URI_SCHEMES: &'static [&'static str] = &["https://", "ipfs://", "ar://"];
    
    // Forbidden characters for XSS prevention
    pub const FORBIDDEN_CHARS: &'static [char] = &['<', '>', '"', '\'', '&', '\0', '\x01', '\x02', '\x03', '\x04', '\x05', '\x06', '\x07', '\x08', '\x0B', '\x0C', '\x0E', '\x0F', '\x10', '\x11', '\x12', '\x13', '\x14', '\x15', '\x16', '\x17', '\x18', '\x19', '\x1A', '\x1B', '\x1C', '\x1D', '\x1E', '\x1F', '\x7F'];
    
    // Maximum allowed special characters ratio (to prevent spam/malformed content)
    pub const MAX_SPECIAL_CHAR_RATIO: f32 = 0.3;
}

/// Metadata validation utilities
pub struct MetadataValidator;

impl MetadataValidator {
    /// Validates complete certificate metadata parameters
    pub fn validate_mint_params(env: &Env, params: &MintCertificateParams) -> Result<(), CertificateError> {
        // Validate certificate ID
        Self::validate_certificate_id(&params.certificate_id)?;
        
        // Validate string fields
        Self::validate_course_id(&params.course_id)?;
        Self::validate_title(&params.title)?;
        Self::validate_description(&params.description)?;
        Self::validate_metadata_uri(&params.metadata_uri)?;
        
        // Validate addresses (Soroban SDK handles basic address validation)
        Self::validate_address(&params.student)?;
        
        // Validate dates
        Self::validate_expiry_date(env, params.expiry_date)?;
        
        Ok(())
    }
    
    /// Validates certificate ID format and uniqueness requirements
    fn validate_certificate_id(certificate_id: &BytesN<32>) -> Result<(), CertificateError> {
        // BytesN<32> is already validated by Soroban SDK for length
        // Additional validation can be added here if needed (e.g., format requirements)
        
        // Check if all bytes are zero (invalid certificate ID)
        let bytes = certificate_id.to_array();
        if bytes.iter().all(|&b| b == 0) {
            return Err(CertificateError::InvalidMetadata);
        }
        
        Ok(())
    }
    
    /// Validates course ID format and content
    fn validate_course_id(course_id: &String) -> Result<(), CertificateError> {
        let course_str = course_id.to_string();
        
        // Check length constraints
        if course_str.len() < ValidationConfig::MIN_COURSE_ID_LENGTH as usize {
            return Err(CertificateError::InvalidMetadata);
        }
        
        if course_str.len() > ValidationConfig::MAX_COURSE_ID_LENGTH as usize {
            return Err(CertificateError::InvalidMetadata);
        }
        
        // Check for forbidden characters
        Self::validate_no_forbidden_chars(&course_str)?;
        
        // Validate alphanumeric with allowed separators
        Self::validate_course_id_format(&course_str)?;
        
        Ok(())
    }
    
    /// Validates certificate title
    fn validate_title(title: &String) -> Result<(), CertificateError> {
        let title_str = title.to_string();
        
        // Check length constraints
        if title_str.len() < ValidationConfig::MIN_TITLE_LENGTH as usize {
            return Err(CertificateError::InvalidMetadata);
        }
        
        if title_str.len() > ValidationConfig::MAX_TITLE_LENGTH as usize {
            return Err(CertificateError::InvalidMetadata);
        }
        
        // Check for forbidden characters
        Self::validate_no_forbidden_chars(&title_str)?;
        
        // Validate content quality
        Self::validate_text_quality(&title_str)?;
        
        Ok(())
    }
    
    /// Validates certificate description
    fn validate_description(description: &String) -> Result<(), CertificateError> {
        let desc_str = description.to_string();
        
        // Check length constraints
        if desc_str.len() < ValidationConfig::MIN_DESCRIPTION_LENGTH as usize {
            return Err(CertificateError::InvalidMetadata);
        }
        
        if desc_str.len() > ValidationConfig::MAX_DESCRIPTION_LENGTH as usize {
            return Err(CertificateError::InvalidMetadata);
        }
        
        // Check for forbidden characters
        Self::validate_no_forbidden_chars(&desc_str)?;
        
        // Validate content quality
        Self::validate_text_quality(&desc_str)?;
        
        Ok(())
    }
    
    /// Validates metadata URI format and scheme
    fn validate_metadata_uri(uri: &String) -> Result<(), CertificateError> {
        let uri_str = uri.to_string();
        
        // Check length constraints
        if uri_str.len() < ValidationConfig::MIN_URI_LENGTH as usize {
            return Err(CertificateError::InvalidUri);
        }
        
        if uri_str.len() > ValidationConfig::MAX_URI_LENGTH as usize {
            return Err(CertificateError::InvalidUri);
        }
        
        // Check for forbidden characters
        Self::validate_no_forbidden_chars(&uri_str)?;
        
        // Validate URI scheme
        Self::validate_uri_scheme(&uri_str)?;
        
        // Validate URI format
        Self::validate_uri_format(&uri_str)?;
        
        Ok(())
    }
    
    /// Validates address format (basic validation, Soroban handles most)
    fn validate_address(address: &Address) -> Result<(), CertificateError> {
        // Soroban SDK handles basic address validation
        // Additional custom validation can be added here if needed
        Ok(())
    }
    
    /// Validates expiry date
    fn validate_expiry_date(env: &Env, expiry_date: u64) -> Result<(), CertificateError> {
        let current_time = env.ledger().timestamp();
        
        // Expiry date must be in the future
        if expiry_date <= current_time {
            return Err(CertificateError::InvalidMetadata);
        }
        
        // Expiry date should not be too far in the future (e.g., 100 years)
        let max_future_time = current_time + (100 * 365 * 24 * 60 * 60); // 100 years in seconds
        if expiry_date > max_future_time {
            return Err(CertificateError::InvalidMetadata);
        }
        
        Ok(())
    }
    
    /// Validates that string contains no forbidden characters
    fn validate_no_forbidden_chars(text: &str) -> Result<(), CertificateError> {
        for &forbidden_char in ValidationConfig::FORBIDDEN_CHARS {
            if text.contains(forbidden_char) {
                return Err(CertificateError::InvalidMetadata);
            }
        }
        Ok(())
    }
    
    /// Validates course ID format (alphanumeric with hyphens and underscores)
    fn validate_course_id_format(course_id: &str) -> Result<(), CertificateError> {
        // Course ID should contain only alphanumeric characters, hyphens, and underscores
        for ch in course_id.chars() {
            if !ch.is_alphanumeric() && ch != '-' && ch != '_' {
                return Err(CertificateError::InvalidMetadata);
            }
        }
        
        // Should not start or end with separator
        if course_id.starts_with('-') || course_id.starts_with('_') || 
           course_id.ends_with('-') || course_id.ends_with('_') {
            return Err(CertificateError::InvalidMetadata);
        }
        
        Ok(())
    }
    
    /// Validates text quality (prevents spam and malformed content)
    fn validate_text_quality(text: &str) -> Result<(), CertificateError> {
        // Check for excessive special characters
        let special_char_count = text.chars()
            .filter(|&ch| !ch.is_alphanumeric() && !ch.is_whitespace())
            .count();
        
        let special_char_ratio = special_char_count as f32 / text.len() as f32;
        if special_char_ratio > ValidationConfig::MAX_SPECIAL_CHAR_RATIO {
            return Err(CertificateError::InvalidMetadata);
        }
        
        // Check for excessive whitespace
        if text.trim().is_empty() {
            return Err(CertificateError::InvalidMetadata);
        }
        
        // Check for repeated characters (potential spam)
        Self::validate_no_excessive_repetition(text)?;
        
        Ok(())
    }
    
    /// Validates URI scheme is allowed
    fn validate_uri_scheme(uri: &str) -> Result<(), CertificateError> {
        let uri_lower = uri.to_lowercase();
        
        let has_valid_scheme = ValidationConfig::VALID_URI_SCHEMES
            .iter()
            .any(|&scheme| uri_lower.starts_with(scheme));
        
        if !has_valid_scheme {
            return Err(CertificateError::InvalidUri);
        }
        
        Ok(())
    }
    
    /// Validates URI format structure
    fn validate_uri_format(uri: &str) -> Result<(), CertificateError> {
        // Basic URI format validation
        
        // Should not contain spaces
        if uri.contains(' ') {
            return Err(CertificateError::InvalidUri);
        }
        
        // Should not have consecutive slashes after scheme
        if uri.contains("///") {
            return Err(CertificateError::InvalidUri);
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
    fn validate_https_uri(domain_path: &str) -> Result<(), CertificateError> {
        if domain_path.is_empty() {
            return Err(CertificateError::InvalidUri);
        }
        
        // Should contain at least a domain
        let parts: Vec<&str> = domain_path.split('/').collect();
        if parts.is_empty() || parts[0].is_empty() {
            return Err(CertificateError::InvalidUri);
        }
        
        // Basic domain validation
        let domain = parts[0];
        if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
            return Err(CertificateError::InvalidUri);
        }
        
        Ok(())
    }
    
    /// Validates IPFS URI hash format
    fn validate_ipfs_uri(hash: &str) -> Result<(), CertificateError> {
        // IPFS hash should be alphanumeric and of appropriate length
        if hash.len() < 40 || hash.len() > 100 {
            return Err(CertificateError::InvalidUri);
        }
        
        // Should contain only alphanumeric characters
        if !hash.chars().all(|c| c.is_alphanumeric()) {
            return Err(CertificateError::InvalidUri);
        }
        
        Ok(())
    }
    
    /// Validates Arweave URI transaction ID format
    fn validate_arweave_uri(tx_id: &str) -> Result<(), CertificateError> {
        // Arweave transaction ID should be 43 characters, base64url encoded
        if tx_id.len() != 43 {
            return Err(CertificateError::InvalidUri);
        }
        
        // Should contain only valid base64url characters
        for ch in tx_id.chars() {
            if !ch.is_alphanumeric() && ch != '-' && ch != '_' {
                return Err(CertificateError::InvalidUri);
            }
        }
        
        Ok(())
    }
    
    /// Validates no excessive character repetition
    fn validate_no_excessive_repetition(text: &str) -> Result<(), CertificateError> {
        let chars: Vec<char> = text.chars().collect();
        let mut consecutive_count = 1;
        
        for i in 1..chars.len() {
            if chars[i] == chars[i-1] {
                consecutive_count += 1;
                if consecutive_count > 5 { // Max 5 consecutive identical characters
                    return Err(CertificateError::InvalidMetadata);
                }
            } else {
                consecutive_count = 1;
            }
        }
        
        Ok(())
    }
    
    /// Validates URI update parameters
    pub fn validate_uri_update(uri: &String) -> Result<(), CertificateError> {
        Self::validate_metadata_uri(uri)
    }
    
    /// Sanitizes text content for safe storage and display
    pub fn sanitize_text(text: &str) -> String {
        text.chars()
            .filter(|&ch| !ValidationConfig::FORBIDDEN_CHARS.contains(&ch))
            .collect::<String>()
            .trim()
            .to_string()
    }
    
    /// Validates batch certificate parameters
    pub fn validate_batch_params(env: &Env, params_list: &[MintCertificateParams]) -> Result<(), CertificateError> {
        // Validate batch size
        if params_list.is_empty() {
            return Err(CertificateError::InvalidInput);
        }
        
        if params_list.len() > 100 { // Max 100 certificates per batch
            return Err(CertificateError::InvalidInput);
        }
        
        // Validate each certificate in the batch
        for params in params_list {
            Self::validate_mint_params(env, params)?;
        }
        
        // Check for duplicate certificate IDs in batch
        for i in 0..params_list.len() {
            for j in (i + 1)..params_list.len() {
                if params_list[i].certificate_id == params_list[j].certificate_id {
                    return Err(CertificateError::CertificateAlreadyExists);
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String as SorobanString, BytesN};
    
    #[test]
    fn test_validate_title_success() {
        let valid_title = SorobanString::from_str(&Env::default(), "Valid Certificate Title");
        assert!(MetadataValidator::validate_title(&valid_title).is_ok());
    }
    
    #[test]
    fn test_validate_title_too_short() {
        let short_title = SorobanString::from_str(&Env::default(), "AB");
        assert_eq!(MetadataValidator::validate_title(&short_title), Err(CertificateError::InvalidMetadata));
    }
    
    #[test]
    fn test_validate_title_too_long() {
        let long_title = SorobanString::from_str(&Env::default(), &"A".repeat(201));
        assert_eq!(MetadataValidator::validate_title(&long_title), Err(CertificateError::InvalidMetadata));
    }
    
    #[test]
    fn test_validate_title_forbidden_chars() {
        let malicious_title = SorobanString::from_str(&Env::default(), "Title with <script>");
        assert_eq!(MetadataValidator::validate_title(&malicious_title), Err(CertificateError::InvalidMetadata));
    }
    
    #[test]
    fn test_validate_uri_https_success() {
        let valid_uri = SorobanString::from_str(&Env::default(), "https://example.com/metadata.json");
        assert!(MetadataValidator::validate_metadata_uri(&valid_uri).is_ok());
    }
    
    #[test]
    fn test_validate_uri_ipfs_success() {
        let valid_uri = SorobanString::from_str(&Env::default(), "ipfs://QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");
        assert!(MetadataValidator::validate_metadata_uri(&valid_uri).is_ok());
    }
    
    #[test]
    fn test_validate_uri_invalid_scheme() {
        let invalid_uri = SorobanString::from_str(&Env::default(), "http://example.com/metadata.json");
        assert_eq!(MetadataValidator::validate_metadata_uri(&invalid_uri), Err(CertificateError::InvalidUri));
    }
    
    #[test]
    fn test_validate_course_id_success() {
        let valid_course_id = SorobanString::from_str(&Env::default(), "CS-101_Advanced");
        assert!(MetadataValidator::validate_course_id(&valid_course_id).is_ok());
    }
    
    #[test]
    fn test_validate_course_id_invalid_chars() {
        let invalid_course_id = SorobanString::from_str(&Env::default(), "CS@101");
        assert_eq!(MetadataValidator::validate_course_id(&invalid_course_id), Err(CertificateError::InvalidMetadata));
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
        let past_date = env.ledger().timestamp() - 86400; // 1 day in past
        assert_eq!(MetadataValidator::validate_expiry_date(&env, past_date), Err(CertificateError::InvalidMetadata));
    }
}
