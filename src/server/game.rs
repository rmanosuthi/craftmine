use crate::*;
use super::ServerPrefix;
use hashbrown::HashMap;
use uuid::Uuid;
use config::ConfigCollection;
use std::{net::Shutdown, pin::Pin, time::Duration, convert::TryFrom};
use tokio::prelude::*;
use server::net::*;

pub struct GameServer {
    pub prefix: ServerPrefix,
    pub init_flags: ValidatedInitFlags,
    pub worlds: HashMap<Uuid, GameWorld>,
    pub cli_recv: crossbeam::Receiver<CliMessageOutbound>,
    pub cli_send: crossbeam::Sender<CliMessageInbound>,
    pub async_net_instance: AsyncNetInstance,
    pub cc: ConfigCollection
}

pub struct GameWorld {}

impl GameServer {
}

pub enum NetSendMsg {}
pub enum NetRecvMsg {}