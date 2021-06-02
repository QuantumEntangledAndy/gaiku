#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::{
  boxify::*,
  chunk::{Chunkify, ChunkifyMut},
};

/// Provides a `Chunkify` implementation with a hashmap and `u8` position based on x, y and z axis with `u8` value.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SparseChunk {
  width: u16,
  height: u16,
  depth: u16,
  data: HashMap<(u16, u16, u16), u8>,
}

impl Chunkify<u16, u8> for SparseChunk {
  fn is_air(&self, x: u16, y: u16, z: u16) -> bool {
    self.get(x, y, z) == 0
  }

  fn get(&self, x: u16, y: u16, z: u16) -> u8 {
    *self.data.get(&(x, y, z)).unwrap_or(&0)
  }
}

impl Sizable<u16> for SparseChunk {
  fn with_size(width: u16, height: u16, depth: u16) -> Self {
    Self {
      width,
      height,
      depth,
      data: HashMap::new(),
    }
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

impl ChunkifyMut<u16, u8> for SparseChunk {
  fn set(&mut self, x: u16, y: u16, z: u16, value: u8) {
    self.data.insert((x, y, z), value);
  }
}
