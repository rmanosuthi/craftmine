use serde::{Serialize, Deserialize};
use super::ConfigFile;

#[derive(Clone, Serialize, Deserialize)]
/// Experimental options based on Mumbo's video. Defaults to vanilla Minecraft behavior.
pub struct ConfigExp {
    /// Whether interacting with a bed will set the respawn point.
    pub bed_interact_set_respawn_point: bool,
    /// Whether sneaking on magma blocks will damage the player.
    pub sneak_on_magma_damage: bool,
    /// Limit slime blocks' push limit
    pub slime_push_limit: Option<u64>,
    /// Whether player has to be sneaking to rotate an item frame's content.
    pub player_sneak_rotate_item_frame_content: bool,
    /// Whether double tapping the sneak key will toggle sneak, the same way sprint does.
    pub double_tap_sneak: bool,
    /// Max threshold to notify player about low elytra durability.
    pub elytra_notify_ceiling: Option<u64>,
    /// Whether hoppers can load jukeboxes with discs.
    pub hopper_load_jukebox: bool,
    /// Maximum amount of maps that will be modified when blocks in the world are modified.
    /// Recommended set to off.
    pub world_maps_max_update: Option<u64>,
    /// Whether redstone can be placed on pistons.
    pub redstone_on_piston: bool,
    /// Whether beacons have an additional effect of preventing mob spawns for a configurable radius.
    pub beacon_deny_mob_spawn_radius: Option<u64>,
    /// Whether containers can be pushed by pistons.
    pub piston_push_containers: bool
}

impl Default for ConfigExp {
    fn default() -> Self {
        Self {
            bed_interact_set_respawn_point: false,
            sneak_on_magma_damage: false,
            slime_push_limit: Some(8),
            player_sneak_rotate_item_frame_content: false,
            double_tap_sneak: false,
            elytra_notify_ceiling: None,
            hopper_load_jukebox: false,
            world_maps_max_update: None,
            redstone_on_piston: false,
            beacon_deny_mob_spawn_radius: None,
            piston_push_containers: false
        }
    }
}

impl ConfigFile for ConfigExp {
    fn get_filename() -> &'static str {
        "experimental.json"
    }
}