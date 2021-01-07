mod auth;
mod cap;
mod exp;
mod init;
mod net;
mod perf;

pub use self::auth::ConfigAuth;
pub use self::cap::ConfigCap;
pub use self::exp::ConfigExp;
pub use self::init::ConfigInit;
pub use self::net::ConfigNet;
pub use self::perf::ConfigPerf;

use std::{path::{Path, PathBuf}, error::Error};
use crate::init_flags::ValidatedInitFlags;
use serde::{Serialize, Deserialize};

pub enum ValidatorInfo {
    Info(String),
    Warn(String),
    Error(String, Box<dyn Error>)
}

pub trait ConfigValidator {
    fn validate(&self) -> Vec<ValidatorInfo>;
}

/// Collection of configs used in the server.
/// A restart is required for the changes to take effect,
/// the reason being access is frequent and synchronization incurs too much performance penalty.
#[derive(Clone)]
pub struct ConfigCollection {
    pub auth: ConfigAuth,
    pub cap: ConfigCap,
    pub exp: ConfigExp,
    pub init: ConfigInit,
    pub net: ConfigNet,
    pub perf: ConfigPerf
}

pub struct ConfigFolder(pub PathBuf);

impl ConfigFolder {
    pub fn load_or_new_all(vf: &ValidatedInitFlags) -> Result<ConfigCollection, Vec<String>> {
        let mut errors = vec![];
        let config_path = &vf.config_path.0;
        std::fs::create_dir(&config_path);
        let auth = match ConfigAuth::load_or_new(config_path) {
            Ok(c) => Some(c),
            Err(e) => {
                ConfigAuth::push_error(&mut errors, config_path, e);
                None
            }
        };
        let cap = match ConfigCap::load_or_new(config_path) {
            Ok(c) => Some(c),
            Err(e) => {
                ConfigCap::push_error(&mut errors, config_path, e);
                None
            }
        };
        let exp = match ConfigExp::load_or_new(config_path) {
            Ok(c) => Some(c),
            Err(e) => {
                ConfigExp::push_error(&mut errors, config_path, e);
                None
            }
        };
        let init = match ConfigInit::load_or_new(config_path) {
            Ok(c) => Some(c),
            Err(e) => {
                ConfigInit::push_error(&mut errors, config_path, e);
                None
            }
        };
        let net = match ConfigNet::load_or_new(config_path) {
            Ok(c) => Some(c),
            Err(e) => {
                ConfigNet::push_error(&mut errors, config_path, e);
                None
            }
        };
        let perf = match ConfigPerf::load_or_new(config_path) {
            Ok(c) => Some(c),
            Err(e) => {
                ConfigPerf::push_error(&mut errors, config_path, e);
                None
            }
        };
        if errors.len() == 0 {
            info!("Config collection looks good");
            Ok(ConfigCollection {
                auth: auth.unwrap(), cap: cap.unwrap(), exp: exp.unwrap(), init: init.unwrap(), net: net.unwrap(), perf: perf.unwrap()
            })
        } else {
            Err(errors)
        }
    }
}

pub trait ConfigFile: Sized + serde::ser::Serialize + serde::de::DeserializeOwned + Default {
    fn load_or_new(folder_path: &Path) -> Result<Self, Box<dyn Error>> {
        let filename = Self::get_path(folder_path);
        match std::fs::File::open(&filename) {
            Ok(mut file) => {
                let mut rdr = std::io::BufReader::new(file);
                serde_json::from_reader(rdr).map_err(|e| e.into())
            },
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    let def = Self::default();
                    let mut file = std::fs::File::create(&filename)?;
                    let mut writer = std::io::BufWriter::new(file);
                    warn!("Creating new {} at {:?}", Self::get_filename(), &filename);
                    serde_json::to_writer(writer, &def)?;
                    Ok(def)
                } else {
                    Err(e.into())
                }
            }
        }
    }
    fn get_path(folder_path: &Path) -> PathBuf {
        let mut filename = PathBuf::from(folder_path);
        filename.push(Self::get_filename());
        filename
    }
    fn push_error(errors: &mut Vec<String>, folder_path: &Path, err: Box<dyn Error>) {
        errors.push(
            format!("Failed to load or create {} at {:?}\n{:?}", 
                Self::get_filename(),
                Self::get_path(&folder_path),
                err
            )
        );
    }
    fn get_filename() -> &'static str;
}