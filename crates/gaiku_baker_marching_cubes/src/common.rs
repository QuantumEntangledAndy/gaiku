use gaiku_common::mint::Vector3;
use glam::Vec3;

pub(crate) const EPSILON: f32 = 1e-4;

#[derive(Debug)]
pub(crate) struct GridCell {
  pub value: [f32; 8],
  pub point: [Vector3<f32>; 8],
}

use super::tables::{EDGE_TABLE, TRIANGLE_TABLE};

impl GridCell {
  fn lerp(&self, index1: usize, index2: usize, isolevel: f32) -> [f32; 3] {
    let mut index1 = index1;
    let mut index2 = index2;

    if self.value[index2] < self.value[index1] {
      std::mem::swap(&mut index1, &mut index2);
    }

    let point1: Vec3 = self.point[index1].into();
    let point2: Vec3 = self.point[index2].into();

    if (point1 - point2).length() > EPSILON {
      let value1 = self.value[index1] as f32;
      let value2 = self.value[index2] as f32;

      if (isolevel - value1).abs() <= EPSILON {
        point1.into()
      } else if (isolevel - value2).abs() <= EPSILON {
        point2.into()
      } else if isolevel < value1 {
        unreachable!();
      } else if isolevel > value2 {
        unreachable!();
      } else {
        let weight = (isolevel - value1) / (value2 - value1);
        (point1 * weight + point2 * (1. - weight)).into()
      }
    } else {
      self.point[index1].into()
    }
  }

  /// Return the index of the corner nearest this point
  pub(crate) fn nearest_corner(&self, point: &[f32; 3]) -> usize {
    let dist: Vec<_> = self
      .point
      .iter()
      .map(|p| vec_sum_sq(&vec_sub(point, &[p.x, p.y, p.z])))
      .collect();

    let mut i = 0;
    for (j, &value) in dist.iter().enumerate() {
      if value < dist[i] {
        i = j;
      }
    }

    i
  }

  // Now for the UVs
  //
  // Plan is, put vertex coordinates relative to nearest cube corner + [.5,.5,.5]
  //
  // Find which axis direction the normal is pointing
  //
  // Map uvs on that axis direction as if the face was perfectly aligned with that axis
  //
  // This is simlar to tri-linear shader expect that it is without the blending
  // and only contains one map instead of three
  //
  // This should be fine for anything that is mostly axis aligned
  //
  // Caveat Emptor
  pub(crate) fn compute_uvs(&self, vertex: &[[f32; 3]; 3]) -> [[f32; 2]; 3] {
    let normal = compute_normal(&vertex);

    let face_mid = vec_ave(vec![&vertex[0], &vertex[1], &vertex[2]]);
    // Move the face center backwards by small amount relative to normal
    let epsilon = vec_mult(&normal, -EPSILON);
    let face_mid_eps = vec_add(&face_mid, &epsilon);
    // Get the corner of the grid nearest to the face
    let corner_idx = self.nearest_corner(&face_mid_eps);
    // Get the verts coordinates relative to the grid corner point
    let cube_center = [
      self.point[corner_idx].x,
      self.point[corner_idx].y,
      self.point[corner_idx].z,
    ];
    let vertex_mapped: Vec<_> = vertex
      .iter()
      .map(|v| {
        let mut vertex_relative = vec_sub(v, &cube_center);
        // Scale so that gridcell is of size 1
        vertex_relative[0] /= self.point[6].x - self.point[0].x;
        vertex_relative[1] /= self.point[6].y - self.point[0].y;
        vertex_relative[2] /= self.point[6].z - self.point[0].z;
        // Put it into the range 0..1 instead of -0.5..0.5
        vertex_relative[0] += 0.5;
        vertex_relative[1] += 0.5;
        vertex_relative[2] += 0.5;
        vertex_relative
      })
      .collect();

    // Is the normal pointing along x, y, or z
    // We use that to decide how to map the uvs
    // dot product gives the cosine of the angle
    // between. We take abs and find the maximum
    let cos = [
      vec_dot(&normal, &[1., 0., 0.]),
      vec_dot(&normal, &[0., 1., 0.]),
      vec_dot(&normal, &[0., 0., 1.]),
    ];

    // Nearest axis alignment is this one!
    let mut i = 0;
    for (j, &value) in cos.iter().enumerate() {
      if value > cos[i].abs() {
        i = j;
      }
    }
    let max_cos = cos[i];

    let permutation = [
      [1, 2], // If aligned with x then uv is on y,z
      [0, 2], // If aligned with y then uv is on x,z
      [0, 1], // If aligned with z then uv is on x,y
    ];

    // When cos < 0 we flip it (cos its facing against the axis)
    let (j, k) = if max_cos >= 0. { (0, 1) } else { (1, 0) };

    // Result time
    [
      [
        vertex_mapped[0][permutation[i][j]].clamp(0., 1.),
        vertex_mapped[0][permutation[i][k]].clamp(0., 1.),
      ],
      [
        vertex_mapped[1][permutation[i][j]].clamp(0., 1.),
        vertex_mapped[1][permutation[i][k]].clamp(0., 1.),
      ],
      [
        vertex_mapped[2][permutation[i][j]].clamp(0., 1.),
        vertex_mapped[2][permutation[i][k]].clamp(0., 1.),
      ],
    ]
  }

  pub(crate) fn polygonize(&self, isolevel: f32) -> Vec<[[f32; 3]; 3]> {
    let mut cube_index = 0;
    let mut vertex_list = [[0.0, 0.0, 0.0]; 12];
    let mut triangles = vec![];

    if self.value[0] < isolevel {
      cube_index |= 1;
    }
    if self.value[1] < isolevel {
      cube_index |= 2;
    }
    if self.value[2] < isolevel {
      cube_index |= 4;
    }
    if self.value[3] < isolevel {
      cube_index |= 8;
    }
    if self.value[4] < isolevel {
      cube_index |= 16;
    }
    if self.value[5] < isolevel {
      cube_index |= 32;
    }
    if self.value[6] < isolevel {
      cube_index |= 64;
    }
    if self.value[7] < isolevel {
      cube_index |= 128;
    }

    if EDGE_TABLE[cube_index] == 0 {
      return vec![];
    }

    if (EDGE_TABLE[cube_index] & 1) != 0 {
      vertex_list[0] = self.lerp(0, 1, isolevel);
    }

    if (EDGE_TABLE[cube_index] & 2) != 0 {
      vertex_list[1] = self.lerp(1, 2, isolevel);
    }

    if (EDGE_TABLE[cube_index] & 4) != 0 {
      vertex_list[2] = self.lerp(2, 3, isolevel);
    }

    if (EDGE_TABLE[cube_index] & 8) != 0 {
      vertex_list[3] = self.lerp(3, 0, isolevel);
    }

    if (EDGE_TABLE[cube_index] & 16) != 0 {
      vertex_list[4] = self.lerp(4, 5, isolevel);
    }

    if (EDGE_TABLE[cube_index] & 32) != 0 {
      vertex_list[5] = self.lerp(5, 6, isolevel);
    }

    if (EDGE_TABLE[cube_index] & 64) != 0 {
      vertex_list[6] = self.lerp(6, 7, isolevel);
    }

    if (EDGE_TABLE[cube_index] & 128) != 0 {
      vertex_list[7] = self.lerp(7, 4, isolevel);
    }

    if (EDGE_TABLE[cube_index] & 256) != 0 {
      vertex_list[8] = self.lerp(0, 4, isolevel);
    }

    if (EDGE_TABLE[cube_index] & 512) != 0 {
      vertex_list[9] = self.lerp(1, 5, isolevel);
    }

    if (EDGE_TABLE[cube_index] & 1024) != 0 {
      vertex_list[10] = self.lerp(2, 6, isolevel);
    }

    if (EDGE_TABLE[cube_index] & 2048) != 0 {
      vertex_list[11] = self.lerp(3, 7, isolevel);
    }

    let mut i = 0;

    loop {
      if TRIANGLE_TABLE[cube_index][i] == -1 {
        break;
      }

      triangles.push([
        vertex_list[TRIANGLE_TABLE[cube_index][i] as usize],
        vertex_list[TRIANGLE_TABLE[cube_index][i + 1] as usize],
        vertex_list[TRIANGLE_TABLE[cube_index][i + 2] as usize],
      ]);

      i += 3;
    }
    triangles
  }
}

pub(crate) fn vec_sum_sq(a: &[f32; 3]) -> f32 {
  a[0].powi(2) + a[1].powi(2) + a[2].powi(2)
}

pub(crate) fn vec_length(a: &[f32; 3]) -> f32 {
  vec_sum_sq(a).sqrt()
}

pub(crate) fn vec_sub(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
  [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(crate) fn vec_add(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
  [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub(crate) fn vec_ave(vecs: Vec<&[f32; 3]>) -> [f32; 3] {
  vec_mult(
    &vecs.iter().fold([0., 0., 0.], |acc, v| vec_add(&acc, v)),
    1. / vecs.len() as f32,
  )
}

pub(crate) fn vec_mult(vec: &[f32; 3], factor: f32) -> [f32; 3] {
  [vec[0] * factor, vec[1] * factor, vec[2] * factor]
}

pub(crate) fn vec_cross(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
  [
    a[1] * b[2] - a[2] * b[1],
    a[2] * b[0] - a[0] * b[2],
    a[0] * b[1] - a[1] * b[0],
  ]
}

pub(crate) fn vec_dot(a: &[f32; 3], b: &[f32; 3]) -> f32 {
  a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(crate) fn vec_normalised(a: &[f32; 3]) -> [f32; 3] {
  vec_mult(a, 1. / vec_length(a))
}

pub(crate) fn compute_normal(triangle: &[[f32; 3]; 3]) -> [f32; 3] {
  vec_cross(
    &vec_normalised(&vec_sub(&triangle[1], &triangle[0])),
    &vec_normalised(&vec_sub(&triangle[2], &triangle[0])),
  )
}
