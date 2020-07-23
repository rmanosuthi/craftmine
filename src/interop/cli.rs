use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum CliMessageInbound {}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum CliMessageOutbound {}