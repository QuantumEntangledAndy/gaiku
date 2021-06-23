use gaiku_common::{prelude::*, Result};
use std::convert::TryInto;

use gox::{Block, Data, Gox, Only};

/// Converts a `gox` file to 3d chunk data.
pub struct GoxReader;

// TODO: The generated data appears rotated, need to rotate from back to bottom
impl FileFormat for GoxReader {
  type Value = u8;
  type AtlasValue = u8;
  type Coord = u16;
  type OriginCoord = f32;

  fn load<C, T>(bytes: Vec<u8>) -> Result<(Vec<C>, Option<TextureAtlas2d<T>>)>
  where
    C: Chunkify<Self::Coord, Self::Value>
      + ChunkifyMut<Self::Coord, Self::Value>
      + Boxify<Self::OriginCoord, Self::Coord>
      + AtlasifyMut<Self::Coord, Self::AtlasValue>,
    T: Texturify2d,
  {
    let gox = Gox::from_bytes(bytes, vec![Only::Layers, Only::Blocks]);
    let mut colors: Vec<[u8; 4]> = Vec::with_capacity(255);
    let mut result = vec![];
    let mut block_data: Vec<&Block> = vec![];

    for data in gox.data.iter() {
      if let Data::Blocks(data) = &data {
        block_data.push(data);
      }
    }

    for data in gox.data.iter() {
      if let Data::Layers(layers, _bounds) = &data {
        for layer in layers.iter() {
          if !layer.blocks.is_empty() {
            for data in layer.blocks.iter() {
              let border: usize = 1;
              let block_colors = block_data[data.block_index];
              let mut chunk = C::new(
                [
                  (data.x - border as i32) as Self::OriginCoord,
                  (data.z - border as i32) as Self::OriginCoord,
                  (data.y - border as i32) as Self::OriginCoord,
                ],
                (16 + border * 2) as Self::Coord,
                (16 + border * 2) as Self::Coord,
                (16 + border * 2) as Self::Coord,
              );

              for x in 0..chunk.width() as usize - border * 2 {
                for y in 0..chunk.height() as usize - border * 2 {
                  for z in 0..chunk.depth() as usize - border * 2 {
                    if !block_colors.is_empty(x, y, z) {
                      let color = block_colors.get_pixel(x, y, z);
                      let index = if let Some((index, _)) =
                        colors.iter().enumerate().find(|(_, value)| {
                          value[0] == color[0]
                            && value[1] == color[1]
                            && value[2] == color[2]
                            && value[3] == color[3]
                        }) {
                        index
                      } else {
                        let index = colors.len();
                        colors.push(color);
                        index
                      };

                      if index <= std::u8::MAX as usize {
                        chunk.set(
                          (x + border) as Self::Coord,
                          (z + border) as Self::Coord,
                          (y + border) as Self::Coord,
                          1,
                        );
                        chunk.set_atlas(
                          (x + border) as Self::Coord,
                          (z + border) as Self::Coord,
                          (y + border) as Self::Coord,
                          index as Self::AtlasValue,
                        );
                      }
                    }
                  }
                }
              }

              result.push(chunk);
            }
          }
        }
      }
    }

    if !colors.is_empty() {
      let mut atlas = TextureAtlas2d::new(1);

      for (index, color) in colors.iter().enumerate() {
        // colors should limited to 255 so (index.try_into().unwrap()) should fit into u8 for set_at_index
        atlas.fill_at_index(index.try_into().unwrap(), *color);
      }

      Ok((result, Some(atlas)))
    } else {
      Ok((result, None))
    }
  }
}
