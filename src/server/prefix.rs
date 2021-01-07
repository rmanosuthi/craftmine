use crate::imports::*;
use crate::server::symbols::*;

use super::Snapshot;

#[derive(Debug, Clone)]
pub struct ServerPrefix {
    pub path: PathBuf,
    pub users: UserPrefix,
    pub worlds: WorldPrefix
}

impl UserPrefix {
    pub fn load_or_new_online(&self, uuid: &Uuid, username: &str, cc: &ConfigCollection, puid: &mut PersistUuidAllocator, default_w_props: &WorldProperty) -> Result<UserRecord, ()> {
        let mut path = self.0.clone();
        path.push(format!(
            "{}.json",
            uuid.to_string()
        ));
        let world_name = cc.auth.default_world_name.to_owned();
        UserRecord::load_or_new(&path, UserRecord {
            username: username.to_owned(),
            world: world_name.clone(),
            locality: default_w_props.default_spawn,
            uuid: Some(uuid.to_owned()),
            online: true,
            gamemode: default_w_props.default_gamemode,
            persist_id: puid.new_online_user(&uuid)
        })
    }
    pub fn load_or_new_offline(&self, username: &str, cc: &ConfigCollection, puid: &mut PersistUuidAllocator, default_w_props: &WorldProperty) -> Result<UserRecord, ()> {
        let mut path = self.0.clone();
        
        let world_name = cc.auth.default_world_name;
        let mut def = UserRecord {
            username: username.to_owned(),
            world: world_name.clone(),
            locality: default_w_props.default_spawn,
            uuid: None,
            online: true,
            gamemode: default_w_props.default_gamemode,
            persist_id: puid.new_offline_user(&username)
        };
        warn!("Offline user {} has a persist UUID (v5, namespace OID, SHA-1 of username) of {}", username, &def.uuid);
        path.push(format!(
            "{}.json",
            def.uuid.to_string()
        ));
        UserRecord::load_or_new(&path, def)
    }
}

pub trait LoadOrNew: serde::ser::Serialize + serde::de::DeserializeOwned {
    fn load_or_new(path: &Path, default: Self) -> Result<Self, ()> {
        debug!("LOAD_OR_NEW {:?}", &path);
        match std::fs::OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(&path) {
            Ok(file) => {
                // file exists, open ok
                let reader = std::io::BufReader::new(file);
                serde_json::from_reader(reader).map_err(|_| ())
            },
            Err(e) => {
                debug!("USER_ACCESS ERROR {:?}", e.kind());
                match e.kind() {
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidInput => {
                        let file = std::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(&path).map_err(|_| ())?;
                        let writer = std::io::BufWriter::new(file);
                        serde_json::to_writer_pretty(writer, &default).map_err(|_| ())?;
                        Ok(default)
                    },
                    _ => Err(())
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserPrefix(pub PathBuf);

impl UserPrefix {
    pub fn from(p: &Path) -> Self {
        let mut path = p.to_owned();
        path.push("users");
        Self(path)
    }
    pub fn online_path(&self) -> PathBuf {
        [self.0, "online".into()].iter().collect()
    }

    pub fn offline_path(&self) -> PathBuf {
        [self.0, "offline".into()].iter().collect()
    }
}

#[derive(Debug, Clone)]
pub struct WorldPrefix(pub PathBuf);

impl WorldPrefix {
    pub fn from(p: &Path) -> Self {
        let mut path = p.to_owned();
        path.push("worlds");
        Self(path)
    }
    pub fn snapshot_props(&self) -> Snapshot<WorldProperty> {
        unimplemented!()
    }
    /// Get the **on-disk properties** of a world.
    pub fn snapshot_name(world_name: &str) -> Option<Snapshot<WorldProperty>> {
        unimplemented!()
    }
}

impl ServerPrefix {
    /// Load a prefix or create a new one.
    pub fn load_or_new(path: &Path) -> (ServerPrefix, Vec<(PathBuf, Option<ServerPrefixError>)>) {
        (
            ServerPrefix {
                path: path.to_owned(),
                users: UserPrefix::from(path),
                worlds: WorldPrefix::from(path)
            },
            ServerPrefix::init_folders(&path)
        )
    }
    fn init_folders(path: &Path) -> Vec<(PathBuf, Option<ServerPrefixError>)> {
        [
            DirOrFile::Dir("", vec![]),
            DirOrFile::Dir("worlds", vec![]),
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
    pub fn read_or_new_user(&self, uuid: &Uuid) -> Result<UserRecord, ServerPrefixError> {
        unimplemented!()
    }
}

enum DirOrFile<'a> {
    Dir(&'a str, Vec<DirOrFile<'a>>),
    EmptyFile(&'a str),
    File(&'a str, Result<Vec<u8>, Box<dyn Error>>)
}

impl<'a> DirOrFile<'a> {
    pub fn recursive_write(&self, full_path: &Path, result: &mut Vec<(PathBuf, Option<ServerPrefixError>)>) {
        match self {
            DirOrFile::Dir(dir_name, children) => {
                let path: PathBuf = [full_path, &PathBuf::from(dir_name)].iter().collect();
                result.push((path.clone(), match std::fs::create_dir(&path) {
                    Ok(_) => None,
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::AlreadyExists => None,
                        _ => Some(ServerPrefixError::IoError(e))
                    }
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
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::AlreadyExists => None,
                        _ => Some(ServerPrefixError::IoError(e))
                    }
                }));
                return;
            },
            DirOrFile::File(file_name, content_maybe) => {
                let path: PathBuf = [full_path, &PathBuf::from(file_name)].iter().collect();
                match content_maybe {
                    Ok(content) => {
                        result.push((path.clone(), match std::fs::write(path, content) {
                            Ok(_) => None,
                            Err(e) => match e.kind() {
                                std::io::ErrorKind::AlreadyExists => None,
                                _ => Some(ServerPrefixError::IoError(e))
                            }
                        }));
                        return;
                    },
                    Err(e) => {
                        result.push((path.clone(),
                            Some(ServerPrefixError::GenericError(
                                e.to_string()
                            ))
                        ));
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
    GenericError(String)
}

impl ToString for ServerPrefixError {
    fn to_string(&self) -> String {
        match &self {
            ServerPrefixError::AlreadyExists => "already exists".to_owned(),
            ServerPrefixError::IoError(e) => format!("IO error - {:?}", e),
            ServerPrefixError::GenericError(e) => format!("Generic error - {}", e)
        }
    }
}