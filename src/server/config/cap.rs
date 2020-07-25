use serde::{Serialize, Deserialize};
use super::ConfigFile;

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigCap {

}

impl Default for ConfigCap {
    fn default() -> Self {
        Self {}
    }
}

impl ConfigFile for ConfigCap {
    fn get_filename() -> &'static str {
        "cap.json"
    }
}