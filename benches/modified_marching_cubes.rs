use criterion::{criterion_group, criterion_main, Criterion, SamplingMode};
use gaiku::{
  common::{
    chunk::Chunk,
    mesh::Mesh,
    prelude::*,
    texture::{Texture2d, TextureAtlas2d},
    Result,
  },
  GoxReader, ModMarchingCubesBaker,
};

fn get_chunks(name: &str) -> Result<(Vec<Chunk>, Option<TextureAtlas2d<Texture2d>>)> {
  let file = format!(
    "{}/examples/assets/{}.gox",
    env!("CARGO_MANIFEST_DIR"),
    name
  );

  GoxReader::read(&file)
}

fn marching_cubes_benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("Modified Marching cubes");
  let (chunks, texture) = get_chunks("terrain").unwrap();
  let options = BakerOptions {
    texture,
    ..Default::default()
  };
  group.sampling_mode(SamplingMode::Flat);
  group.sample_size(10);

  group.bench_function("Terrain", |b| {
    b.iter(|| {
      let mut meshes: Vec<(Mesh, [f32; 3])> = vec![];

      for chunk in chunks.iter() {
        let mesh = ModMarchingCubesBaker::bake(chunk, &options).unwrap();
        if let Some(mesh) = mesh {
          meshes.push((mesh, chunk.position()));
        }
      }
    })
  });

  let (chunks, texture) = get_chunks("planet").unwrap();
  let options = BakerOptions {
    texture,
    ..Default::default()
  };

  group.bench_function("Planet", |b| {
    b.iter(|| {
      let mut meshes: Vec<(Mesh, [f32; 3])> = vec![];

      for chunk in chunks.iter() {
        let mesh = ModMarchingCubesBaker::bake(chunk, &options).unwrap();
        if let Some(mesh) = mesh {
          meshes.push((mesh, chunk.position()));
        }
      }
    })
  });

  let (chunks, texture) = get_chunks("small_tree").unwrap();
  let options = BakerOptions {
    texture,
    ..Default::default()
  };

  group.sample_size(100);
  group.bench_function("Small tree", |b| {
    b.iter(|| {
      let mut meshes: Vec<(Mesh, [f32; 3])> = vec![];

      for chunk in chunks.iter() {
        let mesh = ModMarchingCubesBaker::bake(chunk, &options).unwrap();
        if let Some(mesh) = mesh {
          meshes.push((mesh, chunk.position()));
        }
      }
    })
  });

  group.finish();
}

criterion_group!(benches, marching_cubes_benchmark);

criterion_main! {
    benches,
}
