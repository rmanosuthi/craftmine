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

use std::error::Error;

pub enum ValidatorInfo {
    Info(String),
    Warn(String),
    Error(String, Box<dyn Error>)
}

pub trait ConfigValidator {
    fn validate(&self) -> Vec<ValidatorInfo>;
}