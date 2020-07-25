use serde::{Serialize, Deserialize};
use super::ConfigFile;

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigInit {

}

impl Default for ConfigInit {
    fn default() -> Self {
        Self {}
    }
}

impl ConfigFile for ConfigInit {
    fn get_filename() -> &'static str {
        "init.json"
    }
}