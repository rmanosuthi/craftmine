use std::path::{Path, PathBuf};
use std::error::Error;
use crate::server::config::*;
use toml::to_vec;

pub struct ServerPrefix(pub PathBuf);

impl ServerPrefix {
    /// Create a new prefix.
    pub fn new_no_override(path: &Path) -> (ServerPrefix, Vec<(PathBuf, Option<ServerPrefixError>)>) {
        (
            ServerPrefix(path.to_owned()),
            ServerPrefix::init_folders(&path)
        )
    }
    /// Load an already existing prefix.
    pub fn from(path: &Path) -> (ServerPrefix, Vec<(PathBuf, Option<ServerPrefixError>)>) {
        (
            ServerPrefix(path.to_owned()),
            unimplemented!()
        )
    }
    fn init_folders(path: &Path) -> Vec<(PathBuf, Option<ServerPrefixError>)> {
        [
            DirOrFile::Dir("worlds", vec![]),
            DirOrFile::Dir("config", vec![
                DirOrFile::File("init.toml", toml::to_vec(&ConfigInit::default())),
                DirOrFile::File("network.toml", toml::to_vec(&ConfigNetwork::default())),
                DirOrFile::File("performance.toml", toml::to_vec(&ConfigPerformance::default())),
                DirOrFile::File("experimental.toml", toml::to_vec(&ConfigExperimental::default())),
                DirOrFile::File("auth.toml", toml::to_vec(&ConfigAuth::default())),
                DirOrFile::File("cap.toml", toml::to_vec(&ConfigCap::default())),
            ]),
            DirOrFile::Dir("users", vec![])
        ].iter().map(|dir_or_file| {
            let mut local_result = Vec::new();
            dir_or_file.recursive_write(path, &mut local_result);
            local_result
        }).flatten().collect()
    }
    pub fn check_prefix(path: &Path) -> Vec<ServerPrefixError> {
        unimplemented!()
    }
}

enum DirOrFile<'a> {
    Dir(&'a str, Vec<DirOrFile<'a>>),
    EmptyFile(&'a str),
    File(&'a str, Result<Vec<u8>, toml::ser::Error>)
}

impl<'a> DirOrFile<'a> {
    pub fn recursive_write(&self, full_path: &Path, result: &mut Vec<(PathBuf, Option<ServerPrefixError>)>) {
        match self {
            DirOrFile::Dir(dir_name, children) => {
                let path: PathBuf = [full_path, &PathBuf::from(dir_name)].iter().collect();
                result.push((path.clone(), match std::fs::create_dir(&path) {
                    Ok(_) => None,
                    Err(e) => Some(ServerPrefixError::IoError(e))
                }));
                children.iter().for_each(|child| {
                    child.recursive_write(&path, result);
                });
                return;
            },
            DirOrFile::EmptyFile(empty_file) => {
                let path: PathBuf = [full_path, &PathBuf::from(empty_file)].iter().collect();
                result.push((path.clone(), match std::fs::write(path, vec![]) {
                    Ok(_) => None,
                    Err(e) => Some(ServerPrefixError::IoError(e))
                }));
                return;
            },
            DirOrFile::File(file_name, content_maybe) => {
                let path: PathBuf = [full_path, &PathBuf::from(file_name)].iter().collect();
                match content_maybe {
                    Ok(content) => {
                        result.push((path.clone(), match std::fs::write(path, content) {
                            Ok(_) => None,
                            Err(e) => Some(ServerPrefixError::IoError(e))
                        }));
                        return;
                    },
                    Err(e) => {
                        result.push((path.clone(), Some(ServerPrefixError::TomlError(e.clone()))));
                        return;
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ServerPrefixError {
    AlreadyExists,
    IoError(std::io::Error),
    TomlError(toml::ser::Error)
}

impl ToString for ServerPrefixError {
    fn to_string(&self) -> String {
        todo!()
    }
}