use serde::{Serialize, Deserialize, Serializer};
use crate::imports::*;
use crate::server::symbols::*;
use std::fmt::{Formatter, Display};

#[derive(Serialize, Deserialize, Clone)]
pub struct SerializedUserRecord {
    pub username: String,
    pub world: String,
    pub locality: LocalityRecord,
    pub uuid: Option<Uuid>,
    pub online: bool,
    pub gamemode: Gamemode,
    pub persist_id: PersistId
}

impl SerializedUserRecord {
    pub fn to_internal_copy(&self) -> InternalUserRecord {
        InternalUserRecord {
            username: self.username.to_owned(),
            world: self.world.to_owned(),
            locality: self.locality.to_owned(),
            online: self.online,
            gamemode: self.gamemode.to_owned(),
            persist_id: self.persist_id.to_owned()
        }
    }
}

#[derive(Clone)]
pub struct InternalUserRecord {
    pub username: String,
    pub world: String,
    pub locality: LocalityRecord,
    pub online: bool,
    pub gamemode: Gamemode,
    pub persist_id: PersistId
}

impl InternalUserRecord {
    pub fn to_serialized_copy(&self, uuid: Option<Uuid>) -> SerializedUserRecord {
        SerializedUserRecord {
            username: self.username.to_owned(),
            world: self.world.to_owned(),
            locality: self.locality.to_owned(),
            online: self.online,
            gamemode: self.gamemode.to_owned(),
            persist_id: self.persist_id.to_owned(),
            uuid: uuid
        }
    }
}

/*impl Default for UserRecord {
    fn default() -> Self {
        Self {
            username: "".to_owned(),
            world: "overworld".to_owned(),
            locality: LocalityRecord::default(),
            uuid: Uuid::default(),
            online: false,
            gamemode: Gamemode::Survival,
            persist_id: 
        }
    }
}*/

impl LoadOrNew for SerializedUserRecord {}

pub struct VarIntRecord(i32);
pub struct VarLongRecord(i32);

#[derive(Serialize, Deserialize, Clone)]
pub struct LocalityRecord {
    pub x: i32,
    pub y: i16,
    pub z: i32,
    pub yaw: u8,
    pub pitch: u8,
    pub head_pitch: u8,
    pub vel_x: i16,
    pub vel_y: i16,
    pub vel_z: i16
}

impl Default for LocalityRecord {
    fn default() -> Self {
        Self {
            x: 0,
            y: 64,
            z: 0,
            yaw: 0,
            pitch: 0,
            head_pitch: 0,
            vel_x: 0,
            vel_y: 0,
            vel_z: 0
        }
    }
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Copy)]
pub struct BlockId {pub maj: u16, pub min: u16}

/*impl Serialize for BlockId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!(
            "({}:{})", self.0, self.1
        ))
    }
}*/

/*impl Deserialize for BlockId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de> {

        }
}*/

impl Display for BlockId {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({}:{})", self.maj, self.min)
    }
}