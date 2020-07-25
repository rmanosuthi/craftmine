use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ConfigNetwork {
    pub sync_async_channel_len: usize,
    pub web_addr_port: String
}

impl Default for ConfigNetwork {
    fn default() -> Self {
        todo!()
    }
}