#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeploymentEnv {
    Development,
    Staging,
    Production,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentationDefaults {
    pub max_doc_size: u32,
    pub require_review: bool,
    pub enable_contributions: bool,
    pub enable_analytics: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MobileDefaults {
    pub max_batch_size: u32,
    pub default_gas_limit: u64,
    pub session_timeout_seconds: u64,
    pub offline_queue_limit: u32,
    pub network_timeout_ms: u32,
    pub retry_attempts: u32,
    pub cache_ttl_seconds: u64,
    pub max_devices_per_user: u32,
    pub analytics_retention_days: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecurityDefaults {
    pub burst_detection_threshold: u32,
    pub burst_window_seconds: u64,
    pub error_rate_threshold: u32,
    pub actor_anomaly_threshold: u32,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout: u64,
    pub auto_mitigation_enabled: bool,
    pub rate_limit_per_window: u32,
    pub rate_limit_window: u64,
}

pub struct ContractConfig;

impl ContractConfig {
    pub const fn documentation(profile: DeploymentEnv) -> DocumentationDefaults {
        match profile {
            DeploymentEnv::Development => DocumentationDefaults {
                max_doc_size: 150_000,
                require_review: false,
                enable_contributions: true,
                enable_analytics: true,
            },
            DeploymentEnv::Staging => DocumentationDefaults {
                max_doc_size: 120_000,
                require_review: true,
                enable_contributions: true,
                enable_analytics: true,
            },
            DeploymentEnv::Production => DocumentationDefaults {
                max_doc_size: 100_000,
                require_review: true,
                enable_contributions: true,
                enable_analytics: true,
            },
        }
    }

    pub const fn mobile(profile: DeploymentEnv) -> MobileDefaults {
        match profile {
            DeploymentEnv::Development => MobileDefaults {
                max_batch_size: 20,
                default_gas_limit: 1_500_000,
                session_timeout_seconds: 7_200,
                offline_queue_limit: 200,
                network_timeout_ms: 45_000,
                retry_attempts: 7,
                cache_ttl_seconds: 172_800,
                max_devices_per_user: 10,
                analytics_retention_days: 30,
            },
            DeploymentEnv::Staging => MobileDefaults {
                max_batch_size: 15,
                default_gas_limit: 1_250_000,
                session_timeout_seconds: 5_400,
                offline_queue_limit: 150,
                network_timeout_ms: 35_000,
                retry_attempts: 6,
                cache_ttl_seconds: 129_600,
                max_devices_per_user: 7,
                analytics_retention_days: 60,
            },
            DeploymentEnv::Production => MobileDefaults {
                max_batch_size: 10,
                default_gas_limit: 1_000_000,
                session_timeout_seconds: 3_600,
                offline_queue_limit: 100,
                network_timeout_ms: 30_000,
                retry_attempts: 5,
                cache_ttl_seconds: 86_400,
                max_devices_per_user: 5,
                analytics_retention_days: 90,
            },
        }
    }

    pub const fn security(profile: DeploymentEnv) -> SecurityDefaults {
        match profile {
            DeploymentEnv::Development => SecurityDefaults {
                burst_detection_threshold: 250,
                burst_window_seconds: 60,
                error_rate_threshold: 20,
                actor_anomaly_threshold: 15,
                circuit_breaker_threshold: 8,
                circuit_breaker_timeout: 120,
                auto_mitigation_enabled: false,
                rate_limit_per_window: 200,
                rate_limit_window: 3_600,
            },
            DeploymentEnv::Staging => SecurityDefaults {
                burst_detection_threshold: 150,
                burst_window_seconds: 60,
                error_rate_threshold: 12,
                actor_anomaly_threshold: 12,
                circuit_breaker_threshold: 6,
                circuit_breaker_timeout: 240,
                auto_mitigation_enabled: true,
                rate_limit_per_window: 150,
                rate_limit_window: 3_600,
            },
            DeploymentEnv::Production => SecurityDefaults {
                burst_detection_threshold: 100,
                burst_window_seconds: 60,
                error_rate_threshold: 10,
                actor_anomaly_threshold: 10,
                circuit_breaker_threshold: 5,
                circuit_breaker_timeout: 300,
                auto_mitigation_enabled: true,
                rate_limit_per_window: 100,
                rate_limit_window: 3_600,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ContractConfig, DeploymentEnv};

    #[test]
    fn production_profile_stays_stricter_than_development() {
        let dev = ContractConfig::security(DeploymentEnv::Development);
        let prod = ContractConfig::security(DeploymentEnv::Production);

        assert!(prod.burst_detection_threshold < dev.burst_detection_threshold);
        assert!(prod.circuit_breaker_timeout > dev.circuit_breaker_timeout);
    }
}
