use crate::imports::*;
use crate::server::symbols::*;
use crate::init_flags::*;

pub struct GameServer {
    pub prefix: ServerPrefix,
    pub init_flags: ValidatedInitFlags,
    pub worlds: HashMap<Uuid, World>,
    pub cli_recv: crossbeam::Receiver<String>,
    pub cli_send: crossbeam::Sender<String>,
    pub async_net_instance: NetServer,
    pub cc: ConfigCollection,
    pub tick: crossbeam::Receiver<Instant>,
    pub recv_status: crossbeam::Receiver<ServerStatus>,
    pub send_status: crossbeam::Sender<ServerStatus>,
    pub users: HashMap<Uuid, UserRecord>,
    pub last_tick: Instant
}

impl GameServer {
    pub fn run(&mut self) {
        info!("Starting game server");
        let mut run = true;
        while run {
            crossbeam::select! {
                recv(self.recv_status) -> status => {
                    debug!("Got status {:?}", &status);
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
                    while let Ok(inc_net_packet) = self.async_net_instance.ani_recv.try_recv() {
                        debug!("{:?}", inc_net_packet);
                        match inc_net_packet.inner {
                            NetRecvInner::NewSession {
                                username: u,
                                online: online
                            } => {
                                if let Ok(user) = match online {
                                    true => {
                                        info!("{} ({}) has joined the server.", &u, &inc_net_packet.uuid);
                                        self.prefix.users.load_or_new_online(
                                            &inc_net_packet.uuid,
                                            &u
                                        )
                                    },
                                    false => {
                                        warn!("OFFLINE {} ({}) has joined the server.", &u, &inc_net_packet.uuid);
                                        self.prefix.users.load_or_new_offline(&u)
                                    }
                                } {
                                    self.users.insert(inc_net_packet.uuid.clone(), user);
                                    self.accept_user(&user);
                                } else {
                                    error!("Failed to load or create user record for {}, disconnecting player", &u);
                                    self.async_net_instance.disconnect(
                                        &inc_net_packet.uuid,
                                        "Failed to access user records"
                                    );
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
    pub fn accept_user(&mut self, u: &UserRecord) {
        self.async_net_instance.single(&u.uuid, JeJoinGame {

        });
    }
}