use super::ServerPrefix;
use hashbrown::HashMap;
use crossbeam::{Sender, Receiver};
use uuid::Uuid;

pub struct GameServer {
    prefix: ServerPrefix,
    worlds: HashMap<Uuid, GameWorld>,
    net_send: Sender<NetCommandSend>,
    net_recv: Receiver<NetCommandRecv>
}