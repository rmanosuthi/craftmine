use serde::{Serialize, Deserialize};
use super::ConfigFile;

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigAuth {
    pub online_mode: bool,
    pub max_players: u8,
    pub default_world_name: String
}

impl Default for ConfigAuth {
    fn default() -> Self {
        Self {
            online_mode: true,
            max_players: 20,
            default_world_name: "overworld".to_owned()
        }
    }
}

impl ConfigFile for ConfigAuth {
    fn get_filename() -> &'static str {
        "auth.json"
    }
}