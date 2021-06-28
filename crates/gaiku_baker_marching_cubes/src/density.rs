use super::common::*;
use gaiku_common::{prelude::*, Result};

/// Implementation of the marching cubes terrain generation.
pub struct DensityMarchingCubesBaker;

impl Baker for DensityMarchingCubesBaker {
  type Value = f32;
  type Coord = f32;
  type AtlasValue = u8;

  fn bake<C, T, M>(chunk: &C, options: &BakerOptions<T>) -> Result<Option<M>>
  where
    C: Chunkify<Self::Coord, Self::Value>
      + Sizable<Self::Coord>
      + Atlasify<Self::Coord, Self::AtlasValue>,
    T: Texturify2d,
    M: Meshify,
  {
    let mut builder = MeshBuilder::create(
      [
        chunk.width() as f32 / 2.0,
        chunk.height() as f32 / 2.0,
        chunk.depth() as f32 / 2.0,
      ],
      [
        chunk.width() as f32,
        chunk.height() as f32,
        chunk.depth() as f32,
      ],
    );

    let isovalue = options.isovalue;

    for x in 0..chunk.width() as usize - 1 {
      let fx = x as f32;
      let x = x as Self::Coord;
      for y in 0..chunk.height() as usize - 1 {
        let fy = y as f32;
        let y = y as Self::Coord;
        for z in 0..chunk.depth() as usize - 1 {
          let fz = z as f32;
          let z = z as Self::Coord;

          let air_check = [
            chunk.is_air(x, y, z),
            chunk.is_air(x + 1., y, z),
            chunk.is_air(x + 1., y + 1., z),
            chunk.is_air(x, y + 1., z),
            chunk.is_air(x, y, z + 1.),
            chunk.is_air(x + 1., y, z + 1.),
            chunk.is_air(x + 1., y + 1., z + 1.),
            chunk.is_air(x, y + 1., z + 1.),
          ];
          if air_check.iter().all(|&v| v == false) || air_check.iter().all(|&v| v == true) {
            continue;
          }

          let grid = GridCell {
            value: [
              chunk.get(x, y, z) as f32,
              chunk.get(x + 1., y, z) as f32,
              chunk.get(x + 1., y + 1., z) as f32,
              chunk.get(x, y + 1., z) as f32,
              chunk.get(x, y, z + 1.) as f32,
              chunk.get(x + 1., y, z + 1.) as f32,
              chunk.get(x + 1., y + 1., z + 1.) as f32,
              chunk.get(x, y + 1., z + 1.) as f32,
            ],
            point: [
              [fx + 0.0, fy + 0.0, fz + 0.0].into(),
              [fx + 1.0, fy + 0.0, fz + 0.0].into(),
              [fx + 1.0, fy + 1.0, fz + 0.0].into(),
              [fx + 0.0, fy + 1.0, fz + 0.0].into(),
              [fx + 0.0, fy + 0.0, fz + 1.0].into(),
              [fx + 1.0, fy + 0.0, fz + 1.0].into(),
              [fx + 1.0, fy + 1.0, fz + 1.0].into(),
              [fx + 0.0, fy + 1.0, fz + 1.0].into(),
            ],
          };

          let triangles = grid.polygonize(isovalue);

          for vertex in triangles {
            let normal = compute_normal(&vertex);
            builder.add_triangle(
              vertex,                          // triangle
              Some(normal),                    // normal
              None,                            // uv
              chunk.get_atlas(x, y, z).into(), // atlas
            );
          }
        }
      }
    }

    Ok(builder.build::<M>())
  }
}
