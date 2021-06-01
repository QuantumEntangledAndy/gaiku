/// This structure holds volume data in a 3d grid.
///
/// This data can be sampled off grid using interpolation.
/// The values at each grid point are a float.
///
/// Gradient infomation is also approximated
///
/// To make a surface using a baker we must sample the isosurface.
use crate::prelude::*;

const EPSILON: f32 = 1e-4;

pub enum InterpolateMethod {
  NearestNeighbor,
  Trilinear,
}

pub struct DensityData {
  /// Raw sample data
  samples: Vec<f32>,

  /// bounds are (min, max)
  bounds: ([f32; 3], [f32; 3]),

  /// Dimensions of samples
  dimensions: [usize; 3],

  /// Value Interpolating method
  value_inter: InterpolateMethod,

  /// Gradient Interpolating method
  grad_inter: InterpolateMethod,
}

impl DensityData {
  pub fn new(
    samples: Vec<f32>,
    bounds: ([f32; 3], [f32; 3]),
    dimensions: [usize; 3],
    value_inter: InterpolateMethod,
    grad_inter: InterpolateMethod,
  ) -> Self {
    assert!(samples.len() == dimensions[0] * dimensions[1] * dimensions[2]);
    Self {
      samples,
      bounds,
      dimensions,
      value_inter,
      grad_inter,
    }
  }

  pub fn empty(dimensions: [usize; 3], bounds: ([f32; 3], [f32; 3])) -> Self {
    Self::new(
      vec![0.; dimensions[0] * dimensions[1] * dimensions[2]],
      bounds,
      dimensions,
      InterpolateMethod::Trilinear,
      InterpolateMethod::NearestNeighbor,
    )
  }

  pub fn from_samples(
    samples: Vec<f32>,
    dimensions: [usize; 3],
    bounds: ([f32; 3], [f32; 3]),
  ) -> Self {
    Self::new(
      samples,
      bounds,
      dimensions,
      InterpolateMethod::Trilinear,
      InterpolateMethod::NearestNeighbor,
    )
  }

  pub fn from_3dsamples(
    samples: Vec<Vec<Vec<f32>>>,
    dimensions: [usize; 3],
    bounds: ([f32; 3], [f32; 3]),
  ) -> Self {
    Self::new(
      samples.into_iter().flatten().flatten().collect(),
      bounds,
      dimensions,
      InterpolateMethod::Trilinear,
      InterpolateMethod::NearestNeighbor,
    )
  }

  pub fn get_sample(&self, i: usize, j: usize, k: usize) -> f32 {
    let dimensions = self.dimensions;
    // println!("{},{},{}: {:?}", i, k, j, dimensions);
    assert!(i < dimensions[0]);
    assert!(j < dimensions[1]);
    assert!(k < dimensions[2]);
    let idx = i * dimensions[1] * dimensions[0] + j * dimensions[0] + k;
    self.samples[idx]
  }

  pub fn set_sample(&mut self, i: usize, j: usize, k: usize, value: f32) {
    let dimensions = self.dimensions;
    assert!(i < dimensions[0]);
    assert!(j < dimensions[1]);
    assert!(k < dimensions[2]);
    let idx = k * dimensions[1] * dimensions[0] + j * dimensions[0] + i;
    self.samples[idx] = value;
  }

  pub fn get_sample_grad(&self, i: usize, j: usize, k: usize) -> [f32; 3] {
    let dimensions = self.dimensions;
    assert!(i < dimensions[0]);
    assert!(j < dimensions[1]);
    assert!(k < dimensions[2]);
    [
      if i == 0 {
        self.get_sample(i + 1, j, k) - self.get_sample(i, j, k)
      } else if i + 1 == self.dimensions[0] {
        self.get_sample(i, j, k) - self.get_sample(i - 1, j, k)
      } else {
        (self.get_sample(i + 1, j, k) - self.get_sample(i - 1, j, k)) / 2.
      },
      if j == 0 {
        self.get_sample(i, j + 1, k) - self.get_sample(i, j, k)
      } else if j + 1 == self.dimensions[1] {
        self.get_sample(i, j + 1, k) - self.get_sample(i, j - 1, k)
      } else {
        (self.get_sample(i, j + 1, k) - self.get_sample(i, j - 1, k)) / 2.
      },
      if k == 0 {
        self.get_sample(i, j, k + 1) - self.get_sample(i, j, k)
      } else if k + 1 == self.dimensions[2] {
        self.get_sample(i, j, k) - self.get_sample(i, j, k - 1)
      } else {
        (self.get_sample(i, j, k + 1) - self.get_sample(i, j, k - 1)) / 2.
      },
    ]
  }

  pub fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
    let (min, max) = self.bounds;
    assert!((x - min[0]) >= -EPSILON);
    assert!((max[0] - x) >= -EPSILON);
    assert!((y - min[1]) >= -EPSILON);
    assert!((max[1] - y) >= -EPSILON);
    assert!((z - min[2]) >= -EPSILON);
    assert!((max[2] - z) >= -EPSILON);

    // println!("x,y,z: {},{},{}", x, y, z);

    // In local grid space
    let x = x - min[0];
    let y = y - min[1];
    let z = z - min[2];

    // println!("local x,y,z: {},{},{}", x, y, z);

    // Dimensions of real and grid space
    let dimensions = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let grid_dimensions = self.dimensions;

    // Coordinates in grid space
    let x_grid = x * (grid_dimensions[0] - 1) as f32 / dimensions[0];
    let y_grid = y * (grid_dimensions[1] - 1) as f32 / dimensions[1];
    let z_grid = z * (grid_dimensions[2] - 1) as f32 / dimensions[2];

    // println!("grid x,y,z: {},{},{}", x_grid, y_grid, z_grid);

    // These are the cooridates of the sample
    let i = num::clamp(x_grid as usize, 0, grid_dimensions[0] - 1);
    let j = num::clamp(y_grid as usize, 0, grid_dimensions[1] - 1);
    let k = num::clamp(z_grid as usize, 0, grid_dimensions[2] - 1);

    // println!("i,j,k: {},{},{}", i, j, k);

    match self.value_inter {
      InterpolateMethod::NearestNeighbor => self.get_sample(i, j, k),
      InterpolateMethod::Trilinear => {
        // If close to the vertex just use that
        if (x_grid - i as f32).abs() <= EPSILON
          && (y_grid - j as f32).abs() <= EPSILON
          && (z_grid - k as f32).abs() <= EPSILON
        {
          self.get_sample(i, j, k)
        } else {
          // of a grid space
          match (i, j, k) {
            (i, j, k)
              if i == (grid_dimensions[0] - 1)
                && j == (grid_dimensions[1] - 1)
                && k == (grid_dimensions[2] - 1) =>
            {
              // xyz maxima case (this should be caught by the above check anyways)
              let f000 = self.get_sample(i, j, k);
              f000
            }
            (i, j, k) if i == (grid_dimensions[0] - 1) && j == (grid_dimensions[1] - 1) => {
              // xy maxima case
              let f000 = self.get_sample(i, j, k);
              let f001 = self.get_sample(i, j, k + 1);
              let z_sample = z_grid - k as f32;
              f000 * (1. - z_sample) + f001 * z_sample
            }
            (i, j, k) if i == (grid_dimensions[0] - 1) && k == (grid_dimensions[2] - 1) => {
              // xz maxima case
              let f000 = self.get_sample(i, j, k);
              let f010 = self.get_sample(i, j + 1, k);
              let y_sample = y_grid - j as f32;
              f000 * (1. - y_sample) + f010 * (y_sample)
            }
            (i, j, k) if j == (grid_dimensions[1] - 1) && k == (grid_dimensions[2] - 1) => {
              // yz maxima case
              let f000 = self.get_sample(i, j, k);
              let f100 = self.get_sample(i + 1, j, k);
              let x_sample = x_grid - i as f32;
              f000 * (1. - x_sample) + f100 * (x_sample)
            }
            (i, j, k) if i == (grid_dimensions[0] - 1) => {
              // x maxima case
              let f000 = self.get_sample(i, j, k);
              let f001 = self.get_sample(i, j, k + 1);
              let f010 = self.get_sample(i, j + 1, k);
              let f011 = self.get_sample(i, j + 1, k + 1);
              let y_sample = y_grid - j as f32;
              let z_sample = z_grid - k as f32;
              f000 * (1. - y_sample) * (1. - z_sample)
                + f001 * (1. - y_sample) * z_sample
                + f010 * (y_sample) * (1. - z_sample)
                + f011 * (y_sample) * (z_sample)
            }
            (i, j, k) if j == (grid_dimensions[1] - 1) => {
              // y maxima case
              let f000 = self.get_sample(i, j, k);
              let f001 = self.get_sample(i, j, k + 1);
              let f100 = self.get_sample(i + 1, j, k);
              let f101 = self.get_sample(i + 1, j, k + 1);
              let x_sample = x_grid - i as f32;
              let z_sample = z_grid - k as f32;
              f000 * (1. - x_sample) * (1. - z_sample)
                + f001 * (1. - x_sample) * z_sample
                + f100 * (x_sample) * (1. - z_sample)
                + f101 * (x_sample) * (z_sample)
            }
            (i, j, k) if k == (grid_dimensions[2] - 1) => {
              // z maxima case
              let f000 = self.get_sample(i, j, k);
              let f010 = self.get_sample(i, j + 1, k);
              let f100 = self.get_sample(i + 1, j, k);
              let f110 = self.get_sample(i + 1, j + 1, k);
              let x_sample = x_grid - i as f32;
              let y_sample = y_grid - j as f32;
              f000 * (1. - x_sample) * (1. - y_sample)
                + f010 * (1. - x_sample) * (y_sample)
                + f100 * (x_sample) * (1. - y_sample)
                + f110 * (x_sample) * (y_sample)
            }
            (i, j, k) => {
              // normal case 3d case
              let f000 = self.get_sample(i, j, k);
              let f001 = self.get_sample(i, j, k + 1);
              let f010 = self.get_sample(i, j + 1, k);
              let f100 = self.get_sample(i + 1, j, k);
              let f011 = self.get_sample(i, j + 1, k + 1);
              let f101 = self.get_sample(i + 1, j, k + 1);
              let f110 = self.get_sample(i + 1, j + 1, k);
              let f111 = self.get_sample(i + 1, j + 1, k + 1);
              let x_sample = x_grid - i as f32;
              let y_sample = y_grid - j as f32;
              let z_sample = z_grid - k as f32;
              f000 * (1. - x_sample) * (1. - y_sample) * (1. - z_sample)
                + f001 * (1. - x_sample) * (1. - y_sample) * z_sample
                + f010 * (1. - x_sample) * (y_sample) * (1. - z_sample)
                + f100 * (x_sample) * (1. - y_sample) * (1. - z_sample)
                + f011 * (1. - x_sample) * (y_sample) * (z_sample)
                + f101 * (x_sample) * (1. - y_sample) * (z_sample)
                + f110 * (x_sample) * (y_sample) * (1. - z_sample)
                + f111 * (x_sample) * (y_sample) * (z_sample)
            }
          }
        }
      }
    }
  }

  pub fn get_gradient(&self, x: f32, y: f32, z: f32) -> [f32; 3] {
    let (min, max) = self.bounds;
    assert!((x - min[0]) >= -EPSILON);
    assert!((max[0] - x) >= -EPSILON);
    assert!((y - min[1]) >= -EPSILON);
    assert!((max[1] - y) >= -EPSILON);
    assert!((z - min[2]) >= -EPSILON);
    assert!((max[2] - z) >= -EPSILON);

    // In local grid space
    let x = x - min[0];
    let y = y - min[1];
    let z = z - min[2];

    // Dimensions of real and grid space
    let dimensions = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let grid_dimensions = self.dimensions;

    // Coordinates in grid space
    let x_grid = x * (grid_dimensions[0] - 1) as f32 / dimensions[0];
    let y_grid = y * (grid_dimensions[1] - 1) as f32 / dimensions[1];
    let z_grid = z * (grid_dimensions[2] - 1) as f32 / dimensions[2];

    // These are the cooridates of the sample
    let i = num::clamp(x_grid.round() as usize, 0, grid_dimensions[0] - 1);
    let j = num::clamp(y_grid.round() as usize, 0, grid_dimensions[1] - 1);
    let k = num::clamp(z_grid.round() as usize, 0, grid_dimensions[2] - 1);

    match self.grad_inter {
      InterpolateMethod::NearestNeighbor => self.get_sample_grad(i, j, k),
      InterpolateMethod::Trilinear => {
        // If close to the vertex just use that
        if (x_grid - i as f32).abs() <= EPSILON
          && (y_grid - j as f32).abs() <= EPSILON
          && (z_grid - k as f32).abs() <= EPSILON
        {
          self.get_sample_grad(i, j, k)
        } else {
          // of a grid space
          match (i, j, k) {
            (i, j, k)
              if i == (grid_dimensions[0] - 1)
                && j == (grid_dimensions[1] - 1)
                && k == (grid_dimensions[2] - 1) =>
            {
              // xyz maxima case (this should be caught by the above check anyways)
              let f000 = self.get_sample_grad(i, j, k);
              [f000[0], f000[1], f000[2]]
            }
            (i, j, k) if i == (grid_dimensions[0] - 1) && j == (grid_dimensions[1] - 1) => {
              // xy maxima case
              let f000 = self.get_sample_grad(i, j, k);
              let f001 = self.get_sample_grad(i, j, k + 1);
              let z_sample = z_grid - k as f32;
              [
                f000[0] * (1. - z_sample) + f001[0] * z_sample,
                f000[1] * (1. - z_sample) + f001[1] * z_sample,
                f000[2] * (1. - z_sample) + f001[2] * z_sample,
              ]
            }
            (i, j, k) if i == (grid_dimensions[0] - 1) && k == (grid_dimensions[2] - 1) => {
              // xz maxima case
              let f000 = self.get_sample_grad(i, j, k);
              let f010 = self.get_sample_grad(i, j + 1, k);
              let y_sample = y_grid - j as f32;
              [
                f000[0] * (1. - y_sample) + f010[0] * (y_sample),
                f000[1] * (1. - y_sample) + f010[1] * (y_sample),
                f000[2] * (1. - y_sample) + f010[2] * (y_sample),
              ]
            }
            (i, j, k) if j == (grid_dimensions[1] - 1) && k == (grid_dimensions[2] - 1) => {
              // yz maxima case
              let f000 = self.get_sample_grad(i, j, k);
              let f100 = self.get_sample_grad(i + 1, j, k);
              let x_sample = x_grid - i as f32;
              [
                f000[0] * (1. - x_sample) + f100[0] * (x_sample),
                f000[1] * (1. - x_sample) + f100[1] * (x_sample),
                f000[2] * (1. - x_sample) + f100[2] * (x_sample),
              ]
            }

            (i, j, k) if i == (grid_dimensions[0] - 1) => {
              // x maxima case
              let f000 = self.get_sample_grad(i, j, k);
              let f001 = self.get_sample_grad(i, j, k + 1);
              let f010 = self.get_sample_grad(i, j + 1, k);
              let f011 = self.get_sample_grad(i, j + 1, k + 1);
              let y_sample = y_grid - j as f32;
              let z_sample = z_grid - k as f32;
              [
                f000[0] * (1. - y_sample) * (1. - z_sample)
                  + f001[0] * (1. - y_sample) * z_sample
                  + f010[0] * (y_sample) * (1. - z_sample)
                  + f011[0] * (y_sample) * (z_sample),
                f000[1] * (1. - y_sample) * (1. - z_sample)
                  + f001[1] * (1. - y_sample) * z_sample
                  + f010[1] * (y_sample) * (1. - z_sample)
                  + f011[1] * (y_sample) * (z_sample),
                f000[2] * (1. - y_sample) * (1. - z_sample)
                  + f001[2] * (1. - y_sample) * z_sample
                  + f010[2] * (y_sample) * (1. - z_sample)
                  + f011[2] * (y_sample) * (z_sample),
              ]
            }
            (i, j, k) if j == (grid_dimensions[1] - 1) => {
              // y maxima case
              let f000 = self.get_sample_grad(i, j, k);
              let f001 = self.get_sample_grad(i, j, k + 1);
              let f100 = self.get_sample_grad(i + 1, j, k);
              let f101 = self.get_sample_grad(i + 1, j, k + 1);
              let x_sample = x_grid - i as f32;
              let z_sample = z_grid - k as f32;
              [
                f000[0] * (1. - x_sample) * (1. - z_sample)
                  + f001[0] * (1. - x_sample) * z_sample
                  + f100[0] * (x_sample) * (1. - z_sample)
                  + f101[0] * (x_sample) * (z_sample),
                f000[1] * (1. - x_sample) * (1. - z_sample)
                  + f001[1] * (1. - x_sample) * z_sample
                  + f100[1] * (x_sample) * (1. - z_sample)
                  + f101[1] * (x_sample) * (z_sample),
                f000[2] * (1. - x_sample) * (1. - z_sample)
                  + f001[2] * (1. - x_sample) * z_sample
                  + f100[2] * (x_sample) * (1. - z_sample)
                  + f101[2] * (x_sample) * (z_sample),
              ]
            }
            (i, j, k) if k == (grid_dimensions[2] - 1) => {
              // z maxima case
              let f000 = self.get_sample_grad(i, j, k);
              let f010 = self.get_sample_grad(i, j + 1, k);
              let f100 = self.get_sample_grad(i + 1, j, k);
              let f110 = self.get_sample_grad(i + 1, j + 1, k);
              let x_sample = x_grid - i as f32;
              let y_sample = y_grid - j as f32;
              [
                f000[0] * (1. - x_sample) * (1. - y_sample)
                  + f010[0] * (1. - x_sample) * (y_sample)
                  + f100[0] * (x_sample) * (1. - y_sample)
                  + f110[0] * (x_sample) * (y_sample),
                f000[1] * (1. - x_sample) * (1. - y_sample)
                  + f010[1] * (1. - x_sample) * (y_sample)
                  + f100[1] * (x_sample) * (1. - y_sample)
                  + f110[1] * (x_sample) * (y_sample),
                f000[2] * (1. - x_sample) * (1. - y_sample)
                  + f010[2] * (1. - x_sample) * (y_sample)
                  + f100[2] * (x_sample) * (1. - y_sample)
                  + f110[2] * (x_sample) * (y_sample),
              ]
            }
            (i, j, k) => {
              // normal case 3d case
              let f000 = self.get_sample_grad(i, j, k);
              let f001 = self.get_sample_grad(i, j, k + 1);
              let f010 = self.get_sample_grad(i, j + 1, k);
              let f100 = self.get_sample_grad(i + 1, j, k);
              let f011 = self.get_sample_grad(i, j + 1, k + 1);
              let f101 = self.get_sample_grad(i + 1, j, k + 1);
              let f110 = self.get_sample_grad(i + 1, j + 1, k);
              let f111 = self.get_sample_grad(i + 1, j + 1, k + 1);
              let x_sample = x_grid - i as f32;
              let y_sample = y_grid - j as f32;
              let z_sample = z_grid - k as f32;
              [
                f000[0] * (1. - x_sample) * (1. - y_sample) * (1. - z_sample)
                  + f001[0] * (1. - x_sample) * (1. - y_sample) * z_sample
                  + f010[0] * (1. - x_sample) * (y_sample) * (1. - z_sample)
                  + f100[0] * (x_sample) * (1. - y_sample) * (1. - z_sample)
                  + f011[0] * (1. - x_sample) * (y_sample) * (z_sample)
                  + f101[0] * (x_sample) * (1. - y_sample) * (z_sample)
                  + f110[0] * (x_sample) * (y_sample) * (1. - z_sample)
                  + f111[0] * (x_sample) * (y_sample) * (z_sample),
                f000[1] * (1. - x_sample) * (1. - y_sample) * (1. - z_sample)
                  + f001[1] * (1. - x_sample) * (1. - y_sample) * z_sample
                  + f010[1] * (1. - x_sample) * (y_sample) * (1. - z_sample)
                  + f100[1] * (x_sample) * (1. - y_sample) * (1. - z_sample)
                  + f011[1] * (1. - x_sample) * (y_sample) * (z_sample)
                  + f101[1] * (x_sample) * (1. - y_sample) * (z_sample)
                  + f110[1] * (x_sample) * (y_sample) * (1. - z_sample)
                  + f111[1] * (x_sample) * (y_sample) * (z_sample),
                f000[2] * (1. - x_sample) * (1. - y_sample) * (1. - z_sample)
                  + f001[2] * (1. - x_sample) * (1. - y_sample) * z_sample
                  + f010[2] * (1. - x_sample) * (y_sample) * (1. - z_sample)
                  + f100[2] * (x_sample) * (1. - y_sample) * (1. - z_sample)
                  + f011[2] * (1. - x_sample) * (y_sample) * (z_sample)
                  + f101[2] * (x_sample) * (1. - y_sample) * (z_sample)
                  + f110[2] * (x_sample) * (y_sample) * (1. - z_sample)
                  + f111[2] * (x_sample) * (y_sample) * (z_sample),
              ]
            }
          }
        }
      }
    }
  }

  pub fn get_value_gradient(&self, x: f32, y: f32, z: f32) -> (f32, [f32; 3]) {
    (self.get_value(x, y, z), self.get_gradient(x, y, z))
  }

  /// Fill a chunk based on bounds in this density
  pub fn fill_chunk<Chunk, Value>(
    &self,
    chunk: &mut Chunk,
    fill: Value,
    bounds: &([f32; 3], [f32; 3]),
    isovalue: f32,
  ) where
    Chunk: ChunkifyMut<Value> + Sizable,
    Value: Clone,
  {
    let (min, max) = bounds;
    let size = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let dimensions = [
      chunk.width() as usize,
      chunk.height() as usize,
      chunk.depth() as usize,
    ];
    let delta = [
      size[0] / (dimensions[0] - 1) as f32,
      size[1] / (dimensions[1] - 1) as f32,
      size[2] / (dimensions[2] - 1) as f32,
    ];
    for i in 0..dimensions[0] {
      let x = min[0] + i as f32 * delta[0];
      for j in 0..dimensions[1] {
        let y = min[1] + j as f32 * delta[1];
        for k in 0..dimensions[2] {
          let z = min[2] + k as f32 * delta[2];
          let value = self.get_value(x, y, z);
          if value > isovalue {
            // println!("Filling");
            chunk.set(i, j, k, fill.clone());
          }
        }
      }
    }
  }
}
