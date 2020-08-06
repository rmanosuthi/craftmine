use crate::imports::*;
use crate::server::symbols::*;

#[derive(Debug)]
pub enum NetSendMsg {
    All(i32, Vec<u8>),
    Broadcast(Vec<Uuid>, i32, Vec<u8>),
    Single(Uuid, i32, Vec<u8>),
    Disconnect(Uuid, String),
    DefiniteTimeout(Uuid, Duration, String),
    IndefiniteTimeout(Uuid, String),
    UnsetTimeout(Uuid),
    SetBlock(Uuid, Instant, String),
    UnsetBlock(Uuid)
}

#[derive(Debug)]
pub struct NetRecvMsg {
    pub uuid: Uuid,
    pub inner: NetRecvInner
}

#[derive(Debug)]
pub enum NetRecvInner {
    NewSession {
        username: String
    },
    EndSession,
    Packet {
        id: i32,
        data: Vec<u8>
    }
}