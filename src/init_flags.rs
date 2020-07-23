use std::path::{Path, PathBuf};
use std::env::current_dir;
use structopt::StructOpt;
use crate::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "craftmine", about = "CraftMine CLI")]
pub(crate) struct InitFlags {
    #[structopt(parse(from_os_str), short = "pfx", long)]
    pub prefix: Option<PathBuf>,

    #[structopt(parse(from_os_str), short = "cfg", long)]
    pub config_path: Option<PathBuf>,

    #[structopt(long)]
    pub je_port: Option<u16>,

    #[structopt(long)]
    pub be_port: Option<u16>,

    #[structopt(short = "d")]
    pub daemon: bool
}

pub(crate) struct ValidatedInitFlags {
    pub prefix: (PathBuf, Vec<String>),
    pub config_path: (PathBuf, Vec<String>),
    pub je_port: (u16, Vec<String>),
    pub be_port: (u16, Vec<String>)
}

impl InitFlags {
    pub fn to_validated(&self) -> Result<ValidatedInitFlags, Vec<ServerInitError>> {
        ValidatedInitFlags::from_init(&self)
    }
    pub fn try_create_prefix(&self) -> Vec<ServerInitError> {
        match current_dir() {
            Ok(path) => {
                // new prefix here
                ServerPrefix::new_no_override(&path).1.iter().map(|tup| match tup.1 {
                    Some(a) => Some(ServerInitError::PfxError(a)),
                    None => None
                }).collect()
            },
            Err(e) => vec![Some(ServerInitError::PfxIoError(Box::new(e)))]
        }
    }
}

impl ValidatedInitFlags {
    pub fn from_init(init_flags: &InitFlags) -> Result<ValidatedInitFlags, Vec<ServerInitError>> {
        let prefix_path = init_flags.prefix.unwrap_or(current_dir().unwrap());
        let mut warns = Vec::new();
        Ok(ValidatedInitFlags {
            prefix: match init_flags.prefix {
                Some(pfx) => {},
                None => {
                    let current_dir = current_dir().unwrap();
                    // use current dir as prefix
                    // add warning
                    (current_dir.clone(), vec![
                        format!("Using current dir {:?} as prefix", &current_dir)
                    ])
                }
            },
            config_path: init_flags.config_path.unwrap_or([
                &prefix_path,
                "config.toml"
            ].iter().collect()),
            je_port: 
        })
    }
}