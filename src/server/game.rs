use crate::*;
use super::ServerPrefix;
use hashbrown::HashMap;
use uuid::Uuid;
use config::ConfigCollection;
use std::{net::Shutdown, pin::Pin};
use tokio::prelude::*;
use server::net::read_from_je;

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

pub struct AsyncNetInstance {
    pub rt_handle: std::thread::JoinHandle<()>,
    pub ani_send: tokio::sync::mpsc::Sender<NetSendMsg>,
    pub ani_recv: crossbeam::Receiver<NetRecvMsg>
}

pub enum NetSendMsg {}
pub enum NetRecvMsg {}

impl AsyncNetInstance {
    pub fn new(vf: ValidatedInitFlags, cc: &ConfigCollection) -> AsyncNetInstance {
        let mut rt = tokio::runtime::Builder::new().basic_scheduler().build().unwrap();
        let (ani_send, mut async_recv) = tokio::sync::mpsc::channel(cc.network.sync_async_channel_len);
        let (shutdown_send, shutdown) = tokio::sync::oneshot::channel::<u64>();
        let (async_send, ani_recv) = crossbeam::unbounded();
        let vf = vf.clone();
        let rt_handle = std::thread::spawn(move || {
            rt.block_on(async move {
                let mut async_recv = async_recv;
                let mut listener = tokio::net::TcpListener::bind(format!("{}:{}", &vf.bind_addr.0, &vf.be_port.0)).await.unwrap();
                //let mut streams = HashMap::new();
                loop {
                    tokio::select! {
                        net_msg = async_recv.recv() => {},
                        Ok((stream, addr)) = listener.accept() => {
                            //streams.insert(addr, stream);
                            tokio::task::spawn(async move {
                                let mut je_client = stream;
                                let mut addr = addr;
                                println!("New JE client from {}", &addr);
                                while let Ok((packet_len, packet_id, packet_data)) = read_from_je(&mut je_client).await {
                                    todo!();
                                }
                                je_client.shutdown(Shutdown::Both);
                            });
                        },
                        /*done = shutdown => {
                            break;
                        }*/
                    };
                }
            });
        });
        AsyncNetInstance {
            rt_handle,
            ani_send,
            ani_recv
        }
    }
}