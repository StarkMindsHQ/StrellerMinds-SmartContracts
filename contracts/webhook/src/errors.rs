use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Debug, PartialEq)]
pub enum WebhookError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    WebhookNotFound = 4,
    WebhookInactive = 5,
    TooManyWebhooks = 6,
    InvalidUrl = 7,
    InvalidEventType = 8,
    DeliveryNotFound = 9,
    RetryLimitExceeded = 10,
    RetryTooEarly = 11,
    NoEventTypesSpecified = 12,
}
