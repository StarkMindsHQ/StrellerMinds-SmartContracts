use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GdprError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 10,
    ExportRequestNotFound = 20,
    ExportNotReady = 21,
    ExportExpired = 22,
    InvalidRequestId = 23,
    DataRetrievalFailed = 30,
    RateLimitExceeded = 40,
}