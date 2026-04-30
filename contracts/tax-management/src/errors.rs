use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TaxError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,

    InvalidIpfsHash = 10,
    InvalidTaxYear = 11,
    InvalidPropertyId = 12,
    DocumentNotFound = 13,
    DocumentAlreadyVerified = 14,

    AdvisorAlreadyRegistered = 20,
    AdvisorNotFound = 21,
    AdvisorInactive = 22,
    InvalidLicense = 23,
    NoJurisdictions = 24,
    AdvisorNotAssigned = 25,
}
