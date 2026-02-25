use soroban_sdk::{contracttype, Env, Map, String, Vec};

use crate::types::*;

pub struct NetworkManager;

impl NetworkManager {
    pub fn detect_network_quality(env: &Env) -> NetworkQuality {
        let timestamp = env.ledger().timestamp();
        match timestamp % 5 {
            0 => NetworkQuality::Offline,
            1 => NetworkQuality::Poor,
            2 => NetworkQuality::Fair,
            3 => NetworkQuality::Good,
            _ => NetworkQuality::Excellent,
        }
    }

    pub fn optimize_connection_settings(network_quality: &NetworkQuality) -> ConnectionSettings {
        match network_quality {
            NetworkQuality::Excellent => ConnectionSettings {
                timeout_ms: 5000,
                max_concurrent_operations: 5,
                batch_size_limit: 10,
                compression_enabled: false,
                priority_queue_enabled: false,
                aggressive_caching: false,
            },
            NetworkQuality::Good => ConnectionSettings {
                timeout_ms: 8000,
                max_concurrent_operations: 3,
                batch_size_limit: 7,
                compression_enabled: true,
                priority_queue_enabled: false,
                aggressive_caching: false,
            },
            NetworkQuality::Fair => ConnectionSettings {
                timeout_ms: 15000,
                max_concurrent_operations: 2,
                batch_size_limit: 5,
                compression_enabled: true,
                priority_queue_enabled: true,
                aggressive_caching: true,
            },
            NetworkQuality::Poor => ConnectionSettings {
                timeout_ms: 30000,
                max_concurrent_operations: 1,
                batch_size_limit: 3,
                compression_enabled: true,
                priority_queue_enabled: true,
                aggressive_caching: true,
            },
            NetworkQuality::Offline => ConnectionSettings {
                timeout_ms: 1000,
                max_concurrent_operations: 0,
                batch_size_limit: 0,
                compression_enabled: true,
                priority_queue_enabled: true,
                aggressive_caching: true,
            },
        }
    }

    pub fn get_bandwidth_optimization(
        _env: &Env,
        network_quality: &NetworkQuality,
        data_usage_mode: &DataUsageMode,
    ) -> BandwidthOptimization {
        let base_settings = Self::optimize_connection_settings(network_quality);

        let (image_quality, video_quality, prefetch) = match data_usage_mode {
            DataUsageMode::Unlimited => (100u32, 100u32, true),
            DataUsageMode::Conservative => (60, 50, false),
            DataUsageMode::WifiOnly => match network_quality {
                NetworkQuality::Excellent => (100, 100, true),
                _ => (40, 0, false),
            },
            DataUsageMode::Emergency => (20, 0, false),
        };

        let estimated_bandwidth_kbps = match network_quality {
            NetworkQuality::Excellent => 10000,
            NetworkQuality::Good => 5000,
            NetworkQuality::Fair => 1000,
            NetworkQuality::Poor => 200,
            NetworkQuality::Offline => 0,
        };

        BandwidthOptimization {
            connection_settings: base_settings,
            image_quality_percent: image_quality,
            video_quality_percent: video_quality,
            prefetch_enabled: prefetch,
            estimated_bandwidth_kbps,
            data_compression_ratio: Self::get_compression_ratio(network_quality),
        }
    }

    pub fn get_network_statistics(env: &Env, session_id: String) -> NetworkStatistics {
        NetworkStatistics {
            session_id,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_latency_ms: 0,
            data_sent_bytes: 0,
            data_received_bytes: 0,
            network_quality_samples: Map::new(env),
            retry_count: 0,
        }
    }

    pub fn adapt_to_network_change(
        env: &Env,
        previous_quality: &NetworkQuality,
        current_quality: &NetworkQuality,
    ) -> NetworkAdaptation {
        let degraded = Self::quality_level(current_quality) < Self::quality_level(previous_quality);

        let mut actions = Vec::new(env);
        if degraded {
            actions.push_back(String::from_str(env, "Enabling aggressive caching"));
            actions.push_back(String::from_str(env, "Reducing concurrent operations"));
            if *current_quality == NetworkQuality::Offline {
                actions.push_back(String::from_str(env, "Switching to offline mode"));
            }
        } else {
            actions.push_back(String::from_str(env, "Resuming normal operations"));
            if *current_quality == NetworkQuality::Excellent {
                actions.push_back(String::from_str(env, "Triggering background sync"));
            }
        }

        NetworkAdaptation {
            previous_quality: previous_quality.clone(),
            current_quality: current_quality.clone(),
            degraded,
            actions,
            new_settings: Self::optimize_connection_settings(current_quality),
        }
    }

    fn quality_level(quality: &NetworkQuality) -> u32 {
        match quality {
            NetworkQuality::Offline => 0,
            NetworkQuality::Poor => 1,
            NetworkQuality::Fair => 2,
            NetworkQuality::Good => 3,
            NetworkQuality::Excellent => 4,
        }
    }

    /// Returns ratio as basis points (10000 = 1.0x = no compression)
    fn get_compression_ratio(quality: &NetworkQuality) -> u32 {
        match quality {
            NetworkQuality::Excellent => 10000,
            NetworkQuality::Good => 8000,
            NetworkQuality::Fair => 5000,
            NetworkQuality::Poor => 3000,
            NetworkQuality::Offline => 2000,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectionSettings {
    pub timeout_ms: u32,
    pub max_concurrent_operations: u32,
    pub batch_size_limit: u32,
    pub compression_enabled: bool,
    pub priority_queue_enabled: bool,
    pub aggressive_caching: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BandwidthOptimization {
    pub connection_settings: ConnectionSettings,
    pub image_quality_percent: u32,
    pub video_quality_percent: u32,
    pub prefetch_enabled: bool,
    pub estimated_bandwidth_kbps: u32,
    pub data_compression_ratio: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkStatistics {
    pub session_id: String,
    pub total_requests: u32,
    pub successful_requests: u32,
    pub failed_requests: u32,
    pub avg_latency_ms: u32,
    pub data_sent_bytes: u64,
    pub data_received_bytes: u64,
    pub network_quality_samples: Map<String, u32>,
    pub retry_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkAdaptation {
    pub previous_quality: NetworkQuality,
    pub current_quality: NetworkQuality,
    pub degraded: bool,
    pub actions: Vec<String>,
    pub new_settings: ConnectionSettings,
}
