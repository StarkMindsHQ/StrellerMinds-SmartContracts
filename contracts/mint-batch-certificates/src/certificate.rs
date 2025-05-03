use soroban_sdk::{contracttype, contracterror, BytesN, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CertificateType {
    Standard = 0,
    Premium = 1,
    Gold = 2,
}

impl CertificateType {
    pub fn from_u32(value: u32) -> Option<CertificateType> {
        match value {
            0 => Some(CertificateType::Standard),
            1 => Some(CertificateType::Premium),
            2 => Some(CertificateType::Gold),
            _ => None,
        }
    }

    pub fn to_u32(&self) -> u32 {
        match self {
            CertificateType::Standard => 0,
            CertificateType::Premium => 1,
            CertificateType::Gold => 2,
        }
    }
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ConversionError {
    InvalidValue = 1,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateData {
    pub id: u64,
    pub metadata_hash: BytesN<32>,
    pub valid_from: u64,
    pub valid_until: u64,
    pub revocable: bool,
    pub cert_type: CertificateType,
}

impl CertificateData {
    pub fn validate(&self, env: &Env) -> bool {
        // Validate certificate data
        // 1. Check if valid_from is before valid_until
        // 2. Check if valid_until is in the future
        let now = env.ledger().timestamp();
        
        // Valid time range check
        if self.valid_from >= self.valid_until {
            return false;
        }
        
        // Valid until should be in the future
        if self.valid_until <= now {
            return false;
        }
        
        true
    }
}
