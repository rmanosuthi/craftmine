use crate::imports::*;
use crate::server::symbols::*;
use crate::init_flags::*;
use tokio::future::poll_fn;
use std::net::Shutdown;

/// Network instance.
/// TODO make sure clients can't spoof uuid because of the shared uuid socket
pub struct NetServer {
    pub rt_handle: std::thread::JoinHandle<()>,
    pub ani_send: tokio::sync::mpsc::Sender<NetSendMsg>,
    pub ani_recv: crossbeam::Receiver<NetRecvMsg>,
    pub signal_shutdown: tokio::sync::mpsc::UnboundedSender<u64>,
    cc_ptr: &'static ConfigCollection
}

impl NetServer {
    pub fn new(vf: ValidatedInitFlags, cc: ConfigCollection) -> NetServer {
        let mut rt = tokio::runtime::Builder::new().basic_scheduler().enable_all().build().unwrap();
        let (ani_send, mut async_recv) = tokio::sync::mpsc::channel(cc.net.sync_async_channel_len);
        let (shutdown_send, mut shutdown) = tokio::sync::mpsc::unbounded_channel::<u64>();
        let (async_send, ani_recv) = crossbeam::unbounded();
        let vf = vf.clone();
        let cc = Box::leak(Box::new(cc)) as &'static ConfigCollection;
        let rsa_keypair = Box::leak(Box::new(openssl::rsa::Rsa::generate(1024).unwrap())) as &'static openssl::rsa::Rsa<_>;
        let rt_handle = std::thread::spawn(move || {
            rt.block_on(async {
                let mut async_recv = async_recv;
                let listen_bind = format!("{}:{}", &vf.bind_addr.0, &vf.je_port.0);
                let mut listener = tokio::net::TcpListener::bind(&listen_bind).await.unwrap();
                let (send_new_conn, mut recv_new_conn) = tokio::sync::mpsc::unbounded_channel::<JeConnection>();
                let mut map_uuid_conn: HashMap<Uuid, JeConnection> = HashMap::new();
                let mut async_net_active = true;
                info!("Listening on {}", &listen_bind);
                //let mut streams = HashMap::new();
                while async_net_active {
                    tokio::select! {
                        Some(net_msg) = async_recv.recv() => {
                            match net_msg {
                                _ => {}
                            }
                        },
                        Some(_) = shutdown.recv() => {
                            async_net_active = false;
                        }
                        Some(new_conn) = recv_new_conn.recv() => {
                            let (uuid, conn) = (new_conn.uuid.clone(), new_conn);
                            debug!("Adding session {:?}, {:?}", &uuid, &conn);
                            info!("{} ({}) has joined the server from {}", &conn.username, &conn.uuid, &conn.addr);
                            
                            async_send.send(NetRecvMsg {
                                uuid: uuid.clone(),
                                inner: NetRecvInner::NewSession {
                                    username: conn.username.clone()
                                }
                            });
                            map_uuid_conn.insert(uuid, conn);
                        },
                        Ok((stream, addr)) = listener.accept() => {
                            let send_new_conn = send_new_conn.clone();
                            let async_send = async_send.clone();
                            //streams.insert(addr, stream);
                            tokio::task::spawn(async move {
                                // TODO timeout
                                let mut je_client = stream;
                                let mut state = 0;
                                let mut last_seen = tokio::time::Instant::now();
                                info!("New JE client from {}", &addr);
                                let mut peek_buf = [0u8; 1];
                                //tokio::pin!(je_client);
                                let (send_to_session, mut recv_send_to_session) = tokio::sync::mpsc::unbounded_channel::<(i32, Vec<JeNetVal>)>();
                                let mut conn: Option<JeConnection> = None;
                                let mut run = true;
                                'streamloop: while run {
                                    let send_to_session = send_to_session.clone();

                                    let poll_try = poll_fn(|mut cx| je_client.poll_peek(&mut cx, &mut peek_buf));
                                    tokio::select! {
                                        Some(msg_to_session) = recv_send_to_session.recv() => {
                                            debug!("{} <- new msg", &addr);
                                            match &conn {
                                                Some(c) => {
                                                    match &c.enc {
                                                        Some(s_enc) => {},
                                                        None => {
                                                            write_to_je(&mut je_client, msg_to_session.0, &msg_to_session.1).await;
                                                        }
                                                    }
                                                },
                                                None => {
                                                    warn!("{} unexpected outbound packet to incomplete connection", &addr);
                                                }
                                            }
                                        }
                                        poll_result = poll_try => {
                                            match poll_result {
                                                Ok(0) => {
                                                    debug!("{} poll_peek len 0, closing", &addr);
                                                    run = false;
                                                }
                                                Ok(bytes_avail) => {
                                                    match read_from_je(&mut je_client).await {
                                                        Ok((packet_len, packet_id, packet_data)) => {
                                                            debug!("{} IN P (len {} id {}) DATA\n\t{:?}", &addr, &packet_len, &packet_id, &packet_data);
                                                            match state {
                                                                0 => {
                                                                    debug!("{} state 0, parsing as handshake scanning for next", &addr);
                                                                    if let Ok(pk_handshake) = JePacketHandshake::try_from_raw(&packet_data) {
                                                                        state = pk_handshake.next_state.0;
                                                                        debug!("{} state -> {}", &addr, state);
                                                                    } else {
                                                                        debug!("DE: decode JePacketHandshake failed, skipping");
                                                                    }
                                                                }
                                                                1 => {
                                                                    match packet_id {
                                                                        0 => {
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
                                                                        },
                                                                        1 => {
                                                                            // pong
                                                                            if let Ok(data) = parse_je_data(packet_data.len(), &packet_data, &[
                                                                                JeNetType::Long
                                                                            ]) {
                                                                                if let JeNetVal::Long(ping) = data[0] {
                                                                                    write_to_je(&mut je_client, 0x1, &[
                                                                                        JeNetVal::Long(ping)
                                                                                    ]).await;
                                                                                    run = false;
                                                                                }
                                                                            } else {
                                                                                debug!("invalid ping packet");
                                                                            }
                                                                        },
                                                                        _ => {}
                                                                    }
                                                                }
                                                                2 => {
                                                                    if let Ok(pk_login_start) = JeLoginStart::try_from_raw(&packet_data) {
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
                                                                            let new_conn = JeConnection {
                                                                                state: state,
                                                                                enc: None,
                                                                                uuid: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
                                                                                addr: addr.clone(),
                                                                                username: pk_login_start.name.clone(),
                                                                                send: send_to_session
                                                                            };
                                                                            send_new_conn.send(new_conn.clone());
                                                                            conn = Some(new_conn);

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
                                                    run = false;
                                                }
                                            }
                                        }
                                    };
                                }
                                if let Some(c) = conn {
                                    &async_send.send(NetRecvMsg {
                                        uuid: c.uuid,
                                        inner: NetRecvInner::EndSession
                                    });
                                }
                                je_client.shutdown(Shutdown::Both);

                            });
                        }
                    }
                }
                info!("Async net thread shutting down");
                
                unsafe {
                    drop(Box::from_raw(
                        std::mem::transmute::<_, *mut ConfigCollection>(cc)
                    ));
                }
            });
        });
        Self {
            rt_handle,
            ani_send,
            ani_recv,
            cc_ptr: cc,
            signal_shutdown: shutdown_send
        }
    }

    pub fn all(&mut self, packet_id: i32, data: &[u8]) {
        self.ani_send.send(NetSendMsg::All(
            packet_id, data.to_owned()
        ));
    }
    pub fn broadcast(&mut self, packet_id: i32, to: &[Uuid], data: &[u8]) {
        self.ani_send.send(NetSendMsg::Broadcast(
            to.to_owned(), packet_id, data.to_owned()
        ));
    }
    pub fn single(&mut self, packet_id: i32, to: &Uuid, data: &[u8]) {
        self.ani_send.send(NetSendMsg::Single(
            to.to_owned(), packet_id, data.to_owned()
        ));
    }
    pub fn disconnect(&mut self, to: &Uuid, msg: &str) {
        self.ani_send.send(NetSendMsg::Disconnect(
            to.to_owned(), msg.to_owned()
        ));
    }
    pub fn timeout(&mut self, player: &Uuid, duration: Option<Duration>, msg: &str) {
        self.ani_send.send(
            if let Some(dur) = duration {
                NetSendMsg::DefiniteTimeout(player.to_owned(), dur, msg.to_owned())
            } else {
                NetSendMsg::IndefiniteTimeout(player.to_owned(), msg.to_owned())
            }
        );
    }
}