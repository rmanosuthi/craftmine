use std::path::{Path, PathBuf};
use std::env::current_dir;
use structopt::StructOpt;
use crate::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "craftmine", about = "CraftMine CLI")]
/// Init flags.
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

/// Validated init flags.
/// Values are "validated" in the sense that they are present. They are not necessary *correct*, for example, a path may not point to an existing file.
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
}

impl ValidatedInitFlags {
    pub fn from_init(init_flags: &InitFlags) -> Result<ValidatedInitFlags, Vec<ServerInitError>> {
        let prefix = match init_flags.prefix {
            Some(pfx) => (pfx, vec![]),
            None => {
                let current_dir = current_dir().unwrap();
                // use current dir as prefix
                // add warning
                (current_dir.clone(), vec![
                    format!("Using current dir {:?} as prefix", &current_dir)
                ])
            }
        };
        Ok(ValidatedInitFlags {
            prefix,
            config_path: match init_flags.config_path {
                Some(cfg_path) => (cfg_path, vec![]),
                None => {
                    let def_path = [
                        &prefix.0.as_os_str().to_str().unwrap(),
                        "config"
                    ].iter().collect();
                    (def_path, vec![
                        format!("Using default path {:?}", &def_path)
                    ])
                }
            },
            je_port: match init_flags.je_port {
                Some(je_port) => (je_port, vec![]),
                None => (25565, vec!["Using default port 25565".to_owned()])
            },
            be_port: match init_flags.be_port {
                Some(be_port) => (be_port, vec![]),
                None => (9999, vec!["TODO implement BE".to_owned()])
            }
        })
    }
    pub fn try_create_prefix(&self) -> Vec<ServerInitError> {
        ServerPrefix::new_no_override(&self.prefix.0).1.iter().map(|tup| match tup.1 {
            Some(a) => Some(ServerInitError::PfxError(a)),
            None => None
        }).filter_map(|err| err).collect()
    }
}