use super::*;
use std::time::Duration;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct ConfigPerformance {
    pub view_distance_chunks: u64,
    pub items_dropped_gc: Option<Duration>,
    pub items_dropped_ttl: Duration,
    pub chunks_active_enable: bool,
    pub chunks_active_max: Option<u64>,
    pub spawn_active_keep: bool,
    pub smp_threads_tick: u64,
    pub chunks_pools: ConfigChunkPools
}

impl Default for ConfigPerformance {
    fn default() -> Self {
        todo!()
    }
}

impl ConfigValidator for ConfigPerformance {
    fn validate(&self) -> Vec<ValidatorInfo> {
        todo!()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ConfigChunkPools {
    online_to_partial_scan_interval: Duration,
    online_to_partial_distance_max: u64,
    online_to_partial_move_delay: Duration,
    partial_keepalive_distance_max: u64,
    partial_to_suspended_move_delay: Duration,
    partial_culling_whitelist: Vec<PartialTickProcessingType>,
    suspended_ttl_max: Duration,
    suspended_to_offline_move_delay: Duration
}

#[derive(Serialize, Deserialize)]
pub enum PartialTickProcessingType {
    MobPassive,
    MobNeutral,
    MobPet,
    MobHostile,
    Redstone
}