// This is the source used to generate the tables

use lazy_static::lazy_static;
use std::collections::HashMap;

fn calc_cube_index(all: &[bool; 8]) -> usize {
  let result = all.iter().enumerate().fold(0, |acc, (idx, val)| {
    if *val == true {
      acc | 2_usize.pow(idx as u32)
    } else {
      acc
    }
  });
  result
}

// Order of verts is important
// verts[0], verts[1], verts[2] must form a triangle where clockwise
// is outwards facing if a is true and b is false
fn add_to_tables(
  a: bool,
  b: bool,
  cube_index: usize,
  verts: [i8; 4],
  edge_table: &mut [u32; 256],
  triangle_table: &mut Vec<Vec<i8>>,
) {
  if a != b {
    for v in verts.iter() {
      edge_table[cube_index] |= 2_u32.pow(*v as u32);
    }
    if a {
      // A face out
      triangle_table[cube_index].append(&mut vec![verts[0], verts[1], verts[2]]);
      triangle_table[cube_index].append(&mut vec![verts[2], verts[1], verts[3]]);
    } else {
      // B face outwards
      triangle_table[cube_index].append(&mut vec![verts[0], verts[2], verts[1]]);
      triangle_table[cube_index].append(&mut vec![verts[1], verts[2], verts[3]]);
    }
  }
}

lazy_static! {
  static ref CORNER_MAP: HashMap<i8, [i8; 3]> = {
    let mut m = HashMap::new();
    m.insert(0, [0, 0, 0]);
    m.insert(1, [2, 0, 0]);
    m.insert(2, [2, 2, 0]);
    m.insert(3, [0, 2, 0]);
    m.insert(4, [0, 0, 2]);
    m.insert(5, [2, 0, 2]);
    m.insert(6, [2, 2, 2]);
    m.insert(7, [0, 2, 2]);
    m
  };
  static ref VERT_MAP: HashMap<[i8; 3], i8> = {
    let mut m = HashMap::new();
    m.insert([1, 0, 0], 0);
    m.insert([2, 1, 0], 1);
    m.insert([1, 2, 0], 2);
    m.insert([0, 1, 0], 3);
    m.insert([1, 0, 2], 4);
    m.insert([2, 1, 2], 5);
    m.insert([1, 2, 2], 6);
    m.insert([0, 1, 2], 7);
    m.insert([0, 0, 1], 8);
    m.insert([2, 0, 1], 9);
    m.insert([2, 2, 1], 10);
    m.insert([0, 2, 1], 11);
    m.insert([1, 0, 1], 12);
    m.insert([2, 1, 1], 13);
    m.insert([1, 2, 1], 14);
    m.insert([0, 1, 1], 15);
    m.insert([1, 1, 0], 16);
    m.insert([1, 1, 2], 17);
    m.insert([1, 1, 1], 18);
    m
  };
}

fn vec_eq(a: [i8; 3], b: [i8; 3]) -> bool {
  a[0] == b[0] && a[1] == b[1] && a[2] == b[2]
}

fn vec_add(a_coord: [i8; 3], b_coord: [i8; 3]) -> [i8; 3] {
  [
    a_coord[0] + b_coord[0],
    a_coord[1] + b_coord[1],
    a_coord[2] + b_coord[2],
  ]
}

fn vec_sub(a_coord: [i8; 3], b_coord: [i8; 3]) -> [i8; 3] {
  [
    a_coord[0] - b_coord[0],
    a_coord[1] - b_coord[1],
    a_coord[2] - b_coord[2],
  ]
}

fn vec_cross(a: [i8; 3], b: [i8; 3]) -> [i8; 3] {
  [
    a[1] * b[2] - a[2] * b[1],
    a[2] * b[0] - a[0] * b[2],
    a[0] * b[1] - a[1] * b[0],
  ]
}

fn get_verts(a_coord: [i8; 3], b_coord: [i8; 3]) -> Option<[i8; 4]> {
  // All a_coord or b_coors are corner points so their values all always [0/2, 0/2, 0/2] never 1
  let mid_coord = [
    (b_coord[0] + a_coord[0]) / 2,
    (b_coord[1] + a_coord[1]) / 2,
    (b_coord[2] + a_coord[2]) / 2,
  ];
  // Filter our diagonals and self
  if mid_coord
    .iter()
    .fold(0, |acc, &v| if v == 1 { acc + 1 } else { acc })
    != 1
  {
    return None;
  }
  // Since we only care for the neighbours in the non diagonal axis we can find the
  // points by first finding the axis which has a value of 1
  // e.g. ([0,0,0] + [0,2,0])/2 = [0, 1, 0]
  //       This means y is the axis aligned with b->a
  //       then we want to get the other two face points
  let axis = mid_coord.iter().position(|&i| i == 1).unwrap();
  // This permutation will give the axis direction of the other two face points
  let permutation = [[1, 2], [0, 2], [0, 1]];
  let c = {
    // other face point
    let i = permutation[axis][0];
    let mut new_c = mid_coord.clone();
    if new_c[i] + 1 > 2 {
      new_c[i] -= 1
    } else {
      new_c[i] += 1
    }
    new_c
  }; // last face point
  let d = {
    let i = permutation[axis][1];
    let mut new_d = mid_coord.clone();
    if new_d[i] + 1 > 2 {
      new_d[i] -= 1
    } else {
      new_d[i] += 1
    }
    new_d
  };
  // A face is made from mid, c, d, and center (center is at [1,1,1])
  // But which way should point outwards?
  // We use the cross product to find out
  let c_delta = vec_sub(c, mid_coord);
  let d_delta = vec_sub(d, mid_coord);
  let cross = vec_cross(c_delta, d_delta);
  let mid_cross = vec_add(mid_coord, cross);
  let result;
  if vec_eq(mid_cross, a_coord) {
    // Cross of c,d + mid == a
    result = [mid_coord, d, c, [1, 1, 1]];
  } else {
    result = [mid_coord, c, d, [1, 1, 1]];
  }
  Some([
    *VERT_MAP.get(&result[0]).unwrap(),
    *VERT_MAP.get(&result[1]).unwrap(),
    *VERT_MAP.get(&result[2]).unwrap(),
    *VERT_MAP.get(&result[3]).unwrap(),
  ])
}

fn main() {
  let mut edge_table: [u32; 256] = [0; 256];
  let mut triangle_table: Vec<Vec<i8>> = (0..256).map(|_| vec![]).collect();

  for a in [false, true] {
    for b in [false, true] {
      for c in [false, true] {
        for d in [false, true] {
          for e in [false, true] {
            for f in [false, true] {
              for g in [false, true] {
                for h in [false, true] {
                  let values = [a, b, c, d, e, f, g, h];
                  let cube_index = calc_cube_index(&values);

                  for i in 0..8 {
                    for j in (i + 1)..8 {
                      let m = values[i];
                      let n = values[j];
                      let verts = get_verts(CORNER_MAP[&(i as i8)], CORNER_MAP[&(j as i8)]);
                      if let Some(verts) = verts {
                        add_to_tables(
                          m,
                          n,
                          cube_index,
                          verts,
                          &mut edge_table,
                          &mut triangle_table,
                        );
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
  let longest = triangle_table.iter().map(|c| c.len()).max().unwrap();
  assert!(longest == 12 * 3 * 2);
  triangle_table
    .iter_mut()
    .for_each(|c| c.append(&mut vec![-1; longest - c.len()]));
  println!("pub const EDGE_TABLE: [u32; 256] = {:?};", edge_table);
  println!(
    "pub const TRIANGLE_TABLE: [[i8; {}]; 256] = {:?};",
    longest, triangle_table
  );
}
