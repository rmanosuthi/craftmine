use serde::{Serialize, Deserialize};
use super::ConfigFile;

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigNet {
    pub sync_async_channel_len: usize,
    pub web_addr_port: String,
    pub kick_invalid_packet: bool,
    pub server_name: String,
    pub server_description: String
}

impl Default for ConfigNet {
    fn default() -> Self {
        Self {
            sync_async_channel_len: 10000,
            web_addr_port: "127.0.0.1:8080".to_owned(),
            kick_invalid_packet: false,
            server_name: crate::SERVER_RELNAME.to_owned(),
            server_description: format!("A CraftMine server ({})", crate::SERVER_RELNAME)
        }
    }
}

impl ConfigFile for ConfigNet {
    fn get_filename() -> &'static str {
        "network.json"
    }
}