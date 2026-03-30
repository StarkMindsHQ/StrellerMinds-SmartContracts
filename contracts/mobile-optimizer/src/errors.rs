use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MobileOptimizerError {
    /// The contract has not been initialized yet.
    NotInitialized = 1,
    /// The contract has already been initialized and cannot be re-initialized.
    AlreadyInitialized = 2,
    /// Creating a new mobile session failed.
    SessionCreationFailed = 3,
    /// Updating an existing session's state failed.
    SessionUpdateFailed = 4,
    /// No session found for the specified session ID.
    SessionNotFound = 5,
    /// The session has expired and can no longer be used.
    SessionExpired = 6,
    /// Executing the batch of operations failed.
    BatchExecutionFailed = 7,
    /// No batch found for the specified batch ID.
    BatchNotFound = 8,
    /// The batch has expired before execution could complete.
    BatchExpired = 9,
    /// Estimating gas cost for the operation failed.
    GasEstimationFailed = 10,
    /// The optimization process failed to produce a result.
    OptimizationFailed = 11,
    /// A quick interaction flow failed to complete.
    InteractionFailed = 12,
    /// Queuing or executing an offline operation failed.
    OfflineOperationFailed = 13,
    /// Syncing offline operations to the network failed.
    OfflineSyncFailed = 14,
    /// The offline operation queue has reached its capacity limit.
    OfflineQueueFull = 15,
    /// Resolving conflicting offline operations failed.
    ConflictResolutionFailed = 16,
    /// Updating the user's mobile preferences failed.
    PreferenceUpdateFailed = 17,
    /// Analytics data is not available for the requested query.
    AnalyticsNotAvailable = 18,
    /// The optimizer configuration record was not found in storage.
    ConfigNotFound = 19,
    /// The admin address has not been set in storage.
    AdminNotSet = 20,
    /// Caller is not the authorized admin of this contract.
    UnauthorizedAdmin = 21,
    /// Caller is not authorized to perform this operation.
    Unauthorized = 22,
    /// A content cache read or write operation failed.
    CacheError = 23,
    /// The content cache has reached its maximum size limit.
    CacheFull = 24,
    /// The device has not been registered for the user account.
    DeviceNotRegistered = 25,
    /// The user has reached the maximum number of registered devices.
    MaxDevicesReached = 26,
    /// A data synchronization operation failed.
    SyncFailed = 27,
    /// A security policy violation was detected.
    SecurityViolation = 28,
    /// Biometric authentication verification failed.
    BiometricAuthFailed = 29,
    /// The account is locked due to too many failed authentication attempts.
    AccountLocked = 30,
    /// A push notification or reminder operation failed.
    NotificationError = 31,
    /// A Progressive Web App operation failed.
    PwaError = 32,
    /// The provided input value is invalid or out of range.
    InvalidInput = 33,
    /// An unexpected internal error occurred.
    InternalError = 34,
    /// A content management operation failed.
    ContentError = 35,
    /// A collaboration feature operation failed.
    CollaborationError = 36,
    /// A user experience or UI preference operation failed.
    UserExperienceError = 37,
}
