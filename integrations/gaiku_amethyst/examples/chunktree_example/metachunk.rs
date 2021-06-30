use amethyst::ecs::prelude::*;
use gaiku_amethyst::prelude::*;
use gaiku_common::{chunk::Chunk, density::DensityData};
use std::convert::TryInto;

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

  /// Fill a chunk based on bounds in a density
  pub fn fill_chunk(
    &mut self,
    atlas_fill: u8,
    density: &DensityData,
    bounds: &([f32; 3], [f32; 3]),
  ) {
    let (min, max) = bounds;
    let size = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let dimensions = [
      self.chunk.width() as usize,
      self.chunk.height() as usize,
      self.chunk.depth() as usize,
    ];
    let delta = [
      size[0] / (dimensions[0] - 1) as f32,
      size[1] / (dimensions[1] - 1) as f32,
      size[2] / (dimensions[2] - 1) as f32,
    ];
    for i in 0..dimensions[0] {
      let x = min[0] + i as f32 * delta[0];
      for j in 0..dimensions[1] {
        let y = min[1] + j as f32 * delta[1];
        for k in 0..dimensions[2] {
          let z = min[2] + k as f32 * delta[2];
          let value = density.get_value(x, y, z);
          if value > 1. {
            let i: u16 = i.try_into().unwrap();
            let j: u16 = j.try_into().unwrap();
            let k: u16 = k.try_into().unwrap();
            self.chunk.set(i, j, k, 1);
            self.chunk.set_atlas(i, j, k, atlas_fill);
          }
        }
      }
    }
  }
}

impl Boxify<f32, u16> for MetaChunk {
  fn new(position: [f32; 3], width: u16, height: u16, depth: u16) -> Self {
    Self {
      chunk: Chunk::new(position, width, height, depth),
      entity: None,
    }
  }
}

impl Chunkify<u16, u8> for MetaChunk {
  fn is_air(&self, x: u16, y: u16, z: u16) -> bool {
    self.chunk.is_air(x, y, z)
  }

  fn get(&self, x: u16, y: u16, z: u16) -> u8 {
    self.chunk.get(x, y, z)
  }
}

impl ChunkifyMut<u16, u8> for MetaChunk {
  fn set(&mut self, x: u16, y: u16, z: u16, value: u8) {
    self.chunk.set(x, y, z, value)
  }
}

impl Atlasify<u16, u8> for MetaChunk {
  fn get_atlas(&self, x: u16, y: u16, z: u16) -> u8 {
    self.chunk.get_atlas(x, y, z)
  }
}

impl AtlasifyMut<u16, u8> for MetaChunk {
  fn set_atlas(&mut self, x: u16, y: u16, z: u16, value: u8) {
    self.chunk.set_atlas(x, y, z, value)
  }
}

impl Positionable<f32> for MetaChunk {
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

impl Sizable<u16> for MetaChunk {
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
