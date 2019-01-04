//!
//! The mighty triangle example.
//! This examples shows colord triangle on white background.
//! Nothing fancy. Just prove that `rendy` works.
//!

#![forbid(overflowing_literals)]
#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(intra_doc_link_resolution_failure)]
#![deny(path_statements)]
#![deny(trivial_bounds)]
#![deny(type_alias_bounds)]
#![deny(unconditional_recursion)]
#![deny(unions_with_drop_fields)]
#![deny(while_true)]
#![deny(unused)]
#![deny(bad_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![allow(unused_unsafe)]

use rendy::{
    command::{RenderPassInlineEncoder},
    factory::{Config, Factory},
    graph::{Graph, GraphBuilder, render::RenderPass, present::PresentNode, NodeBuffer, NodeImage},
    memory::MemoryUsageValue,
    mesh::{AsVertex, PosColor, Mesh as RendyMesh},
    shader::{Shader, StaticShaderInfo, ShaderKind, SourceLanguage},
    resource::buffer::Buffer,
};

use gfx_hal::queue::QueueFamilyId;

use winit::{
    EventsLoop, WindowBuilder,
};

use gaiku_3d::common::Mesh;

//#[cfg(feature = "dx12")]
type Backend = rendy::dx12::Backend;

//#[cfg(feature = "metal")]
//type Backend = rendy::metal::Backend;
//
//#[cfg(feature = "vulkan")]
//type Backend = rendy::vulkan::Backend;

lazy_static::lazy_static! {
    static ref vertex: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/shader.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    );

    static ref fragment: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/shader.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    );
}

#[derive(Debug)]
struct MeshRenderPass<B: gfx_hal::Backend> {
    vertex: Buffer<B>,
}

impl<B, T> RenderPass<B, T> for MeshRenderPass<B>
    where
        B: gfx_hal::Backend,
        T: ?Sized,
{
    fn name() -> &'static str {
        "Gaiku"
    }

    fn vertices() -> Vec<(
        Vec<gfx_hal::pso::Element<gfx_hal::format::Format>>,
        gfx_hal::pso::ElemStride,
    )> {
        vec![PosColor::VERTEX.gfx_vertex_input_desc()]
    }

    fn load_shader_sets<'a>(
        storage: &'a mut Vec<B::ShaderModule>,
        factory: &mut Factory<B>,
        _aux: &mut T,
    ) -> Vec<gfx_hal::pso::GraphicsShaderSet<'a, B>> {
        storage.clear();

        log::trace!("Load shader module '{:#?}'", *vertex);
        storage.push(vertex.module(factory).unwrap());

        log::trace!("Load shader module '{:#?}'", *fragment);
        storage.push(fragment.module(factory).unwrap());

        vec![gfx_hal::pso::GraphicsShaderSet {
            vertex: gfx_hal::pso::EntryPoint {
                entry: "main",
                module: &storage[0],
                specialization: gfx_hal::pso::Specialization::default(),
            },
            fragment: Some(gfx_hal::pso::EntryPoint {
                entry: "main",
                module: &storage[1],
                specialization: gfx_hal::pso::Specialization::default(),
            }),
            hull: None,
            domain: None,
            geometry: None,
        }]
    }

    fn build<'a>(
        _factory: &mut Factory<B>,
        _aux: &mut T,
        buffers: &mut [NodeBuffer<'a, B>],
        images: &mut [NodeImage<'a, B>],
        sets: &[impl AsRef<[B::DescriptorSetLayout]>],
    ) -> Self {
        assert_eq!(buffers.len(), 1);
        assert!(images.is_empty());
        assert_eq!(sets.len(), 1);
        assert!(sets[0].as_ref().is_empty());

        MeshRenderPass {
            vertex: *buffers[0].buffer,
        }
    }

    fn prepare(&mut self, factory: &mut Factory<B>, _aux: &T) -> bool {
        false
    }

    fn draw(
        &mut self,
        _layouts: &[B::PipelineLayout],
        pipelines: &[B::GraphicsPipeline],
        mut encoder: RenderPassInlineEncoder<'_, B>,
        _index: usize,
        _aux: &T,
    ) {
        let vbuf = self.vertex;
        encoder.bind_graphics_pipeline(&pipelines[0]);
        encoder.bind_vertex_buffers(0, Some((vbuf.raw(), 0)));
        // TODO: change here the number of vertices
        encoder.draw(0..6, 0..1);
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &mut T) {

    }
}

//#[cfg(any(feature = "dx12", feature = "metal", feature = "vulkan"))]
fn run(event_loop: &mut EventsLoop, factory: &mut Factory<Backend>, mut graph: Graph<Backend, ()>) -> Result<(), failure::Error> {

    let started = std::time::Instant::now();

    std::thread::spawn(move || {
        while started.elapsed() < std::time::Duration::new(30, 0) {
            std::thread::sleep(std::time::Duration::new(1, 0));
        }

        std::process::abort();
    });

    let mut frames = 0u64 ..;
    let mut elapsed = started.elapsed();

    for _ in &mut frames {
        event_loop.poll_events(|_| ());
        graph.run(factory, &mut ());

        elapsed = started.elapsed();
        if elapsed >= std::time::Duration::new(5, 0) {
            break;
        }
    }

    let elapsed_ns = elapsed.as_secs() * 1_000_000_000 + elapsed.subsec_nanos() as u64;

    log::info!("Elapsed: {:?}. Frames: {}. FPS: {}", elapsed, frames.start, frames.start * 1_000_000_000 / elapsed_ns);

    graph.dispose(factory, &mut ());
    Ok(())
}

//#[cfg(any(feature = "dx12", feature = "metal", feature = "vulkan"))]
pub fn draw(meshes: Vec<Mesh>) {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .filter_module("triangle", log::LevelFilter::Trace)
        .init();

    let config: Config = Default::default();

    let mut factory: Factory<Backend> = Factory::new(config).unwrap();

    let mut event_loop = EventsLoop::new();

    let window = WindowBuilder::new()
        .with_title("Rendy example")
        .build(&event_loop).unwrap();

    event_loop.poll_events(|_| ());

    let surface = factory.create_surface(window);

    let mut graph_builder = GraphBuilder::<Backend, ()>::new();

    let color = graph_builder.create_image(
        surface.kind(),
        1,
        gfx_hal::format::Format::Rgba8Unorm,
        MemoryUsageValue::Data,
        Some(gfx_hal::command::ClearValue::Color([0.29, 0.42, 0.67, 1.0].into())),
    );

    let mesh = meshes[0];
    let rendy_mesh = RendyMesh::builder()
        .with_indices(mesh.indices)
        .with_vertices(mesh.vertices)
        .build(QueueFamilyId(0), &factory);

    // TODO: change here the number of vertices
//    let mut vertex_buffer = factory.create_buffer(512, PosColor::VERTEX.stride as u64 * 6, (gfx_hal::buffer::Usage::VERTEX, MemoryUsageValue::Dynamic))
//        .unwrap();
//
//    unsafe {
//        // Fresh buffer.
//        // TODO: pass here the data
//        factory.upload_visible_buffer(&mut vertex_buffer, 0, &[
//            PosColor {
//                position: [0.0, -0.5, 0.0].into(),
//                color: [1.0, 0.0, 0.0, 1.0].into(),
//            },
//            PosColor {
//                position: [0.5, 0.5, 0.0].into(),
//                color: [0.0, 1.0, 0.0, 1.0].into(),
//            },
//            PosColor {
//                position: [-0.5, 0.5, 0.0].into(),
//                color: [0.0, 0.0, 1.0, 1.0].into(),
//            },
//            PosColor {
//                position: [1.0, -1.5, 1.0].into(),
//                color: [1.0, 0.0, 0.0, 1.0].into(),
//            },
//            PosColor {
//                position: [1.5, 1.5, 1.0].into(),
//                color: [0.0, 1.0, 0.0, 1.0].into(),
//            },
//            PosColor {
//                position: [-0.5, 0.5, 0.0].into(),
//                color: [0.0, 0.0, 1.0, 1.0].into(),
//            },
//        ]).unwrap();
//    }

    let pass = graph_builder.add_node(
        MeshRenderPass::builder()
            .with_image(color)
    );

    graph_builder.add_node(
        PresentNode::builder(surface)
            .with_image(color)
            .with_dependency(pass)
    );

    let graph = graph_builder.build(&mut factory, &mut ()).unwrap();

    run(&mut event_loop, &mut factory, graph).unwrap();

    factory.dispose();
}

//#[cfg(not(any(feature = "dx12", feature = "metal", feature = "vulkan")))]
//fn draw(_meshes: Vec<Mesh>) {
//    panic!("Specify feature: { dx12, metal, vulkan }");
//}