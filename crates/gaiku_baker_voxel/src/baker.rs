use super::common::*;
use gaiku_common::{prelude::*, Result};

use std::convert::TryInto;

pub struct VoxelBaker;

// TODO: Optimize, don't create faces between chunks if there's a non empty voxel
impl Baker for VoxelBaker {
  type Value = f32;
  type AtlasValue = u8;

  fn bake<C, T, M>(chunk: &C, options: &BakerOptions<T>) -> Result<Option<M>>
  where
    C: Chunkify<Self::Value> + Sizable + Atlasify<Self::AtlasValue>,
    T: Texturify2d,
    M: Meshify,
  {
    type Coord = usize;
    let chunk_width = chunk.width();
    let chunk_height = chunk.height();
    let chunk_depth = chunk.depth();
    let mut builder = MeshBuilder::create(
      [
        chunk_width as f32 / 2.0,
        chunk_height as f32 / 2.0,
        chunk_depth as f32 / 2.0,
      ],
      [chunk_width as f32, chunk_height as f32, chunk_depth as f32],
    );

    let isovalue = options.isovalue;

    let x_limit = chunk_width as usize - 1;
    let y_limit = chunk_height as usize - 1;
    let z_limit = chunk_depth as usize - 1;

    for x in 0..x_limit {
      let x = x as Coord;
      let fx = x as f32;
      for y in 0..y_limit {
        let y = y as Coord;
        let fy = y as f32;
        for z in 0..z_limit {
          let z = z as Coord;
          let fz = z as f32;

          let air_check = [
            chunk.is_air(x, y, z),
            chunk.is_air(x + 1, y, z),
            chunk.is_air(x + 1, y + 1, z),
            chunk.is_air(x, y + 1, z),
            chunk.is_air(x, y, z + 1),
            chunk.is_air(x + 1, y, z + 1),
            chunk.is_air(x + 1, y + 1, z + 1),
            chunk.is_air(x, y + 1, z + 1),
          ];
          if air_check.iter().all(|&v| v == false) || air_check.iter().all(|&v| v == true) {
            continue;
          }

          let grid = GridCell {
            value: [
              chunk.get(x, y, z) as f32,
              chunk.get(x + 1, y, z) as f32,
              chunk.get(x + 1, y + 1, z) as f32,
              chunk.get(x, y + 1, z) as f32,
              chunk.get(x, y, z + 1) as f32,
              chunk.get(x + 1, y, z + 1) as f32,
              chunk.get(x + 1, y + 1, z + 1) as f32,
              chunk.get(x, y + 1, z + 1) as f32,
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

          let polys = grid.polygonize(isovalue);

          for (vertex, face_uvs, corner) in polys {
            let normal = compute_normal(&vertex);

            // Get atlas at this corner_idx
            let atlas = match corner {
              0 => chunk.get_atlas(x, y, z),
              1 => chunk.get_atlas(x + 1, y, z),
              2 => chunk.get_atlas(x + 1, y + 1, z),
              3 => chunk.get_atlas(x, y + 1, z),
              4 => chunk.get_atlas(x, y, z + 1),
              5 => chunk.get_atlas(x + 1, y, z + 1),
              6 => chunk.get_atlas(x + 1, y + 1, z + 1),
              7 => chunk.get_atlas(x, y + 1, z + 1),
              _ => unreachable!(),
            };

            let uvs = if let Some(texture) = &options.texture {
              // Get the atlas corners
              // 3-2
              // 0-1
              let uvs = texture.get_uv(atlas);

              let atlas_origin = uvs.0;
              let atlas_dimensions = [uvs.2[0] - uvs.0[0], uvs.2[1] - uvs.0[1]];
              // Put face uvs into atlas uv space
              let final_uvs: [[f32; 2]; 3] = face_uvs
                .iter()
                .map(|uv| {
                  [
                    atlas_origin[0] + uv[0] * atlas_dimensions[0],
                    atlas_origin[1] + uv[1] * atlas_dimensions[1],
                  ]
                })
                .collect::<Vec<[f32; 2]>>()
                .try_into()
                .unwrap();
              Some(final_uvs)
            } else {
              None
            };

            builder.add_triangle(
              vertex,       // triangle
              Some(normal), // normal
              uvs,          // uv
              atlas.into(), // atlas
            );
          }
        }
      }
    }

    Ok(builder.build::<M>())
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use gaiku_common::{chunk::Chunk, mesh::Mesh, texture::Texture2d};

  #[test]
  fn simple_test() {
    let options = Default::default();
    let mut chunk = Chunk::new([0.0, 0.0, 0.0], 3, 3, 3);

    chunk.set(1, 1, 1, 1.);
    chunk.set_atlas(1, 1, 1, 0);

    let mesh = VoxelBaker::bake::<Chunk, Texture2d, Mesh>(&chunk, &options)
      .unwrap()
      .unwrap();

    let positions_count = mesh.get_positions().len();
    let indices_count = mesh.get_indices().len();

    assert_eq!(indices_count, 144);
    assert_eq!(positions_count, 78);
  }
}