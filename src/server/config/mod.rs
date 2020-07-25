mod auth;
mod cap;
mod experimental;
mod init;
mod network;
mod performance;

pub use self::auth::*;
pub use self::cap::*;
pub use self::experimental::*;
pub use self::init::*;
pub use self::network::*;
pub use self::performance::*;

use std::{path::PathBuf, error::Error};
use crate::init_flags::ValidatedInitFlags;

pub enum ValidatorInfo {
    Info(String),
    Warn(String),
    Error(String, Box<dyn Error>)
}

pub trait ConfigValidator {
    fn validate(&self) -> Vec<ValidatorInfo>;
}

pub struct ConfigCollection {
    pub network: ConfigNetwork
}

pub struct ConfigFolder(pub PathBuf);

impl ConfigFolder {
    pub fn load_or_new_all(vf: &ValidatedInitFlags) -> Result<ConfigCollection, Vec<String>> {
        todo!()
    }
}