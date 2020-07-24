use crate::*;
use super::ServerPrefix;
use hashbrown::HashMap;
use crossbeam::{Sender, Receiver};
use uuid::Uuid;

pub struct GameServer {
    prefix: ServerPrefix,
    init_flags: ValidatedInitFlags,
    worlds: HashMap<Uuid, GameWorld>,
    net_send: Sender<NetCommandSend>,
    net_recv: Receiver<NetCommandRecv>,
    cli_recv: Receiver<CliMessageOutbound>,
    cli_send: Sender<CliMessageInbound>,
    async_net_instance: AsyncNetInstance
}

impl GameServer {
}