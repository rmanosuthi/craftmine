pub struct RawChunk(Chunk);

pub struct Chunk {}

pub struct SelfContainedChunk {
    chunk: Chunk,
    entities: Vec<Entity>,
    ref_structures: Vec<StructureHandle>
}