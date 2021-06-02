use gaiku_common::{prelude::*, Result};

/// Implementation of a naive heightmap terrain generation.
pub struct HeightMapBaker;

impl Baker for HeightMapBaker {
  type Coord = u16;
  type Value = u8;
  type AtlasValue = u8;

  fn bake<C, T, M>(chunk: &C, _options: &BakerOptions<T>) -> Result<Option<M>>
  where
    C: Chunkify<Self::Coord, Self::Value> + Sizable<Self::Coord>,
    T: Texturify2d,
    M: Meshify,
  {
    let mut builder = MeshBuilder::create(
      [
        chunk.width() as f32 / 2.0,
        256. / 2.0,
        chunk.height() as f32 / 2.0,
      ],
      [chunk.width() as f32, 256., chunk.height() as f32],
    );

    for x in 0..chunk.width() as usize - 1 {
      for y in 0..chunk.height() as usize - 1 {
        if chunk.is_air(x as Self::Coord, y as Self::Coord, 0) {
          continue;
        }

        let fx = x as f32;
        let fz = y as f32;

        let lb = chunk.get(x as Self::Coord, y as Self::Coord, 0) as f32 / 255.0;
        let lf = chunk.get(x as Self::Coord, y as Self::Coord + 1, 0) as f32 / 255.0;
        let rb = chunk.get(x as Self::Coord + 1, y as Self::Coord, 0) as f32 / 255.0;
        let rf = chunk.get(x as Self::Coord + 1, y as Self::Coord + 1, 0) as f32 / 255.0;

        let left_back = [fx, lb, fz];
        let right_back = [fx + 1.0, rb, fz];
        let right_front = [fx + 1.0, rf, fz + 1.0];
        let left_front = [fx, lf, fz + 1.0];

        builder.add_triangle([left_front, right_back, left_back], None, None, 0);
        builder.add_triangle([right_front, right_back, left_front], None, None, 0);
      }
    }

    Ok(builder.build::<M>())
  }
}
