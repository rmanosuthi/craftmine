use crate::*;
use tokio::io::{AsyncReadExt, AsyncBufRead, AsyncBufReadExt, BufStream};
use tokio::stream::{StreamExt, StreamMap};
use super::*;
use std::net::Shutdown;
use hashbrown::HashMap;
use tokio::future::poll_fn;
use std::sync::Arc;
use uuid::Uuid;

pub enum NetSendMsg {}

pub enum NetRecvMsg {}

pub struct AsyncNetInstance {
    pub rt_handle: std::thread::JoinHandle<()>,
    pub ani_send: tokio::sync::mpsc::Sender<NetSendMsg>,
    pub ani_recv: crossbeam::Receiver<NetRecvMsg>,
    pub signal_shutdown: tokio::sync::mpsc::UnboundedSender<u64>,
    cc_ptr: &'static ConfigCollection
}

impl AsyncNetInstance {
    pub fn new(vf: ValidatedInitFlags, cc: ConfigCollection) -> AsyncNetInstance {
        let mut rt = tokio::runtime::Builder::new().basic_scheduler().enable_all().build().unwrap();
        let (ani_send, mut async_recv) = tokio::sync::mpsc::channel(cc.net.sync_async_channel_len);
        let (shutdown_send, mut shutdown) = tokio::sync::mpsc::unbounded_channel::<u64>();
        let (async_send, ani_recv) = crossbeam::unbounded();
        let vf = vf.clone();
        let cc = Box::leak(Box::new(cc)) as &'static ConfigCollection;
        let rsa_keypair = Box::leak(Box::new(openssl::rsa::Rsa::generate(1024).unwrap())) as &'static openssl::rsa::Rsa<_>;
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
                                let mut state: i32 = 0;
                                let mut last_seen = tokio::time::Instant::now();
                                info!("New JE client from {}", &addr);
                                let mut peek_buf = [0u8; 1];
                                //tokio::pin!(je_client);
                                'streamloop: loop {
                                    let poll_try = poll_fn(|mut cx| je_client.poll_peek(&mut cx, &mut peek_buf));
                                    tokio::select! {
                                        poll_result = poll_try => {
                                            match poll_result {
                                                Ok(0) => {
                                                    debug!("DE: poll_peek avail 0, assume dc, closing");
                                                    je_client.shutdown(Shutdown::Both);
                                                    break 'streamloop;
                                                }
                                                Ok(bytes_avail) => {
                                                    debug!("D: @{} AVAIL {}", &addr, bytes_avail);
                                                    match read_from_je(&mut je_client).await {
                                                        Ok((packet_len, packet_id, packet_data)) => {
                                                            debug!("D: @{} IN P\nlen {} id {} DATA\n\t{:?}\n", &addr, &packet_len, &packet_id, &packet_data);
                                                            match state {
                                                                0 => {
                                                                    debug!("state 0, parsing as handshake scanning for next");
                                                                    let try_handshake_packet = parse_je_data(packet_data.len(), &packet_data, &[
                                                                        JeNetType::VarInt,
                                                                        JeNetType::String,
                                                                        JeNetType::UShort,
                                                                        JeNetType::VarInt
                                                                    ]);
                                                                    debug!("{:?}", try_handshake_packet);
                                                                    match try_handshake_packet {
                                                                        Ok(mut packet) => {
                                                                            let pk_handshake_maybe = JePacketHandshake::try_raw(&packet);
                                                                            debug!("{:?}", &pk_handshake_maybe);
                                                                            if let Ok(pk_handshake) = pk_handshake_maybe {
                                                                                state = pk_handshake.next_state;
                                                                                debug!("@{} state -> {}", &addr, state);
                                                                            } else {
                                                                                debug!("DE: decode JePacketHandshake failed, skipping");
                                                                            }
                                                                        },
                                                                        Err(_) => {
                                                                            debug!("DE: @{} malformed data, skipping", &addr); 
                                                                        }
                                                                    }
                                                                }
                                                                1 => {
                                                                    // reply
                                                                    debug!("@{} <<< query meta", &addr);
                                                                    write_to_je(&mut je_client, 0x00, &[JeNetVal::String(server_response_json(
                                                                        "CM TEST",
                                                                        578,
                                                                        20,
                                                                        0,
                                                                        &[],
                                                                        "CraftMine Test Server",
                                                                        ""
                                                                    ))]).await;
                                                                }
                                                                2 => {
                                                                    let try_pk_login = parse_je_data(packet_data.len(), &packet_data, &[
                                                                        JeNetType::String
                                                                    ]).unwrap();
                                                                    debug!("{:?}", try_pk_login);
                                                                    if let Ok(pk_login_start) = JeLoginStart::try_raw(&try_pk_login) {
                                                                        debug!("try_pk_login_raw ok");
                                                                        if cc.auth.online_mode {
                                                                            let pubkey = rsa_keypair.public_key_to_der().unwrap();
                                                                            // TODO randomly generate vtoken instead
                                                                            let vtoken = vec![0u8, 1u8, 2u8, 3u8];
                                                                            // send enc request
                                                                            debug!("Sending enc request");
                                                                            write_to_je(&mut je_client, 0x01, &[
                                                                                JeNetVal::String("".to_owned()),
                                                                                JeNetVal::VarInt(pubkey.len() as i32),
                                                                                JeNetVal::Array(pubkey),
                                                                                JeNetVal::VarInt(vtoken.len() as i32),
                                                                                JeNetVal::Array(vtoken.clone())
                                                                            ]).await;
                                                                        } else {
                                                                            debug!("@{} OFFLINE MODE LOGIN", &addr);
                                                                            debug!("sending login success");
                                                                            write_to_je(&mut je_client, 0x02, &[
                                                                                JeNetVal::String("550e8400-e29b-41d4-a716-446655440000".to_owned()),
                                                                                JeNetVal::String(pk_login_start.name.clone())
                                                                            ]).await;
                                                                            state = 3;

                                                                            // join game
                                                                            write_to_je(&mut je_client, 0x26, &[
                                                                                JeNetVal::Int(0x01000000),  // eid
                                                                                JeNetVal::UByte(0x1),       // gamemode
                                                                                JeNetVal::Int(0),           // dimension
                                                                                JeNetVal::Long(0x0),        // hashed seed
                                                                                JeNetVal::UByte(20),        // max players
                                                                                JeNetVal::String("flat".to_owned()),    // level type
                                                                                JeNetVal::VarInt(8),        // view distance
                                                                                JeNetVal::Boolean(false),   // reduced debug
                                                                                JeNetVal::Boolean(false)    // enable respawn

                                                                            ]).await;
                                                                            // initial play state
                                                                            /*debug!("sending initial play state");
                                                                            write_to_je(&mut je_client, 0x00, &[
                                                                                JeNetVal::VarInt(0),
                                                                                JeNetVal::Array(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap().as_bytes().to_vec()),
                                                                                JeNetVal::VarInt(106),
                                                                                JeNetVal::Double(0.0f64),
                                                                                JeNetVal::Double(0.0f64),
                                                                                JeNetVal::Double(0.0f64),
                                                                                JeNetVal::UByte(0),
                                                                                JeNetVal::UByte(0),
                                                                                JeNetVal::Int(0), // data
                                                                                JeNetVal::Short(0),
                                                                                JeNetVal::Short(0),
                                                                                JeNetVal::Short(0)
                                                                            ]).await;*/
                                                                            // ignore client stuff for now
                                                                            // held item change
                                                                            write_to_je(&mut je_client, 0x40, &[
                                                                                JeNetVal::Byte(0)
                                                                            ]).await;
                                                                            // spawn position
                                                                            write_to_je(&mut je_client, 0x4e, &[
                                                                                JeNetVal::Long(0)
                                                                            ]).await;
                                                                            // player position and look
                                                                            write_to_je(&mut je_client, 0x36, &[
                                                                                JeNetVal::Double(0f64),
                                                                                JeNetVal::Double(0f64),
                                                                                JeNetVal::Double(0f64),
                                                                                JeNetVal::Float(0.0f32),
                                                                                JeNetVal::Float(0.0f32),
                                                                                JeNetVal::UByte(0),
                                                                                JeNetVal::VarInt(1)
                                                                            ]).await;
                                                                        }
                                                                    } else {
                                                                        debug!("DE login start err");
                                                                    }
                                                                }
                                                                3 => {
                                                                    debug!("play state");
                                                                }
                                                                _ => {
                                                                    debug!("unknown state");
                                                                }
                                                            }
                                                        },
                                                        Err(_) => {
                                                            debug!("DE: @{} malformed packet, skipping", &addr);

                                                        }
                                                    }
                                                },
                                                Err(e) => {
                                                    debug!("DE: poll_peek failed {:?} closing", e);
                                                    je_client.shutdown(Shutdown::Both);
                                                    break 'streamloop;
                                                }
                                            }
                                        }
                                        /*je_res = read_from_je(&mut je_client) => {
                                            match je_res {
                                                Ok((packet_len, packet_id, packet_data)) => {
                                                    debug!("D: @{} IN P\nlen {} id {} DATA\n\t{:?}\n", &addr, &packet_len, &packet_id, &packet_data);
                                                    match (state, packet_id) {
                                                        (0, 0) => {
                                                            /*match JePacketHandshake::try_from(packet_data.as_slice()) {
                                                                Ok(p) => {},
                                                                Err(e) => {
                                                                    // TODO skip whole packet len bytes
                                                                    warn!("Invalid handshake packet with state 0");
                                                                    if (&cc).net.kick_invalid_packet {
                                                                        je_kick(&mut je_client, e);
                                                                        break 'streamloop;
                                                                    }
                                                                }
                                                            }*/
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
                                        }*/
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