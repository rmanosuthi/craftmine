use crate::imports::*;
use crate::server::symbols::*;
use std::future::Future;

pub type CgUuid = Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct PersistId(pub Uuid);

pub type Eid = i32;

/// The game world.
pub struct World {
    /// Receiver for messages from `NetServer`
    pub inc_packets: crossbeam::Receiver<NetRecvMsg>,
    /// Map of causal groups
    pub cg: HashMap<CgUuid, CausalGroup>,
    /// Lookup table for entities to causal groups
    pub eid_cg_lookup: HashMap<Eid, CgUuid>,
    pub p: WorldProperty,
    pub cc: ConfigCollection,
    pub next_eid: i32, // TODO overflow?
    pub _cache_cg_contains: HashMap<CgUuid, Vec<XZRecord>>,
    pub _q_cg: Vec<CgOperation>,
    pub io: WorldIO
}

pub struct WorldIO {}

impl WorldIO {
    pub fn load_entity_data(&self, persist_id: &PersistId) -> Option<Entity> {
unimplemented!()
    }
}

pub struct GameruleCollection {
    pub do_immediate_respawn: bool
}

pub struct WorldProperty {
    pub dimension: Dimension,
    pub seed: WorldSeed,
    pub level_type: JeLevelType,
    pub default_spawn: LocalityRecord,
    pub default_gamemode: Gamemode,
    pub rules: GameruleCollection,
}

pub struct WorldSeed(u32);

impl WorldSeed {
    pub fn from_raw(raw: u32) -> WorldSeed {
        WorldSeed(raw)
    }
    pub fn from_str(st: &str) -> WorldSeed {
        unimplemented!()
    }
    pub fn hash(&self) -> &[u8; 32] {
        unimplemented!()
    }
    pub fn hash_first_u64(&self) -> i64 {
        unimplemented!()
    }
}

pub type Speculative = bool;

pub struct CgQueue {}

pub enum CgOperation {
    NewUser {e: Eid, rec: UserRecord, e_data: Entity},
    UnloadUser {u: Uuid, cg: CgUuid},
    Merge {l: CgUuid, r: CgUuid},
    MoveChunkColumn {from: CgUuid, to: CgUuid, loc: XZRecord, col: ChunkColumn},
    RequestChunkColumn {recv: CgUuid, loc: XZRecord }
}

impl World {
    pub fn locality_to_cg(&self, at: &LocalityRecord) -> Option<CgUuid> {
        unimplemented!()
    }
    pub fn take_user(&mut self, user: &UserRecord) -> Option<JeJoinGame> {
        // IMPORTANT: WITHOUT ASYNC, eid allocation immediately returning is speculative;
        // it may fail later, where TODO user should be disconnected?
        let (new_eid, op) = self.new_user_eid_speculative(user);
        self._q_cg.push(op);
        Some(JeJoinGame {
            entity_id: new_eid,
            gamemode: user.gamemode.into(),
            dimension: i8::from(self.p.dimension) as i32,
            hashed_seed: self.p.seed.hash_first_u64(),
            max_players: self.cc.auth.max_players,
            level_type: self.p.level_type,
            view_distance: JeVarInt(self.cc.perf.view_distance_chunks as i32),
            reduced_debug_info: false,
            enable_respawn_screen: !self.p.rules.do_immediate_respawn
        })
    }
    pub fn tick(&mut self) -> Vec<WorldBroadcastMsg> {
        unimplemented!()
    }
    pub fn new_eid(&mut self, cg: CgUuid) -> Eid {
        self.eid_cg_lookup.insert(self.next_eid, cg);
        let eid = self.next_eid;
        self.next_eid += 1;
        eid
    }
    pub fn new_user_eid_speculative(&mut self, u: &UserRecord) -> (Eid, CgOperation) {
        let eid = self.next_eid;
        self.next_eid += 1;
        (eid, CgOperation::NewUser {
            e: eid,
            rec: u.to_owned(),
            e_data: self.io.load_entity_data(&u.persist_id).unwrap_or(Entity::from_world_props(&self.p))
        })
    }
}

/// Result of a `CausalGroup` tick.
/// Due to the tick return function, the caller is not included in the enum variants.
pub enum CgBroadcastMsg {
    /// Request merging with another `CausalGroup` by `x, z, radius`.
    /// This operation should be run **after** all others.
    RequestMerge {
        x: f64,
        z: f64,
        radius: u64
    },
    /// Send a packet destined for all members of the `CausalGroup`.
    AllPacket(JeGenericPacket),
    /// Send a packet destined for a single member of the `CausalGroup`.
    OnePacket(Uuid, JeGenericPacket)
}

pub enum WorldBroadcastMsg {
    MovePlayer {u: Uuid, target_world: String, tp: TeleportMode},
    BroadcastAll(JeGenericPacket)
}

pub enum TeleportMode {
    Multiplier {
        origin: LocalityRecord,
        target_center: LocalityRecord,
        multiplier: f32,
        seek: Option<(BlockId, u64)>
    }
}

/// Collection of in-group-affecting components.
///
/// By design, `CausalGroup`s are completely isolated from each other, so that computation may be safely parallelized.
pub struct CausalGroup {
    users: HashMap<Uuid, UserRecord>,
    chunks: HashMap<XZRecord, ChunkColumn>,
    entities: HashMap<Eid, Entity>
}

impl CausalGroup {
    pub fn tick(&mut self) -> Vec<CgBroadcastMsg> {
        unimplemented!()
    }
    pub fn merge(one: CausalGroup, two: CausalGroup) -> CausalGroup {
        unimplemented!()
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct XZRecord {
    pub x: u64,
    pub z: u64
}

pub struct ChunkColumn {
    pub sections: [Option<ChunkSection>; 16]
}

pub struct ChunkSection {
    pub blocks: [BlockData; 4096]
}

pub struct Entity {
    pub max_health: u32,
    pub health: u32,
    pub loc: LocalityRecord
}

impl Entity {
    pub fn from_world_props(wp: &WorldProperty) -> Entity {
        unimplemented!()
    }
}

pub struct BlockData {}