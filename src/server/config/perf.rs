use super::*;
use std::time::Duration;

use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigPerf {
    pub view_distance_chunks: u64,
    pub items_dropped_ttl: Duration,
    pub chunks_active_enable: bool,
    pub chunks_active_max: Option<u64>,
    pub spawn_active_keep: bool,
    pub smp_threads_tick: Option<u64>,
    pub chunks_pools: ConfigChunkPools,
    pub target_tick_s_f64: f64
}

impl Default for ConfigPerf {
    fn default() -> Self {
        Self {
            view_distance_chunks: 16,
            items_dropped_ttl: Duration::from_secs(300),
            chunks_active_enable: true,
            chunks_active_max: Some(32),
            spawn_active_keep: true,
            smp_threads_tick: None,
            chunks_pools: ConfigChunkPools::default(),
            target_tick_s_f64: 0.05f64
        }
    }
}

impl ConfigValidator for ConfigPerf {
    fn validate(&self) -> Vec<ValidatorInfo> {
        todo!()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigChunkPools {
    pub online_to_partial_scan_interval: Duration,
    pub online_to_partial_distance_max: u64,
    pub online_to_partial_move_delay: Duration,
    pub partial_keepalive_distance_max: u64,
    pub partial_to_suspended_move_delay: Duration,
    pub partial_culling_whitelist: Vec<PartialTickProcessingType>,
    pub suspended_ttl_max: Duration,
    pub suspended_to_offline_move_delay: Duration
}

impl Default for ConfigChunkPools {
    fn default() -> Self {
        Self {
            online_to_partial_scan_interval: Duration::from_secs(45),
            online_to_partial_distance_max: 8,
            online_to_partial_move_delay: Duration::from_secs(15),
            partial_keepalive_distance_max: 8,
            partial_to_suspended_move_delay: Duration::from_secs(120),
            partial_culling_whitelist: vec![
                PartialTickProcessingType::MobPassive,
                PartialTickProcessingType::MobHostile
            ],
            suspended_ttl_max: Duration::from_secs(60),
            suspended_to_offline_move_delay: Duration::from_secs(60)
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PartialTickProcessingType {
    MobPassive,
    MobNeutral,
    MobPet,
    MobHostile,
    Redstone
}

impl ConfigFile for ConfigPerf {
    fn get_filename() -> &'static str {
        "performance.json"
    }
}