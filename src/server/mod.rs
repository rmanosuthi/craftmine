use crate::*;
use crossbeam::{Sender, Receiver};
use serde::{Serialize, Deserialize};
use std::{net::TcpListener, error::Error};
use hashbrown::HashMap;

mod auth;
pub mod config;
mod game;
mod prefix;

pub(crate) use self::auth::*;
pub(crate) use self::prefix::*;
pub(crate) use self::game::GameServer;
pub struct ServerInitializer(pub InitFlags);

pub(crate) type Termination = bool;
pub(crate) type InteractionNeeded = bool;

mod net {
    mod je;
    mod legacy;
    pub use self::je::*;
    use self::legacy::*;
}

impl ServerInitializer {
    pub fn start(&self) -> ServerInitResult {
        let flags = self.0;
        let mut infos = Vec::new();
        let mut warns = Vec::new();
        let mut errs = Vec::new();

        // TODO Check init flags valid
        let validated_flags_maybe = flags.to_validated();

        if let Err(e) = validated_flags_maybe {
            errs.append(&mut e);
        }

        if errs.is_empty() {
            // can safely unwrap validated_flags
            let validated_flags = validated_flags_maybe.unwrap();

            // Check ports bindable
            if let Err(_) = TcpListener::bind(
                format!("127.0.0.1:{}", validated_flags.je_port.0)
            ) {
                errs.push(
                    format!("Failed to bind to JE port {}", validated_flags.je_port.0)
                )
            };

            if let Err(_) = TcpListener::bind(
                format!("127.0.0.1:{}", validated_flags.be_port.0)
            ) {
                errs.push(
                    format!("Failed to bind to BE port {}", validated_flags.be_port.0)
                )
            };

            // Check configs exist and valid
            let cc_maybe = match ConfigFolder::load_or_new_all() {
                Ok(cc) => Some(cc),
                Err(errors) => {
                    for e in errors {
                        errs.push(e);
                    }
                    None
                }
            };

            // Check worlds
            let worlds_maybe = match WorldFolder::load_or_default() {
                Ok(worlds) => Some(worlds),
                Err(errors) => {
                    for e in errors {
                        errs.push(e);
                    }
                    None
                }
            };

            if errs.is_empty() {
                let instance = GameServer {
                    prefix: ServerPrefix(validated_flags.prefix.0),
                    init_flags: validated_flags,
                    worlds: worlds_maybe.unwrap(),
                    net_send,
                    net_recv,
                    cli_recv: gs_cli_recv,
                    cli_send: gs_cli_send,
                    async_net_instance
                };
                ServerInitResult {
                    instance: if errs.is_empty() {
                        Ok((instance, ServerInitChannels {
                            cli_recv: cli_recv,
                            cli_send: cli_send,
                            web_ws: web_addr
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
    instance: Result<(GameServer, ServerInitChannels), Vec<String>>,
    warn: Vec<(String, InteractionNeeded)>,
    info: Vec<String>
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