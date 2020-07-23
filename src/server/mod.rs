use crate::*;
use crossbeam::{Sender, Receiver};
use serde::{Serialize, Deserialize};
use std::error::Error;
use hashbrown::HashMap;

pub mod config;
mod game;
mod prefix;

pub(crate) use self::prefix::*;
pub(crate) use self::game::GameServer;
pub struct ServerInitializer(InitFlags);

pub(crate) type Termination = bool;
pub(crate) type InteractionNeeded = bool;

mod net {
    mod je;
    mod legacy;
    use self::je::*;
    use self::legacy::*;
}

impl ServerInitializer {
    pub fn start(self) -> ServerInitResult {
        ServerInitResult {
            ok_or_err: {
                let err_list = self.0.try_get_errs();
                if err_list.len() == 0 {
                    Ok(force_init_server(&self.0))
                } else {
                    Err(err_list)
                }
            },
            warn: [
                check_config_exists(&self.0)
            ].iter().flatten().map(|e| e.to_kv()).collect()
        }
    }
    fn force_init_server(flags: InitFlags) -> ServerInstance {

    }
}

pub struct ServerInitResult {
    ok_or_err: Result<ServerInstance, Vec<ServerInitError>>,
    warn: HashMap<ServerInitWarn, InteractionNeeded>,
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
pub enum ServerInitError {
    PfxNonexistent,
    PfxActiveServer,
    PfxIoError(Box<dyn Error>),
    PfxError(ServerPrefixError),
    PortInUse(PortType, u16),
    ConfigInvalid(Box<dyn Error>)
}

#[derive(Debug)]
pub enum PortType {
    Java,
    Bedrock
}