use gaiku_common::{chunktree::*, density::*};

use amethyst::{
  assets::{AssetStorage, Handle, Loader},
  controls::FlyControlTag,
  core::{math::Vector4, transform::Transform, Hidden},
  ecs::prelude::*,
  prelude::*,
  renderer::{
    light::{DirectionalLight, Light},
    palette::rgb::Rgb,
    types::{MeshData, TextureData},
    visibility::BoundingSphere,
    ActiveCamera, Camera, Material, MaterialDefaults, Mesh, Texture,
  },
};

use gaiku_amethyst::prelude::*;
use gaiku_baker_voxel::VoxelBaker;

use noise::{Fbm as SourceNoise, NoiseFn, Seedable};

use crate::chunktree_example::metachunk::MetaChunk;

pub struct GameLoad {
  noise_source: SourceNoise,
  visible_entities: Vec<Entity>,
}

impl SimpleState for GameLoad {
  fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
    let world = data.world;

    self.initialise_camera(world);
    self.add_light(world);
    println!("Building noise density terrain");
    self.build_terrain(world);
    println!("Making new chunks with density (if visible)");
    self.update_visible_chunks(world);
  }

  fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
    let world = data.world;
    self.update_visible_chunks(world);
    SimpleTrans::None
  }
}

impl Default for GameLoad {
  fn default() -> Self {
    Self::new()
  }
}

impl GameLoad {
  pub fn new() -> Self {
    Self {
      noise_source: SourceNoise::new().set_seed(0),
      visible_entities: vec![],
    }
  }

  fn initialise_camera(&self, world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0., 70., -90.0);
    //transform.face_towards(Vector3::new(0., 0., 0.), Vector3::new(0., 1., 0.));

    let cam_ent = world
      .create_entity()
      .with(Camera::standard_3d(600., 400.))
      .with(transform)
      .with(FlyControlTag)
      .build();
    let act_cam: &mut ActiveCamera = world.get_mut().expect("There shoud be an active camera");
    act_cam.entity = Some(cam_ent);
  }

  fn add_light(&self, world: &mut World) {
    world
      .create_entity()
      .with(Light::from(DirectionalLight {
        color: Rgb::new(1.0, 1.0, 1.0),
        direction: [-1.0, -1.0, -1.0].into(),
        intensity: 1.0,
      }))
      .build();

    world
      .create_entity()
      .with(Light::from(DirectionalLight {
        color: Rgb::new(1.0, 0.1, 0.1),
        direction: [-1.0, 1.0, -1.0].into(),
        intensity: 1.0,
      }))
      .build();

    world
      .create_entity()
      .with(Light::from(DirectionalLight {
        color: Rgb::new(0.1, 1.0, 0.1),
        direction: [1.0, -1.0, -1.0].into(),
        intensity: 1.0,
      }))
      .build();

    world
      .create_entity()
      .with(Light::from(DirectionalLight {
        color: Rgb::new(0.1, 0.1, 1.0),
        direction: [-1.0, -1.0, 1.0].into(),
        intensity: 1.0,
      }))
      .build();
  }

  fn noise(&self, x: f32, y: f32, z: f32) -> f32 {
    const GROUND_HEIGHT: f32 = 0.0; // Could replace with 2D noise
    const HEIGHT_DROPOFF: f32 = 1.0 / 100.;

    // The fbm noise
    let coords = [x as f64, y as f64, z as f64];
    let noise = self.noise_source.get(coords);

    // Less dense as we go above ground height
    // Just using a linear dropoff but could try exp or power
    let solid_below = -(y - GROUND_HEIGHT) * HEIGHT_DROPOFF;

    noise as f32 + solid_below
  }

  fn build_terrain(&self, world: &mut World) {
    // First we build the noise as an array at lowest LOD
    let range: f32 = 2000.; // 2000 meters in all dimensions
    let resolution: f32 = 0.01; // noise sample mer meter
    let density_dimensions = [(range * 2. * resolution) as usize; 3];

    let origin = [-range, -range, -range];
    let delta = [1. / resolution, 1. / resolution, 1. / resolution];

    println!("Making noise");
    let noise_data = (0..density_dimensions[0])
      .into_iter()
      .map(|i| {
        (0..density_dimensions[1])
          .into_iter()
          .map(|j| {
            (0..density_dimensions[2])
              .into_iter()
              .map(|k| {
                self.noise(
                  origin[0] + delta[0] * (i as f32),
                  origin[1] + delta[1] * (j as f32),
                  origin[2] + delta[2] * (k as f32),
                )
              })
              .collect::<Vec<f32>>()
          })
          .collect::<Vec<Vec<f32>>>()
      })
      .collect::<Vec<Vec<Vec<f32>>>>();

    // println!("Noise source: {:?}", noise_data);

    let bounds = ([-range, -range, -range], [range, range, range]);

    println!("Making density from noise");
    let density = DensityData::from_3dsamples(noise_data.clone(), density_dimensions, bounds);

    assert!(density.get_sample(1, 10, 5) == noise_data[1][10][5]);

    let levels: usize = 8;
    println!("Building tree with {} levels", levels);
    let chunktree = ChunkTree::<MetaChunk, (u8, u8)>::new(bounds.clone(), levels);
    world.insert(chunktree);
    world.insert(density);
  }

  /// Generates chunks for visible LODs only
  /// Hide those that are not visible but have been generated
  /// Unhide those that are visible and have been generated
  fn update_visible_chunks(&mut self, world: &mut World) {
    // First hide all those that are currently shown
    {
      let mut hiddens = world.write_storage::<Hidden>();
      for ent in self.visible_entities.drain(..) {
        if hiddens.insert(ent, Hidden).is_err() {
          println!("Failed to hide old chunk");
        }
      }
    }
    // Now either create or unhide
    type SystemData<'s> = (
      Entities<'s>,
      WriteExpect<'s, ChunkTree<MetaChunk, (u8, u8)>>,
      ReadExpect<'s, Loader>,
      ReadExpect<'s, DensityData>,
      ReadExpect<'s, MaterialDefaults>,
      Read<'s, ActiveCamera>,
      Read<'s, AssetStorage<Mesh>>,
      Read<'s, AssetStorage<Texture>>,
      Read<'s, AssetStorage<Material>>,
      WriteStorage<'s, Transform>,
      WriteStorage<'s, Handle<Mesh>>,
      WriteStorage<'s, Handle<Material>>,
      WriteStorage<'s, BoundingSphere>,
      WriteStorage<'s, Hidden>,
    );
    world.exec(
      |(
        entities,
        mut chunk_tree,
        loader,
        density,
        material_default,
        act_cam,
        meshes,
        textures,
        materials,
        mut transforms,
        mut mesh_storage,
        mut material_storage,
        mut bound_storage,
        mut hidden_storage,
      ): SystemData| {
        if let Some(cam_ent) = act_cam.entity {
          // Get the cameras global position
          let global_cam_pos = {
            let cam_trans = transforms
              .get(cam_ent)
              .expect("Camera should have a transform");
            cam_trans.global_matrix() * Vector4::new(0., 0., 0., 1.0)
          };

          // Make chunks if visible
          let chunk_dimensions = [20, 20, 20];
          for visible_lod in chunk_tree
            .get_visible_lods_mut(
              &[global_cam_pos[0], global_cam_pos[1], global_cam_pos[2]],
              50.,
            )
            .into_iter()
          {
            // println!(
            //   "Making chunk at level: {}, pos: {:?}",
            //   visible_lod.get_level(),
            //   visible_lod.get_center()
            // );
            // Create
            if visible_lod.get_chunk().is_none() {
              // println!("Making chunk");
              let mut chunk = MetaChunk::with_size(
                chunk_dimensions[0],
                chunk_dimensions[1],
                chunk_dimensions[2],
              );

              let bounds = visible_lod.get_bounds();
              density.fill_chunk(&mut chunk, (1, 1), bounds, 0.);

              let origin = visible_lod.get_origin();
              let target_size = visible_lod.get_size();
              let level = visible_lod.get_level();
              let color = [10 * level as u8, 200, 10 * level as u8, 255];

              let bake_result = self.make_mesh_from_chunk(&chunk, color);

              // Make an entity we can assign to the metachunk
              let entity = if let Some((mesh_data, tex_data)) = bake_result {
                // println!("Mesh generated");
                // if the bake was successful this entity will use the baked mesh
                let (mesh, mat) = {
                  let tex = loader.load_from_data(tex_data, (), &textures);
                  let mesh = loader.load_from_data(mesh_data, (), &meshes);
                  let mat: Handle<Material> = loader.load_from_data(
                    Material {
                      albedo: tex,
                      ..material_default.0.clone()
                    },
                    (),
                    &materials,
                  );
                  (mesh, mat)
                };

                // Get transform from the tree data
                let mut transform = Transform::default();
                transform.set_translation_xyz(origin[0], origin[1], origin[2]);

                // Also set the scale so that it fits in the leafs bounds
                let current_size = [
                  chunk.width() as f32,
                  chunk.height() as f32,
                  chunk.depth() as f32,
                ];
                let scale = [
                  target_size[0] / current_size[0],
                  target_size[1] / current_size[1],
                  target_size[2] / current_size[2],
                ];
                // println!(
                //   "target_size: {:?}, current_size: {:?}, scale: {:?}",
                //   target_size, current_size, scale
                // );
                transform.set_scale(scale.into());

                // The bounding box should also be set so that amethyst dosen't clip it
                // These are in mesh coordinates before scaling
                let radius =
                  (current_size[0].powi(2) + current_size[1].powi(2) + current_size[2].powi(2))
                    .sqrt()
                    * 1.5;
                let bounding = BoundingSphere {
                  center: [
                    current_size[0] / 2.,
                    current_size[1] / 2.,
                    current_size[2] / 2.,
                  ]
                  .into(),
                  radius,
                };

                entities
                  .build_entity()
                  .with(mesh, &mut mesh_storage)
                  .with(mat, &mut material_storage)
                  .with(transform, &mut transforms)
                  .with(bounding, &mut bound_storage)
                  .build()
              } else {
                // Otherwise we just assign an empty entity
                // println!("Mesh NOT generated");
                entities.build_entity().build()
              };

              chunk.set_entity(entity);
              self.visible_entities.push(entity.clone());

              visible_lod.set_chunk(chunk);
            } else {
              if let Some(ent) = visible_lod.get_chunk().and_then(|c| c.get_entity()) {
                hidden_storage.remove(ent);
                self.visible_entities.push(ent.clone());
              }
            }
          }
        }
      },
    );
  }

  fn make_mesh_from_chunk(
    &self,
    chunk: &MetaChunk,
    color: [u8; 4],
  ) -> Option<(MeshData, TextureData)> {
    // Make a texture that just has a green tile in it
    let mut texture = TextureAtlas2d::new(4);
    texture.set_at_index(1, [color; 4 * 4].to_vec());

    // Craete the baker options to include this texture
    let options = BakerOptions {
      texture: Some(texture),
      ..Default::default()
    };

    // Bake the mesh
    let meshgox = VoxelBaker::bake::<MetaChunk, GaikuTexture2d, GaikuMesh>(chunk, &options);

    if let Ok(Some(mut mesh)) = meshgox {
      // Some bakers like marching cubes are missing UVs and normals when amethyst expects them
      // For now just make uvs with zeros
      let point_count = mesh.get_positions().len();
      if point_count != mesh.get_uvs().len() {
        mesh.set_uvs(vec![[0., 0.]; point_count]);
      }
      if point_count != mesh.get_normals().len() {
        mesh.set_normals(vec![[0., 0., 0.]; point_count]);
      }
      let tex = options.texture.unwrap().get_texture();

      // Put all data into amethyst format
      let tex_data: TextureData = tex.into();
      let mesh_data: MeshData = mesh.into();
      Some((mesh_data, tex_data))
    } else {
      // Nothing too bake probably an empty chunk
      None
    }
  }
}
