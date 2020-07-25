use std::path::{Path, PathBuf};
use std::env::current_dir;
use structopt::StructOpt;
use crate::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "craftmine", about = "CraftMine CLI")]
/// Init flags.
pub struct InitFlags {
    #[structopt(parse(from_os_str), short = "pfx", long)]
    pub prefix: Option<PathBuf>,

    #[structopt(parse(from_os_str), short = "cfg", long)]
    pub config_path: Option<PathBuf>,

    #[structopt(long)]
    pub je_port: Option<u16>,

    #[structopt(long)]
    pub be_port: Option<u16>,

    #[structopt(short = "d")]
    pub daemon: bool,

    #[structopt(long)]
    pub bind_addr: Option<String>
}

/// Validated init flags.
/// Values are "validated" in the sense that they are present. They are not necessary *correct*, for example, a path may not point to an existing file.
#[derive(Clone)]
pub struct ValidatedInitFlags {
    pub prefix: (PathBuf, Vec<String>),
    pub config_path: (PathBuf, Vec<String>),
    pub je_port: (u16, Vec<String>),
    pub be_port: (u16, Vec<String>),
    pub bind_addr: (String, Vec<String>)
}

impl InitFlags {
    pub fn to_validated(&self) -> Result<ValidatedInitFlags, Vec<String>> {
        ValidatedInitFlags::from_init(&self)
    }
}

impl ValidatedInitFlags {
    pub fn from_init(init_flags: &InitFlags) -> Result<ValidatedInitFlags, Vec<String>> {
        let prefix = match &init_flags.prefix {
            Some(pfx) => (pfx.clone(), vec![]),
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
            prefix: prefix.clone(),
            config_path: match &init_flags.config_path {
                Some(cfg_path) => (cfg_path.to_owned(), vec![]),
                None => {
                    let def_path = [
                        &prefix.0.as_os_str().to_str().unwrap(),
                        "config"
                    ].iter().collect::<PathBuf>();
                    (def_path.to_owned(), vec![
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
            },
            bind_addr: match &init_flags.bind_addr {
                Some(addr) => (addr.to_owned(), vec![]),
                None => ("0.0.0.0".to_owned(), vec![
                    "Using default bind address 0.0.0.0".to_owned()
                ])
            }
        })
    }
    pub fn try_create_prefix(&self) -> Vec<String> {
        ServerPrefix::new_no_override(&self.prefix.0).1.iter().map(|tup| match &tup.1 {
            Some(a) => Some(a.to_string()),
            None => None
        }).filter_map(|err| err).collect()
    }
}