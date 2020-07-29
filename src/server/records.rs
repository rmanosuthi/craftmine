use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct UserRecord {
    pub username: String,
    pub world: String,
    pub locality: LocalityRecord
}

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