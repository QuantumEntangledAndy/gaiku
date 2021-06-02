use gaiku_common::{prelude::*, Result};

use image::load_from_memory;

/// Converts a `png` file to 2d chunk data.
pub struct PNGReader;

impl FileFormat for PNGReader {
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
    let mut result = vec![];
    let img = load_from_memory(&bytes)?.into_luma8();

    assert!(img.width() <= u16::MAX as u32);
    assert!(img.height() <= u16::MAX as u32);

    let mut chunk = C::new(
      [0.0, 0.0, 0.0],
      img.width() as Self::Coord,
      img.height() as Self::Coord,
      1,
    );

    for x in 0..img.width() as u32 {
      for y in 0..img.height() as u32 {
        let color = img.get_pixel(x, y).0[0];
        chunk.set(x as Self::Coord, y as Self::Coord, 0, color);
        chunk.set_atlas(x as Self::Coord, y as Self::Coord, 0, color);
      }
    }

    result.push(chunk);

    Ok((result, None))
  }
}
