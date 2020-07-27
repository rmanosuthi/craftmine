use crate::*;
use crossbeam::{Sender, Receiver};
use serde::{Serialize, Deserialize};
use std::{net::TcpListener, error::Error};
use hashbrown::HashMap;

mod auth;
pub mod config;
mod game;
mod prefix;

pub use self::auth::*;
pub use self::prefix::*;
pub use self::game::GameServer;
use config::ConfigFolder;
use world::WorldFolder;
use net::AsyncNetInstance;
pub struct ServerInitializer(pub InitFlags);

pub type Termination = bool;
pub type InteractionNeeded = bool;

mod net {
    mod instance;
    mod je;
    mod legacy;
    mod packets;
    pub use self::instance::*;
    pub use self::je::*;
    pub use self::packets::*;
    use self::legacy::*;
}

mod world {
    use crate::*;
    use uuid::Uuid;
    use hashbrown::HashMap;
    use server::game::GameWorld;

    pub mod generator;
    pub struct WorldFolder {}
    impl WorldFolder {
        // TODO actually implement
        pub fn load_or_default(vf: &ValidatedInitFlags) -> Result<HashMap<Uuid, GameWorld>, Vec<String>> {
            Ok(HashMap::new())
        }
    }
}

impl ServerInitializer {
    pub fn start(&self) -> ServerInitResult {
        let flags = &self.0;
        let mut infos = Vec::new();
        let mut warns = Vec::new();
        let mut errs = Vec::new();

        // TODO Check init flags valid
        let validated_flags_maybe = flags.to_validated();

        if let Err(mut e) = validated_flags_maybe.clone() {
            errs.append(&mut e);
        }

        if errs.is_empty() {
            // can safely unwrap validated_flags
            let mut validated_flags = validated_flags_maybe.unwrap().clone();

            info!("Startup args validated");
            debug!("Validated flags {:?}", &validated_flags);

            infos.append(&mut validated_flags.prefix.1);
            infos.append(&mut validated_flags.config_path.1);
            infos.append(&mut validated_flags.je_port.1);
            infos.append(&mut validated_flags.be_port.1);
            infos.append(&mut validated_flags.bind_addr.1);

            let mut prefix_errors = validated_flags.try_create_prefix();

            if prefix_errors.len() != 0 {
                errs.append(&mut prefix_errors);
            }

            // Check ports bindable
            if let Err(_) = TcpListener::bind(
                format!("127.0.0.1:{}", validated_flags.je_port.0)
            ) {
                errs.push(
                    format!("Failed to bind to JE port {}", validated_flags.je_port.0)
                )
            } else {
                infos.push(format!("JE: port {} available", validated_flags.je_port.0));
            }

            if let Err(_) = TcpListener::bind(
                format!("127.0.0.1:{}", validated_flags.be_port.0)
            ) {
                errs.push(
                    format!("Failed to bind to BE port {}", validated_flags.be_port.0)
                )
            } else {
                infos.push(format!("BE: port {} available", validated_flags.be_port.0));
            }

            // Check configs exist and valid
            let cc_maybe = match ConfigFolder::load_or_new_all(&validated_flags) {
                Ok(cc) => Some(cc),
                Err(errors) => {
                    for e in errors {
                        errs.push(e);
                    }
                    None
                }
            };

            // Check worlds
            let worlds_maybe = match WorldFolder::load_or_default(&validated_flags) {
                Ok(worlds) => Some(worlds),
                Err(errors) => {
                    for e in errors {
                        errs.push(e);
                    }
                    None
                }
            };

            if errs.is_empty() {
                let cc = cc_maybe.unwrap();
                let async_net_instance = AsyncNetInstance::new(validated_flags.clone(), cc.clone());

                let (cli_send, gs_cli_recv) = crossbeam::unbounded();
                let (gs_cli_send, cli_recv) = crossbeam::unbounded();

                let web_ws = cc.net.web_addr_port.clone();

                let sra = SrAllocator::new(&cc);
                sra.report();

                let instance = GameServer {
                    prefix: ServerPrefix(validated_flags.prefix.0.clone()),
                    init_flags: validated_flags.clone(),
                    worlds: worlds_maybe.unwrap(),
                    cli_recv: gs_cli_recv,
                    cli_send: gs_cli_send,
                    async_net_instance,
                    cc
                };
                ServerInitResult {
                    instance: if errs.is_empty() {
                        Ok((instance, ServerInitChannels {
                            cli_recv: cli_recv,
                            cli_send: cli_send,
                            web_ws
                        }))
                    } else {
                        Err(errs)
                    },
                    warn: warns,
                    info: infos
                }
            } else {
                ServerInitResult {
                    instance: Err(errs),
                    warn: warns,
                    info: infos
                }
            }
        } else {
            // early exit because can't do much without proper init flags
            ServerInitResult {
                instance: Err(errs),
                warn: warns,
                info: infos
            }
        }
    }
}

pub struct ServerInitResult {
    pub instance: Result<(GameServer, ServerInitChannels), Vec<String>>,
    pub warn: Vec<(String, InteractionNeeded)>,
    pub info: Vec<String>
}

pub struct ServerInitChannels {
    pub cli_recv: Receiver<CliMessageInbound>,
    pub cli_send: Sender<CliMessageOutbound>,
    pub web_ws: String
}

#[derive(Debug)]
pub enum ServerInitWarn {
    NoConfig,
    EmptyPrefix,
    UsingCurrentDir
}

impl ServerInitWarn {
    pub fn to_kv(self) -> (ServerInitWarn, InteractionNeeded) {
        match self {
            ServerInitWarn::NoConfig => (self, false),
            _ => unimplemented!()
        }
    }
}

#[derive(Debug)]
pub enum PortType {
    Java,
    Bedrock
}