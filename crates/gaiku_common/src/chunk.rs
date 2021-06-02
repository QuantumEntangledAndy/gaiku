#[allow(clippy::module_inception)]
mod chunk;
mod sparse_chunk;

pub use chunk::Chunk;
pub use sparse_chunk::SparseChunk;

/// Base common denominator across all the chunk implementations used.
pub trait Chunkify<Coord, Value> {
  fn is_air(&self, x: Coord, y: Coord, z: Coord) -> bool;
  fn get(&self, x: Coord, y: Coord, z: Coord) -> Value;
}

/// Defines a mutable chunk.
pub trait ChunkifyMut<Coord, Value> {
  fn set(&mut self, x: Coord, y: Coord, z: Coord, value: Value);
}

/// Signisies that the chunk has an atlas
pub trait Atlasify<Coord, AtlasValue> {
  /// Get the atlas (material)
  fn get_atlas(&self, x: Coord, y: Coord, z: Coord) -> AtlasValue;
}

pub trait AtlasifyMut<Coord, AtlasValue> {
  /// Get the atlas (material)
  fn set_atlas(&mut self, x: Coord, y: Coord, z: Coord, value: AtlasValue);
}
