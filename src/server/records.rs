use serde::{Serialize, Deserialize};
use crate::imports::*;
use crate::server::symbols::*;

#[derive(Serialize, Deserialize)]
pub struct UserRecord {
    pub username: String,
    pub world: String,
    pub locality: LocalityRecord,
    pub uuid: Uuid,
    pub online: bool
}

impl Default for UserRecord {
    fn default() -> Self {
        Self {
            username: "".to_owned(),
            world: "overworld".to_owned(),
            locality: LocalityRecord::default(),
            uuid: Uuid::default(),
            online: false
        }
    }
}

impl LoadOrNew for UserRecord {}

pub struct VarIntRecord(i32);
pub struct VarLongRecord(i32);

#[derive(Serialize, Deserialize)]
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
pub struct BlockId(pub u16, pub u16);