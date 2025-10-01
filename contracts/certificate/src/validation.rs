use soroban_sdk::{String, Env, BytesN, Address};
use crate::errors::CertificateError;
use crate::types::MintCertificateParams;
use shared::validation::{CoreValidator, ValidationConfig};

/// Certificate-specific metadata validation utilities
pub struct MetadataValidator;

impl MetadataValidator {
    /// Validates complete certificate metadata parameters with detailed error reporting
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
        CoreValidator::validate_certificate_id(certificate_id)
            .map_err(|_| CertificateError::InvalidMetadata)
    }
    
    /// Validates course ID format and content
    fn validate_course_id(course_id: &String) -> Result<(), CertificateError> {
        // Convert Soroban String to &str for validation
        let course_str = course_id.to_string();
        CoreValidator::validate_course_id(&course_str)
            .map_err(|_| CertificateError::InvalidMetadata)
    }
    
    /// Validates certificate title
    fn validate_title(title: &String) -> Result<(), CertificateError> {
        // Convert Soroban String to &str for validation
        let title_str = title.to_string();
        CoreValidator::validate_text_field(
            &title_str,
            "title",
            ValidationConfig::MIN_TITLE_LENGTH,
            ValidationConfig::MAX_TITLE_LENGTH,
        ).map_err(|_| CertificateError::InvalidMetadata)
    }
    
    /// Validates certificate description
    fn validate_description(description: &String) -> Result<(), CertificateError> {
        let desc_str = description.to_string();
        CoreValidator::validate_text_field(
            &desc_str,
            "description",
            ValidationConfig::MIN_DESCRIPTION_LENGTH,
            ValidationConfig::MAX_DESCRIPTION_LENGTH,
        ).map_err(|_| CertificateError::InvalidMetadata)
    }
    
    /// Validates metadata URI format and scheme
    fn validate_metadata_uri(uri: &String) -> Result<(), CertificateError> {
        let uri_str = uri.to_string();
        CoreValidator::validate_uri(&uri_str)
            .map_err(|_| CertificateError::InvalidUri)
    }
    
    /// Validates address format (basic validation, Soroban handles most)
    fn validate_address(address: &Address) -> Result<(), CertificateError> {
        // Soroban SDK handles basic address validation
        // Additional custom validation can be added here if needed
        Ok(())
    }
    
    /// Validates expiry date
    fn validate_expiry_date(env: &Env, expiry_date: u64) -> Result<(), CertificateError> {
        CoreValidator::validate_expiry_date(env, expiry_date)
            .map_err(|_| CertificateError::InvalidMetadata)
    }
    
    /// Validates URI update parameters
    pub fn validate_uri_update(uri: &String) -> Result<(), CertificateError> {
        Self::validate_metadata_uri(uri)
    }
    
    /// Sanitizes text content for safe storage and display
    pub fn sanitize_text(text: &str) -> String {
        CoreValidator::sanitize_text(text)
    }
    
    /// Validates batch certificate parameters with enhanced checking
    pub fn validate_batch_params(env: &Env, params_list: &[MintCertificateParams]) -> Result<(), CertificateError> {
        // Validate batch size
        if params_list.is_empty() {
            return Err(CertificateError::InvalidInput);
        }
        
        if params_list.len() > ValidationConfig::MAX_BATCH_SIZE as usize {
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
    
    /// Enhanced validation for metadata updates with detailed error reporting
    pub fn validate_metadata_update(
        title: Option<&String>,
        description: Option<&String>,
        metadata_uri: Option<&String>,
    ) -> Result<(), CertificateError> {
        if let Some(title) = title {
            Self::validate_title(title)?;
        }
        
        if let Some(description) = description {
            Self::validate_description(description)?;
        }
        
        if let Some(uri) = metadata_uri {
            Self::validate_metadata_uri(uri)?;
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
    fn test_validate_title_xss_attempt() {
        let xss_title = SorobanString::from_str(&Env::default(), "<script>alert('xss')</script>");
        assert_eq!(MetadataValidator::validate_title(&xss_title), Err(CertificateError::InvalidMetadata));
    }
    
    #[test]
    fn test_validate_description_success() {
        let valid_desc = SorobanString::from_str(&Env::default(), "This is a valid certificate description.");
        assert!(MetadataValidator::validate_description(&valid_desc).is_ok());
    }
    
    #[test]
    fn test_validate_description_too_short() {
        let short_desc = SorobanString::from_str(&Env::default(), "Short");
        assert_eq!(MetadataValidator::validate_description(&short_desc), Err(CertificateError::InvalidMetadata));
    }
    
    #[test]
    fn test_validate_description_too_long() {
        let long_desc = SorobanString::from_str(&Env::default(), &"A".repeat(1001));
        assert_eq!(MetadataValidator::validate_description(&long_desc), Err(CertificateError::InvalidMetadata));
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
    fn test_validate_uri_arweave_success() {
        let valid_uri = SorobanString::from_str(&Env::default(), "ar://ABC123abc456XYZ789xyz012DEF345def678GHI");
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
        let future_date = env.ledger().timestamp() + 86400;
        assert!(MetadataValidator::validate_expiry_date(&env, future_date).is_ok());
    }
    
    #[test]
    fn test_validate_expiry_date_past() {
        let env = Env::default();
        let past_date = env.ledger().timestamp() - 86400;
        assert_eq!(MetadataValidator::validate_expiry_date(&env, past_date), Err(CertificateError::InvalidMetadata));
    }
    
    #[test]
    fn test_validate_batch_params_success() {
        let env = Env::default();
        let certificate_id = BytesN::from_array(&env, &[1u8; 32]);
        let params = MintCertificateParams {
            certificate_id,
            course_id: SorobanString::from_str(&env, "CS-101"),
            student: Address::generate(&env),
            title: SorobanString::from_str(&env, "Valid Certificate"),
            description: SorobanString::from_str(&env, "A valid test certificate"),
            metadata_uri: SorobanString::from_str(&env, "https://example.com/metadata.json"),
            expiry_date: env.ledger().timestamp() + 86400,
        };
        
        let batch = vec![&env, params];
        assert!(MetadataValidator::validate_batch_params(&env, &batch.to_array()).is_ok());
    }
    
    #[test]
    fn test_validate_batch_params_empty() {
        let env = Env::default();
        let empty_batch: [MintCertificateParams; 0] = [];
        assert_eq!(
            MetadataValidator::validate_batch_params(&env, &empty_batch),
            Err(CertificateError::InvalidInput)
        );
    }
    
    #[test]
    fn test_validate_metadata_update_success() {
        let env = Env::default();
        let title = SorobanString::from_str(&env, "Updated Title");
        let description = SorobanString::from_str(&env, "Updated description");
        let uri = SorobanString::from_str(&env, "https://example.com/updated.json");
        
        assert!(MetadataValidator::validate_metadata_update(
            Some(&title),
            Some(&description),
            Some(&uri)
        ).is_ok());
    }
    
    #[test]
    fn test_validate_metadata_update_invalid_title() {
        let env = Env::default();
        let invalid_title = SorobanString::from_str(&env, "<script>alert('xss')</script>");
        
        assert_eq!(
            MetadataValidator::validate_metadata_update(Some(&invalid_title), None, None),
            Err(CertificateError::InvalidMetadata)
        );
    }
    
    #[test]
    fn test_sanitize_text() {
        let dirty_text = "Clean text with <script> and 'quotes'";
        let clean_text = MetadataValidator::sanitize_text(dirty_text);
        assert!(!clean_text.contains('<'));
        assert!(!clean_text.contains('>'));
        assert!(!clean_text.contains('\''));
    }
}
