use crate::prelude::*;

use std::marker::PhantomData;

pub struct ChunkTreeLeaf<Chunk, Data> {
  // required because we want to have Chunk<Data>
  _marker: PhantomData<Data>,

  /// Bounds represents the desired size of the domain in (min, max)
  bounds: ([f32; 3], [f32; 3]),

  /// The chunk data for this LOD level
  chunk: Option<Chunk>,

  /// This bool controls if this LOD is visible according to the desired screen space error
  /// It gets updated by calling update_lod_visibility
  visible: bool,

  /// LOD Level 0 is highest detail
  level: usize,
}

/// This is an octtree holds chunks at different LODs
/// The root of the tree is the whole terrain at the
/// lowest LOD
/// Subsequent levels hold the higher detail LODS at higher
// resolutions, up to the number of desired LODS.
pub struct ChunkTree<Chunk, Data> {
  value: ChunkTreeLeaf<Chunk, Data>,
  /// Octtree children with order
  /// bottom_front_left
  /// bottom_front_right
  /// bottom_back_right
  /// bottom_back_left
  /// top_front_left
  /// top_front_right
  /// top_back_right
  /// top_back_left
  children: Vec<ChunkTree<Chunk, Data>>,
}

impl<Chunk, Data> ChunkTree<Chunk, Data>
where
  Chunk: Chunkify<Data>,
{
  pub fn get_bounds(&self) -> &([f32; 3], [f32; 3]) {
    self.value.get_bounds()
  }

  pub fn set_chunk(&mut self, chunk: Chunk) {
    self.value.set_chunk(chunk)
  }

  pub fn get_chunk(&self) -> Option<&Chunk> {
    self.value.get_chunk()
  }

  pub fn get_chunk_mut(&mut self) -> Option<&mut Chunk> {
    self.value.get_chunk_mut()
  }

  pub fn get_center(&self) -> [f32; 3] {
    self.value.get_center()
  }

  pub fn get_size(&self) -> [f32; 3] {
    self.value.get_size()
  }

  /// Gets the origin (min) of the leaf
  pub fn get_origin(&self) -> [f32; 3] {
    self.value.get_origin()
  }

  pub fn get_level(&self) -> usize {
    self.value.get_level()
  }

  pub fn is_visible(&self) -> bool {
    self.value.is_visible()
  }

  pub fn new(bounds: ([f32; 3], [f32; 3]), levels: usize) -> ChunkTree<Chunk, Data> {
    let new_children = if levels > 0 {
      let new_levels = levels - 1;
      let (min, max) = bounds;
      let minx = min[0];
      let maxx = max[0];
      let half_x = (min[0] + max[0]) / 2.;
      let miny = min[1];
      let maxy = max[1];
      let half_y = (min[1] + max[1]) / 2.;
      let minz = min[2];
      let maxz = max[2];
      let half_z = (min[2] + max[2]) / 2.;
      vec![
        ChunkTree::new(([minx, miny, half_z], [half_x, half_y, maxz]), new_levels),
        ChunkTree::new(([half_x, miny, half_z], [maxx, half_y, maxz]), new_levels),
        ChunkTree::new(([half_x, miny, minz], [maxx, half_y, half_z]), new_levels),
        ChunkTree::new(([minx, miny, minz], [half_x, half_y, half_z]), new_levels),
        ChunkTree::new(([minx, half_y, half_z], [half_x, maxy, maxz]), new_levels),
        ChunkTree::new(([half_x, half_y, half_z], [maxx, maxy, maxz]), new_levels),
        ChunkTree::new(([half_x, half_y, minz], [maxx, maxy, half_z]), new_levels),
        ChunkTree::new(([minx, half_y, minz], [half_x, maxy, half_z]), new_levels),
      ]
    } else {
      vec![]
    };
    let value = ChunkTreeLeaf::new(bounds, levels);
    ChunkTree {
      value,
      children: new_children,
    }
  }

  /// This works but is really slow for large LODs
  // For example if there are 8 levels there are 8^8=16,777,216 LOD0's
  // A better algorithm would not need to visit all lods
  pub fn update_lod_visibility(&mut self, camera_position: &[f32; 3], lod_distance: f32) {
    for child in self.iter_mut() {
      let center = child.get_center();
      let d = ((center[0] - camera_position[0]).powi(2)
        + (center[1] - camera_position[1]).powi(2)
        + (center[2] - camera_position[2]).powi(2))
      .sqrt();
      let lod: usize = ((d / lod_distance).ln() / 2_f32.ln()).floor() as usize;
      child.visible = child.get_level() == lod;
    }
  }

  /// This is a more effecient way of getting the visible LODs then calling
  /// update_lod_visibility as it dosent need to visit every node
  pub fn get_visible_lods<'b>(
    &self,
    camera_position: &'b [f32; 3],
    lod_distance: f32,
  ) -> ChunkTreeVisibleIter<'_, 'b, Chunk, Data> {
    ChunkTreeVisibleIter::new(self, camera_position, lod_distance)
  }

  pub fn get_visible_lods_mut<'b>(
    &mut self,
    camera_position: &'b [f32; 3],
    lod_distance: f32,
  ) -> ChunkTreeVisibleIterMut<'_, 'b, Chunk, Data> {
    ChunkTreeVisibleIterMut::new(self, camera_position, lod_distance)
  }

  pub fn at_path(&self, path: &[usize]) -> Option<&ChunkTreeLeaf<Chunk, Data>> {
    if path.len() == 0 {
      Some(&self.value)
    } else {
      let child_id = path[0];
      if let Some(child) = self.children.get(child_id) {
        child.at_path(&path[1..])
      } else {
        None
      }
    }
  }

  pub fn at_path_mut(&mut self, path: &[usize]) -> Option<&mut ChunkTreeLeaf<Chunk, Data>> {
    if path.len() == 0 {
      Some(&mut self.value)
    } else {
      let child_id = path[0];
      if let Some(child) = self.children.get_mut(child_id) {
        child.at_path_mut(&path[1..])
      } else {
        None
      }
    }
  }

  pub fn iter(&self) -> ChunkTreeIter<'_, Chunk, Data> {
    ChunkTreeIter::new(self)
  }

  pub fn iter_mut(&mut self) -> ChunkTreeIterMut<'_, Chunk, Data> {
    ChunkTreeIterMut::new(self)
  }
}

impl<Chunk, Data> ChunkTreeLeaf<Chunk, Data>
where
  Chunk: Chunkify<Data>,
{
  pub fn get_bounds(&self) -> &([f32; 3], [f32; 3]) {
    &self.bounds
  }

  pub fn set_chunk(&mut self, chunk: Chunk) {
    self.chunk = Some(chunk);
  }

  pub fn get_chunk(&self) -> Option<&Chunk> {
    self.chunk.as_ref()
  }

  pub fn get_chunk_mut(&mut self) -> Option<&mut Chunk> {
    self.chunk.as_mut()
  }

  pub fn get_center(&self) -> [f32; 3] {
    let (min, max) = self.bounds;
    [
      min[0] + max[0] / 2.,
      min[1] + max[1] / 2.,
      min[2] + max[2] / 2.,
    ]
  }

  pub fn get_size(&self) -> [f32; 3] {
    let (min, max) = self.bounds;
    [max[0] - min[0], max[1] - min[1], max[2] - min[2]]
  }

  /// Gets the origin (min) of the leaf
  pub fn get_origin(&self) -> [f32; 3] {
    let (min, _) = self.bounds;
    [min[0], min[1], min[2]]
  }

  pub fn get_level(&self) -> usize {
    self.level
  }

  pub fn is_visible(&self) -> bool {
    self.visible
  }

  fn new(bounds: ([f32; 3], [f32; 3]), levels: usize) -> ChunkTreeLeaf<Chunk, Data> {
    let chunk: Option<Chunk> = None;

    let leaf = ChunkTreeLeaf {
      bounds: bounds,
      chunk,
      _marker: PhantomData,
      visible: false,
      level: levels,
    };
    leaf
  }
}

pub struct ChunkTreeIter<'a, Chunk, Data> {
  stack: Vec<&'a ChunkTree<Chunk, Data>>,
}

impl<'a, Chunk, Data> ChunkTreeIter<'a, Chunk, Data>
where
  Chunk: Chunkify<Data>,
{
  fn new(tree: &'a ChunkTree<Chunk, Data>) -> ChunkTreeIter<'a, Chunk, Data> {
    ChunkTreeIter { stack: vec![tree] }
  }
}

impl<'a, Chunk, Data> Iterator for ChunkTreeIter<'a, Chunk, Data>
where
  Chunk: Chunkify<Data>,
{
  type Item = &'a ChunkTreeLeaf<Chunk, Data>;

  fn next(&mut self) -> Option<Self::Item> {
    let node = self.stack.pop()?;
    for child in node.children.iter() {
      self.stack.push(child);
    }
    return Some(&node.value);
  }
}

pub struct ChunkTreeIterMut<'a, Chunk, Data> {
  stack: Vec<&'a mut ChunkTree<Chunk, Data>>,
}

impl<'a, Chunk, Data> ChunkTreeIterMut<'a, Chunk, Data>
where
  Chunk: Chunkify<Data>,
{
  fn new(tree: &'a mut ChunkTree<Chunk, Data>) -> ChunkTreeIterMut<'a, Chunk, Data> {
    ChunkTreeIterMut { stack: vec![tree] }
  }
}

impl<'a, Chunk, Data> Iterator for ChunkTreeIterMut<'a, Chunk, Data>
where
  Chunk: Chunkify<Data>,
{
  type Item = &'a mut ChunkTreeLeaf<Chunk, Data>;

  fn next(&mut self) -> Option<Self::Item> {
    let node = self.stack.pop()?;
    for child in node.children.iter_mut() {
      self.stack.push(child);
    }
    return Some(&mut node.value);
  }
}

pub struct ChunkTreeVisibleIter<'a, 'b, Chunk, Data> {
  stack: Vec<&'a ChunkTree<Chunk, Data>>,
  camera_position: &'b [f32; 3],
  lod_distance: f32,
}

impl<'a, 'b, Chunk, Data> ChunkTreeVisibleIter<'a, 'b, Chunk, Data>
where
  Chunk: Chunkify<Data>,
{
  fn new(
    tree: &'a ChunkTree<Chunk, Data>,
    camera_position: &'b [f32; 3],
    lod_distance: f32,
  ) -> ChunkTreeVisibleIter<'a, 'b, Chunk, Data> {
    ChunkTreeVisibleIter {
      stack: vec![tree],
      camera_position,
      lod_distance,
    }
  }
}

impl<'a, 'b, Chunk, Data> Iterator for ChunkTreeVisibleIter<'a, 'b, Chunk, Data>
where
  Chunk: Chunkify<Data>,
{
  type Item = &'a ChunkTreeLeaf<Chunk, Data>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let node = self.stack.pop()?;
      if node.children.len() > 0 {
        let center = node.get_center();
        let d = ((center[0] - self.camera_position[0]).powi(2)
          + (center[1] - self.camera_position[1]).powi(2)
          + (center[2] - self.camera_position[2]).powi(2))
        .sqrt();
        let lod: usize = ((d / self.lod_distance).ln() / 2_f32.ln()).floor() as usize;
        if node.get_level() <= lod {
          return Some(&node.value);
        } else {
          for child in node.children.iter() {
            self.stack.push(child);
          }
        }
      } else {
        return Some(&node.value);
      }
    }
  }
}

pub struct ChunkTreeVisibleIterMut<'a, 'b, Chunk, Data> {
  stack: Vec<&'a mut ChunkTree<Chunk, Data>>,
  camera_position: &'b [f32; 3],
  lod_distance: f32,
}

impl<'a, 'b, Chunk, Data> ChunkTreeVisibleIterMut<'a, 'b, Chunk, Data>
where
  Chunk: Chunkify<Data>,
{
  fn new(
    tree: &'a mut ChunkTree<Chunk, Data>,
    camera_position: &'b [f32; 3],
    lod_distance: f32,
  ) -> ChunkTreeVisibleIterMut<'a, 'b, Chunk, Data> {
    ChunkTreeVisibleIterMut {
      stack: vec![tree],
      camera_position,
      lod_distance,
    }
  }
}

impl<'a, 'b, Chunk, Data> Iterator for ChunkTreeVisibleIterMut<'a, 'b, Chunk, Data>
where
  Chunk: Chunkify<Data>,
{
  type Item = &'a mut ChunkTreeLeaf<Chunk, Data>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let node = self.stack.pop()?;
      if node.children.len() > 0 {
        let center = node.get_center();
        let d = ((center[0] - self.camera_position[0]).powi(2)
          + (center[1] - self.camera_position[1]).powi(2)
          + (center[2] - self.camera_position[2]).powi(2))
        .sqrt();
        let lod: usize = ((d / self.lod_distance).ln() / 2_f32.ln()).floor() as usize;
        if node.get_level() <= lod {
          return Some(&mut node.value);
        } else {
          for child in node.children.iter_mut() {
            self.stack.push(child);
          }
        }
      } else {
        return Some(&mut node.value);
      }
    }
  }
}
