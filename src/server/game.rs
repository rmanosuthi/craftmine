use crate::*;
use super::ServerPrefix;
use hashbrown::HashMap;
use uuid::Uuid;
use config::ConfigCollection;
use std::{net::Shutdown, pin::Pin, time::{Duration, Instant}, convert::TryFrom};
use tokio::prelude::*;
use server::net::*;

pub struct GameServer {
    pub prefix: ServerPrefix,
    pub init_flags: ValidatedInitFlags,
    pub worlds: HashMap<Uuid, GameWorld>,
    pub cli_recv: crossbeam::Receiver<CliMessageOutbound>,
    pub cli_send: crossbeam::Sender<CliMessageInbound>,
    pub async_net_instance: AsyncNetInstance,
    pub cc: ConfigCollection,
    pub tick: crossbeam::Receiver<Instant>,
    pub recv_status: crossbeam::Receiver<ServerStatus>,
    pub send_status: crossbeam::Sender<ServerStatus>,
    pub users: HashMap<Uuid, UserRecord>
}

pub struct GameWorld {}

impl GameServer {
    pub fn run(&mut self) {
        info!("Starting game server");
        let mut run = true;
        while run {
            crossbeam::select! {
                recv(self.recv_status) -> status => {
                    debug!("Got status {:?}", status.unwrap());
                    match status.unwrap() {
                        ServerStatus::Stop => {
                            self.stop();
                            run = false;
                        },
                        ServerStatus::Pause => {
                            info!("Pausing game server");
                            match self.recv_status.recv() {
                                Ok(ServerStatus::Stop) => {
                                    self.stop();
                                    run = false;
                                },
                                Ok(ServerStatus::Start) => {
                                    info!("Resuming game server");
                                },
                                Ok(_) => {},
                                Err(_) => panic!()
                            }
                        },
                        ServerStatus::Start => {
                            run = true;
                        }
                    }
                }
                recv(self.tick) -> _ => {
                    while let Ok(inc_net_packet) = self.async_net_instance.ani_recv.recv() {
                        debug!("{:?}", inc_net_packet);
                        match inc_net_packet.inner {
                            NetRecvInner::NewSession {username: u} => {
                                info!("{} ({}) has joined the server.", &u, &inc_net_packet.uuid);
                                if let Ok(user) = self.prefix.users.load_or_new(&inc_net_packet.uuid) {
                                    self.users.insert(inc_net_packet.uuid.clone(), user);
                                } else {
                                    error!("Failed to load or create user record for {}, disconnecting player", &u);
                                    self.async_net_instance.ani_send();
                                }
                            },
                            NetRecvInner::EndSession => {
                                match self.users.get(&inc_net_packet.uuid) {
                                    Some(u) => {
                                        info!("{} ({}) has left.", u.username, &inc_net_packet.uuid);
                                    },
                                    None => {
                                        warn!("Potentially inconsistent uuid-user map");
                                    }
                                }
                            },
                            _ => ()
                        }
                    }
                }
            }
        }
    }
    pub fn stop(&mut self) {
        self.send_status.send(ServerStatus::Stop);
    }
}

#[derive(Debug)]
pub enum NetSendMsg {
    All(i32, Vec<u8>),
    Broadcast(Vec<Uuid>, i32, Vec<u8>),
    Single(Uuid, i32, Vec<u8>),
    Disconnect(Uuid, String),
    SetTimeout(Uuid, Instant, Instant, String),
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