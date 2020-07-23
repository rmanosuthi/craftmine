pub trait WorldGenerator {
    fn raw_chunk(&self, x: i64, z: i64) -> RawChunk;
}