use crate::imports::*;
use crate::server::symbols::*;

pub type CgUuid = Uuid;
pub type Eid = i32;

pub struct World {
    pub u_cg_lookup: HashMap<Uuid, CgUuid>,
    pub inc_packets: crossbeam::Receiver<NetRecvMsg>,
    pub cg: HashMap<CgUuid, CausalGroup>
}

impl World {
    pub fn take_user(&mut self, user: &UserRecord) {}
    pub fn tick(&mut self) -> Vec<WorldBroadcastMsg> {}
}

pub enum WorldBroadcastMsg<T: JePacket> {
    MovePlayer(Uuid, String, TeleportMode),
    BroadcastCg(Uuid, T)
}

pub enum TeleportMode {
    Multiplier {
        origin: LocalityRecord,
        target_center: LocalityRecord,
        multiplier: f32,
        seek: Option<(BlockId, u64)>
    }
}

pub struct CausalGroup {
    users: HashMap<Uuid, UserRecord>,
    chunks: HashMap<ChunkLocality, ChunkColumn>,
    entities: HashMap<Eid, Entity>
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ChunkLocality {
    pub x: u64,
    pub z: u64
}

pub struct ChunkColumn {
    pub sections: [Option<ChunkSection>; 16]
}

pub struct ChunkSection {
    pub blocks: [BlockData; 4096]
}