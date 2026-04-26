#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCodeCategory {
    Initialization,
    Authorization,
    Validation,
    NotFound,
    BusinessLogic,
    Monitoring,
    Configuration,
    Internal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ErrorDescriptor {
    pub code: &'static str,
    pub message: &'static str,
    pub action: &'static str,
    pub category: ErrorCodeCategory,
}

impl ErrorDescriptor {
    pub const fn new(
        code: &'static str,
        message: &'static str,
        action: &'static str,
        category: ErrorCodeCategory,
    ) -> Self {
        Self { code, message, action, category }
    }
}

pub trait StandardizedError {
    fn descriptor(&self) -> ErrorDescriptor;
}

#[cfg(test)]
mod tests {
    use super::{ErrorCodeCategory, ErrorDescriptor};

    #[test]
    fn descriptor_preserves_standard_fields() {
        let descriptor = ErrorDescriptor::new(
            "SHR-001",
            "Contract is already initialized",
            "Reuse the existing state instead of calling initialize again",
            ErrorCodeCategory::Initialization,
        );

        assert_eq!(descriptor.code, "SHR-001");
        assert_eq!(descriptor.message, "Contract is already initialized");
        assert_eq!(
            descriptor.action,
            "Reuse the existing state instead of calling initialize again"
        );
        assert_eq!(descriptor.category, ErrorCodeCategory::Initialization);
    }
}
