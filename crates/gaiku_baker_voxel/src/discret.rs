use super::common::*;
use gaiku_common::{prelude::*, Result};

use std::convert::TryInto;

pub struct VoxelBaker;

// TODO: Optimize, don't create faces between chunks if there's a non empty voxel
impl Baker for VoxelBaker {
  type Coord = u16;
  type Value = u8;
  type AtlasValue = u8;

  fn bake<C, T, M>(chunk: &C, options: &BakerOptions<T>) -> Result<Option<M>>
  where
    C: Chunkify<Self::Coord, Self::Value>
      + Sizable<Self::Coord>
      + Atlasify<Self::Coord, Self::AtlasValue>,
    T: Texturify2d,
    M: Meshify,
  {
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
      let x = x as Self::Coord;
      let fx = x as f32;
      for y in 0..y_limit {
        let y = y as Self::Coord;
        let fy = y as f32;
        for z in 0..z_limit {
          let z = z as Self::Coord;
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

          let triangles = grid.polygonize(isovalue);

          for vertex in triangles {
            let normal = compute_normal(&vertex);

            let face_mid = vec_ave(vec![&vertex[0], &vertex[1], &vertex[2]]);
            // Move the face center backwards by small amount relative to normal
            let epsilon = vec_mult(&normal, -EPSILON);
            let face_mid_eps = vec_add(&face_mid, &epsilon);
            // Get the corner of the grid nearest to the face
            let corner_idx = grid.nearest_corner(&face_mid_eps);
            // Get atlas at this corner_idx
            let atlas = match corner_idx {
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
              let face_uvs = grid.compute_uvs(&vertex);
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
    let mut chunk = Chunk::new([0.0, 0.0, 0.0], 2, 2, 2);

    chunk.set(0, 0, 0, (0, 1));

    let mesh = VoxelBaker::bake::<Chunk, Texture2d, Mesh>(&chunk, &options)
      .unwrap()
      .unwrap();

    let positions_count = mesh.get_positions().len();
    let indices_count = mesh.get_indices().len();

    assert_eq!(indices_count, 36);
    assert_eq!(positions_count, 24);
  }
}
