use soroban_sdk::{contracttype, Address, Env, String, Vec};

use crate::types::*;

pub struct BatteryOptimizer;

impl BatteryOptimizer {
    pub fn initialize_battery_config(env: &Env, user: &Address) -> BatteryOptimizationConfig {
        let config = BatteryOptimizationConfig {
            low_battery_threshold: 20,
            critical_battery_threshold: 5,
            auto_power_saver: true,
            reduce_sync_frequency: true,
            disable_prefetch_on_low: true,
            reduce_animation: true,
            background_limit_minutes: 30,
        };
        env.storage()
            .persistent()
            .set(&DataKey::BatteryConfig(user.clone()), &config);
        config
    }

    pub fn update_battery_profile(
        env: &Env,
        user: &Address,
        device_id: String,
        battery_level: u32,
        is_charging: bool,
    ) -> Result<BatteryProfile, MobileOptimizerError> {
        let config: BatteryOptimizationConfig = env
            .storage()
            .persistent()
            .get(&DataKey::BatteryConfig(user.clone()))
            .unwrap_or_else(|| Self::initialize_battery_config(env, user));

        let power_mode = Self::determine_power_mode(battery_level, is_charging, &config);
        let estimated_runtime = Self::estimate_runtime(battery_level, &power_mode);

        let profile = BatteryProfile {
            user: user.clone(),
            device_id: device_id.clone(),
            battery_level,
            is_charging,
            power_mode,
            estimated_runtime_minutes: estimated_runtime,
            last_updated: env.ledger().timestamp(),
        };

        let key = Self::battery_key(env, &device_id);
        env.storage()
            .persistent()
            .set(&DataKey::BatteryProfile(key), &profile);

        Ok(profile)
    }

    pub fn get_battery_profile(
        env: &Env,
        device_id: &String,
    ) -> Result<BatteryProfile, MobileOptimizerError> {
        let key = Self::battery_key(env, device_id);
        env.storage()
            .persistent()
            .get(&DataKey::BatteryProfile(key))
            .ok_or(MobileOptimizerError::DeviceNotRegistered)
    }

    pub fn get_optimized_settings(
        env: &Env,
        user: &Address,
        device_id: &String,
    ) -> Result<BatteryOptimizedSettings, MobileOptimizerError> {
        let profile = Self::get_battery_profile(env, device_id)?;
        let config: BatteryOptimizationConfig = env
            .storage()
            .persistent()
            .get(&DataKey::BatteryConfig(user.clone()))
            .unwrap_or_else(|| Self::initialize_battery_config(env, user));

        let settings = match profile.power_mode {
            PowerMode::UltraSaver => BatteryOptimizedSettings {
                sync_interval_seconds: 3600,
                prefetch_enabled: false,
                animation_enabled: false,
                background_sync: false,
                max_concurrent_ops: 1,
                cache_aggressiveness: 100,
                reduce_image_quality: true,
                disable_video_autoplay: true,
            },
            PowerMode::PowerSaver => BatteryOptimizedSettings {
                sync_interval_seconds: 1800,
                prefetch_enabled: false,
                animation_enabled: false,
                background_sync: true,
                max_concurrent_ops: 2,
                cache_aggressiveness: 80,
                reduce_image_quality: true,
                disable_video_autoplay: true,
            },
            PowerMode::Adaptive => {
                if profile.battery_level < config.low_battery_threshold {
                    BatteryOptimizedSettings {
                        sync_interval_seconds: 1200,
                        prefetch_enabled: false,
                        animation_enabled: true,
                        background_sync: true,
                        max_concurrent_ops: 2,
                        cache_aggressiveness: 60,
                        reduce_image_quality: false,
                        disable_video_autoplay: true,
                    }
                } else {
                    BatteryOptimizedSettings {
                        sync_interval_seconds: 600,
                        prefetch_enabled: true,
                        animation_enabled: true,
                        background_sync: true,
                        max_concurrent_ops: 3,
                        cache_aggressiveness: 50,
                        reduce_image_quality: false,
                        disable_video_autoplay: false,
                    }
                }
            }
            PowerMode::Normal => BatteryOptimizedSettings {
                sync_interval_seconds: 300,
                prefetch_enabled: true,
                animation_enabled: true,
                background_sync: true,
                max_concurrent_ops: 5,
                cache_aggressiveness: 30,
                reduce_image_quality: false,
                disable_video_autoplay: false,
            },
            PowerMode::Performance => BatteryOptimizedSettings {
                sync_interval_seconds: 60,
                prefetch_enabled: true,
                animation_enabled: true,
                background_sync: true,
                max_concurrent_ops: 10,
                cache_aggressiveness: 10,
                reduce_image_quality: false,
                disable_video_autoplay: false,
            },
        };

        Ok(settings)
    }

    pub fn estimate_session_battery_impact(
        env: &Env,
        session_id: &String,
        operations_count: u32,
        sync_count: u32,
        cache_operations: u32,
    ) -> BatteryImpactReport {
        let base_drain = 1u32; // 1% base per session
        let ops_drain = operations_count / 5;
        let sync_drain = sync_count / 3;
        let cache_drain = cache_operations / 10;
        let network_drain = (operations_count + sync_count) / 4;

        let total_drain = base_drain + ops_drain + sync_drain + cache_drain + network_drain;
        let total_drain = if total_drain > 100 { 100 } else { total_drain };

        let mut recommendations = Vec::new(env);
        if operations_count > 20 {
            recommendations.push_back(String::from_str(
                env,
                "Batch operations to reduce battery usage",
            ));
        }
        if sync_count > 10 {
            recommendations.push_back(String::from_str(
                env,
                "Reduce sync frequency to save battery",
            ));
        }
        if cache_operations > 50 {
            recommendations.push_back(String::from_str(
                env,
                "Optimize cache strategy for lower battery impact",
            ));
        }
        if total_drain > 15 {
            recommendations.push_back(String::from_str(env, "Consider enabling power saver mode"));
        }

        BatteryImpactReport {
            session_id: session_id.clone(),
            estimated_drain_percent: total_drain,
            operations_count,
            sync_count,
            cache_operations,
            network_calls: operations_count + sync_count,
            recommendations,
        }
    }

    pub fn update_battery_config(
        env: &Env,
        user: &Address,
        config: BatteryOptimizationConfig,
    ) -> Result<(), MobileOptimizerError> {
        env.storage()
            .persistent()
            .set(&DataKey::BatteryConfig(user.clone()), &config);
        Ok(())
    }

    fn determine_power_mode(
        battery_level: u32,
        is_charging: bool,
        config: &BatteryOptimizationConfig,
    ) -> PowerMode {
        if is_charging {
            return PowerMode::Normal;
        }
        if !config.auto_power_saver {
            return PowerMode::Normal;
        }
        if battery_level <= config.critical_battery_threshold {
            PowerMode::UltraSaver
        } else if battery_level <= config.low_battery_threshold {
            PowerMode::PowerSaver
        } else {
            PowerMode::Adaptive
        }
    }

    fn estimate_runtime(battery_level: u32, power_mode: &PowerMode) -> u32 {
        let base_minutes_per_percent: u32 = match power_mode {
            PowerMode::UltraSaver => 8,
            PowerMode::PowerSaver => 6,
            PowerMode::Adaptive => 4,
            PowerMode::Normal => 3,
            PowerMode::Performance => 2,
        };
        battery_level * base_minutes_per_percent
    }

    fn battery_key(_env: &Env, device_id: &String) -> String {
        device_id.clone()
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatteryOptimizedSettings {
    pub sync_interval_seconds: u64,
    pub prefetch_enabled: bool,
    pub animation_enabled: bool,
    pub background_sync: bool,
    pub max_concurrent_ops: u32,
    pub cache_aggressiveness: u32,
    pub reduce_image_quality: bool,
    pub disable_video_autoplay: bool,
}
