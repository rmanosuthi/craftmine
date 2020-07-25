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

pub struct AsyncNetInstance {
    pub rt_handle: std::thread::JoinHandle<()>,
    pub ani_send: tokio::sync::mpsc::Sender<NetSendMsg>,
    pub ani_recv: crossbeam::Receiver<NetRecvMsg>,
    pub signal_shutdown: tokio::sync::mpsc::UnboundedSender<u64>,
    cc_ptr: &'static ConfigCollection
}

pub enum NetSendMsg {}
pub enum NetRecvMsg {}

impl AsyncNetInstance {
    pub fn new(vf: ValidatedInitFlags, cc: ConfigCollection) -> AsyncNetInstance {
        let mut rt = tokio::runtime::Builder::new().basic_scheduler().enable_all().build().unwrap();
        let (ani_send, mut async_recv) = tokio::sync::mpsc::channel(cc.net.sync_async_channel_len);
        let (shutdown_send, mut shutdown) = tokio::sync::mpsc::unbounded_channel::<u64>();
        let (async_send, ani_recv) = crossbeam::unbounded();
        let vf = vf.clone();
        let cc = Box::leak(Box::new(cc)) as &'static ConfigCollection;
        let rt_handle = std::thread::spawn(move || {
            rt.block_on(async move {
                let mut async_recv = async_recv;
                let listen_bind = format!("{}:{}", &vf.bind_addr.0, &vf.je_port.0);
                let mut listener = tokio::net::TcpListener::bind(&listen_bind).await.unwrap();
                info!("Listening on {}", &listen_bind);
                //let mut streams = HashMap::new();
                loop {
                    tokio::select! {
                        net_msg = async_recv.recv() => {},
                        Ok((stream, addr)) = listener.accept() => {
                            //streams.insert(addr, stream);
                            tokio::task::spawn(async move {
                                // TODO timeout
                                let mut je_client = stream;
                                let mut addr = addr;
                                let mut state: i32 = 0;
                                let mut last_seen = tokio::time::Instant::now();
                                info!("New JE client from {}", &addr);
                                let mut peek_buf = [0u8; 1];
                                'streamloop: loop {
                                    tokio::select! {
                                        je_res = read_from_je(&mut je_client) => {
                                            match je_res {
                                                Ok((packet_len, packet_id, packet_data)) => {
                                                    debug!("D: @{} IN P\nlen {} id {} DATA\n\t{:?}\n", &addr, &packet_len, &packet_id, &packet_data);
                                                    match (state, packet_id) {
                                                        (0, 0) => {
                                                            match JePacketHandshake::try_from(packet_data.as_slice()) {
                                                                Ok(p) => {},
                                                                Err(e) => {
                                                                    // TODO skip whole packet len bytes
                                                                    warn!("Invalid handshake packet with state 0");
                                                                    if (&cc).net.kick_invalid_packet {
                                                                        je_kick(&mut je_client, e);
                                                                        break 'streamloop;
                                                                    }
                                                                }
                                                            }
                                                        },
                                                        (1, 0) => {
                                                            if packet_len == 1 {
                                                                // query status
                                                            }
                                                        },
                                                        (ivs, ivpid) => {
                                                            warn!("Invalid state {} and packet id {}", ivs, ivpid);
                                                            if (&cc).net.kick_invalid_packet {
                                                                je_kick(&mut je_client, format!("Invalid state {} and packet id {}", ivs, ivpid).into());
                                                                break 'streamloop;
                                                            }
                                                        }
                                                    }
                                                },
                                                Err(e) => {
                                                    if e.is::<std::io::Error>() {
                                                        debug!("D: @{} SHUTDOWN", &addr);
                                                        debug!("==========");
                                                        je_client.shutdown(Shutdown::Both);
                                                    } else {
                                                        match e.description() {
                                                            "VARINTERR" => {
                                                                debug!("DE: @{} VARINTERR", &addr);
                                                            },
                                                            "empty" => {
                                                                debug!("D: @{} EMPTY", &addr);
                                                            },
                                                            o => {
                                                                debug!("DE: other error {}", o);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    };
                                }
                            });
                        },
                        done = shutdown.recv() => {
                            unsafe {
                                drop(Box::from_raw(
                                    std::mem::transmute::<_, *mut ConfigCollection>(cc)
                                ));
                            }
                            info!("Async network runtime shutting down");
                            break;
                        }
                    };
                }
            });
        });
        AsyncNetInstance {
            rt_handle,
            ani_send,
            ani_recv,
            cc_ptr: cc,
            signal_shutdown: shutdown_send
        }
    }
}

// TODO make sure async runtime's done before freeing cc
impl Drop for AsyncNetInstance {
    fn drop(&mut self) {
        std::thread::sleep_ms(1000);
        self.signal_shutdown.send(0);
    }
}