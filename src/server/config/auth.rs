use serde::{Serialize, Deserialize};
use super::ConfigFile;

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigAuth {
    pub online_mode: bool
}

impl Default for ConfigAuth {
    fn default() -> Self {
        Self {
            online_mode: true
        }
    }
}

impl ConfigFile for ConfigAuth {
    fn get_filename() -> &'static str {
        "auth.json"
    }
}