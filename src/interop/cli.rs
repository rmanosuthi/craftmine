use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum CliMessageInbound {}

#[derive(Serialize, Deserialize, Debug)]
pub enum CliMessageOutbound {}