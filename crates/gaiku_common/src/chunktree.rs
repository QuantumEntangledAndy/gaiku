pub struct ChunkTreeLeaf<Data> {
  /// Bounds represents the desired size of the domain in (min, max)
  bounds: ([f32; 3], [f32; 3]),

  /// Arbitary data for this LOD level
  data: Option<Data>,

  /// LOD Level 0 is highest detail
  level: usize,
}

/// This is an octtree holds chunks at different LODs
/// The root of the tree is the whole terrain at the
/// lowest LOD
/// Subsequent levels hold the higher detail LODS at higher
// resolutions, up to the number of desired LODS.
// Values are held in a leaf so that we can iter them.
// Originally both children and data were on the same struct
// but this causes issues with who owns the data during an iter
pub struct ChunkTree<Data> {
  value: ChunkTreeLeaf<Data>,
  /// Octtree children with order
  /// bottom_front_left
  /// bottom_front_right
  /// bottom_back_right
  /// bottom_back_left
  /// top_front_left
  /// top_front_right
  /// top_back_right
  /// top_back_left
  children: Vec<ChunkTree<Data>>,
}

impl<Data> ChunkTree<Data> {
  pub fn get_bounds(&self) -> &([f32; 3], [f32; 3]) {
    self.value.get_bounds()
  }

  pub fn set_data(&mut self, data: Data) {
    self.value.set_data(data)
  }

  pub fn get_data(&self) -> Option<&Data> {
    self.value.get_data()
  }

  pub fn get_data_mut(&mut self) -> Option<&mut Data> {
    self.value.get_data_mut()
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

  pub fn new(bounds: ([f32; 3], [f32; 3]), levels: usize) -> ChunkTree<Data> {
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

  /// An iter on the visible LODs
  /// lod_distance represents a scaling factor for the lod transition point
  /// The lods transition at lod_distance*(2^(level))
  // TODO: add an iter for lod_distance*(a^(level)) where a is an input variable
  //       this would be nice as then we can get more control
  pub fn get_visible_lods<'b>(
    &self,
    camera_position: &'b [f32; 3],
    lod_distance: f32,
  ) -> ChunkTreeVisibleIter<'_, 'b, Data> {
    ChunkTreeVisibleIter::new(self, camera_position, lod_distance)
  }

  /// A mutable iter on the visible LODs
  /// lod_distance represents a scaling factor for the lod transition point
  /// The lods transition at lod_distance*(2^(level))
  pub fn get_visible_lods_mut<'b>(
    &mut self,
    camera_position: &'b [f32; 3],
    lod_distance: f32,
  ) -> ChunkTreeVisibleIterMut<'_, 'b, Data> {
    ChunkTreeVisibleIterMut::new(self, camera_position, lod_distance)
  }

  /// Given a path such as [0,1,5] get the child node at
  /// self.children[0].children[1].children[5]
  pub fn at_path(&self, path: &[usize]) -> Option<&ChunkTreeLeaf<Data>> {
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

  /// Given a path such as [0,1,5] get the child node at
  /// self.children[0].children[1].children[5] mutably
  pub fn at_path_mut(&mut self, path: &[usize]) -> Option<&mut ChunkTreeLeaf<Data>> {
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

  /// Iters over all leafs in the tree
  pub fn iter(&self) -> ChunkTreeIter<'_, Data> {
    ChunkTreeIter::new(self)
  }

  pub fn iter_mut(&mut self) -> ChunkTreeIterMut<'_, Data> {
    ChunkTreeIterMut::new(self)
  }
}

impl<Data> ChunkTreeLeaf<Data> {
  pub fn get_bounds(&self) -> &([f32; 3], [f32; 3]) {
    &self.bounds
  }

  pub fn set_data(&mut self, data: Data) {
    self.data = Some(data);
  }

  pub fn get_data(&self) -> Option<&Data> {
    self.data.as_ref()
  }

  pub fn get_data_mut(&mut self) -> Option<&mut Data> {
    self.data.as_mut()
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

  fn new(bounds: ([f32; 3], [f32; 3]), levels: usize) -> ChunkTreeLeaf<Data> {
    let data: Option<Data> = None;

    let leaf = ChunkTreeLeaf {
      bounds: bounds,
      data,
      level: levels,
    };
    leaf
  }
}

pub struct ChunkTreeIter<'a, Data> {
  stack: Vec<&'a ChunkTree<Data>>,
}

impl<'a, Data> ChunkTreeIter<'a, Data> {
  fn new(tree: &'a ChunkTree<Data>) -> ChunkTreeIter<'a, Data> {
    ChunkTreeIter { stack: vec![tree] }
  }
}

impl<'a, Data> Iterator for ChunkTreeIter<'a, Data> {
  type Item = &'a ChunkTreeLeaf<Data>;

  fn next(&mut self) -> Option<Self::Item> {
    let node = self.stack.pop()?;
    for child in node.children.iter() {
      self.stack.push(child);
    }
    return Some(&node.value);
  }
}

pub struct ChunkTreeIterMut<'a, Data> {
  stack: Vec<&'a mut ChunkTree<Data>>,
}

impl<'a, Data> ChunkTreeIterMut<'a, Data> {
  fn new(tree: &'a mut ChunkTree<Data>) -> ChunkTreeIterMut<'a, Data> {
    ChunkTreeIterMut { stack: vec![tree] }
  }
}

impl<'a, Data> Iterator for ChunkTreeIterMut<'a, Data> {
  type Item = &'a mut ChunkTreeLeaf<Data>;

  fn next(&mut self) -> Option<Self::Item> {
    let node = self.stack.pop()?;
    for child in node.children.iter_mut() {
      self.stack.push(child);
    }
    return Some(&mut node.value);
  }
}

pub struct ChunkTreeVisibleIter<'a, 'b, Data> {
  stack: Vec<&'a ChunkTree<Data>>,
  camera_position: &'b [f32; 3],
  lod_distance: f32,
}

impl<'a, 'b, Data> ChunkTreeVisibleIter<'a, 'b, Data> {
  fn new(
    tree: &'a ChunkTree<Data>,
    camera_position: &'b [f32; 3],
    lod_distance: f32,
  ) -> ChunkTreeVisibleIter<'a, 'b, Data> {
    ChunkTreeVisibleIter {
      stack: vec![tree],
      camera_position,
      lod_distance,
    }
  }
}

impl<'a, 'b, Data> Iterator for ChunkTreeVisibleIter<'a, 'b, Data> {
  type Item = &'a ChunkTreeLeaf<Data>;

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

pub struct ChunkTreeVisibleIterMut<'a, 'b, Data> {
  stack: Vec<&'a mut ChunkTree<Data>>,
  camera_position: &'b [f32; 3],
  lod_distance: f32,
}

impl<'a, 'b, Data> ChunkTreeVisibleIterMut<'a, 'b, Data> {
  fn new(
    tree: &'a mut ChunkTree<Data>,
    camera_position: &'b [f32; 3],
    lod_distance: f32,
  ) -> ChunkTreeVisibleIterMut<'a, 'b, Data> {
    ChunkTreeVisibleIterMut {
      stack: vec![tree],
      camera_position,
      lod_distance,
    }
  }
}

impl<'a, 'b, Data> Iterator for ChunkTreeVisibleIterMut<'a, 'b, Data> {
  type Item = &'a mut ChunkTreeLeaf<Data>;

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
