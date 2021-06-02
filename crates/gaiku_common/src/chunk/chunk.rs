#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
  boxify::*,
  chunk::{Atlasify, AtlasifyMut, Chunkify, ChunkifyMut},
};

/// Provides a `Chunkify` implementation with index and value support `u8` and an atlas value of `u8`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Chunk {
  position: [f32; 3],
  width: u16,
  height: u16,
  depth: u16,
  values: Vec<(u8, u8)>,
  bounds: ([u16; 3], [u16; 3]),
}

impl Chunk {
  fn index(&self, x: u16, y: u16, z: u16) -> usize {
    x as usize
      + y as usize * self.width as usize
      + z as usize * self.width as usize * self.height as usize
  }

  pub fn values(&self) -> Vec<(u8, u8)> {
    self.values.clone()
  }

  // TODO: This will add  the neighbor data at the border of the chunk, so we can calculate correctly  the normals, heights, etc without need to worry to query each time to get that data
  pub fn update_neighbor_data(&self, _neighbor: &Chunk) {
    unimplemented!();
  }
}

impl Boxify<f32, u16> for Chunk {
  fn new(position: [f32; 3], width: u16, height: u16, depth: u16) -> Self {
    Self {
      position,
      width,
      height,
      depth,
      values: vec![(0, 0); depth as usize * height as usize * width as usize],
      bounds: ([0, 0, 0], [width - 1, depth - 1, height - 1]),
    }
  }
}

impl Chunkify<u16, u8> for Chunk {
  fn is_air(&self, x: u16, y: u16, z: u16) -> bool {
    if x >= self.width || y >= self.height || z >= self.depth {
      true
    } else {
      self.get(x, y, z) == 0
    }
  }

  fn get(&self, x: u16, y: u16, z: u16) -> u8 {
    self.values[self.index(x, y, z)].1
  }
}

impl ChunkifyMut<u16, u8> for Chunk {
  fn set(&mut self, x: u16, y: u16, z: u16, value: u8) {
    let index = self.index(x, y, z);
    self.values[index] = (self.values[index].0, value);
  }
}

impl Atlasify<u16, u8> for Chunk {
  fn get_atlas(&self, x: u16, y: u16, z: u16) -> u8 {
    let index = self.index(x, y, z);
    self.values[index].0
  }
}

impl AtlasifyMut<u16, u8> for Chunk {
  fn set_atlas(&mut self, x: u16, y: u16, z: u16, value: u8) {
    let index = self.index(x, y, z);
    self.values[index] = (value, self.values[index].1);
  }
}

impl Positionable<f32> for Chunk {
  fn with_position(position: [f32; 3]) -> Self {
    Self::new(position, 16, 16, 16)
  }

  fn position(&self) -> [f32; 3] {
    self.position
  }
}

impl Sizable<u16> for Chunk {
  fn with_size(width: u16, height: u16, depth: u16) -> Self {
    Self::new([0.0, 0.0, 0.0], width, height, depth)
  }

  fn depth(&self) -> u16 {
    self.depth
  }

  fn height(&self) -> u16 {
    self.height
  }

  fn width(&self) -> u16 {
    self.width
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn check_index() {
    let chunk = Chunk::new([0.0, 0.0, 0.0], 4, 4, 4);
    let index = chunk.index(1, 2, 3);
    assert_eq!(index, 57);

    let chunk = Chunk::new([0.0, 0.0, 0.0], 4, 5, 6);
    let index = chunk.index(1, 2, 3);
    assert_eq!(index, 69);
  }
}
