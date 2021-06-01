use gaiku_common::chunk::Chunk;

use amethyst::ecs::prelude::*;
use gaiku_amethyst::prelude::*;

/// We use this structure for the LOD tree
/// chunks so that we can hold extra meta data
/// with it related to amethyst and the mesh
pub struct MetaChunk {
  chunk: Chunk,
  entity: Option<Entity>,
}

impl MetaChunk {
  pub fn get_entity(&self) -> Option<Entity> {
    self.entity.clone()
  }

  pub fn set_entity(&mut self, ent: Entity) {
    self.entity = Some(ent);
  }
}

impl Boxify for MetaChunk {
  fn new(position: [f32; 3], width: u16, height: u16, depth: u16) -> Self {
    Self {
      chunk: Chunk::new(position, width, height, depth),
      entity: None,
    }
  }
}

impl Chunkify<(u8, u8)> for MetaChunk {
  fn is_air(&self, x: usize, y: usize, z: usize) -> bool {
    self.chunk.is_air(x, y, z)
  }

  fn get(&self, x: usize, y: usize, z: usize) -> (u8, u8) {
    self.chunk.get(x, y, z)
  }
}

impl ChunkifyMut<(u8, u8)> for MetaChunk {
  fn set(&mut self, x: usize, y: usize, z: usize, value: (u8, u8)) {
    self.chunk.set(x, y, z, value)
  }
}

impl Positionable for MetaChunk {
  fn with_position(position: [f32; 3]) -> Self {
    Self {
      chunk: Chunk::new(position, 16, 16, 16),
      entity: None,
    }
  }

  fn position(&self) -> [f32; 3] {
    self.chunk.position()
  }
}

impl Sizable for MetaChunk {
  fn with_size(width: u16, height: u16, depth: u16) -> Self {
    Self {
      chunk: Chunk::new([0.0, 0.0, 0.0], width, height, depth),
      entity: None,
    }
  }

  fn depth(&self) -> u16 {
    self.chunk.depth()
  }

  fn height(&self) -> u16 {
    self.chunk.height()
  }

  fn width(&self) -> u16 {
    self.chunk.width()
  }
}
