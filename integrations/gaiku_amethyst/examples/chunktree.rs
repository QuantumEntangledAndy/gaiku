/// Chunktree
///
/// This example shows how to create a LOD terrain using the chunk tree
use amethyst::{
  controls::FlyControlBundle,
  core::transform::TransformBundle,
  input::{InputBundle, StringBindings},
  prelude::*,
  renderer::{
    palette::Srgb,
    plugins::{RenderShaded3D, RenderSkybox, RenderToWindow},
    types::DefaultBackend,
    RenderingBundle,
  },
  ui::{RenderUi, UiBundle},
  utils::application_root_dir,
};

mod chunktree_example;

use crate::chunktree_example::game::GameLoad;

fn main() -> amethyst::Result<()> {
  amethyst::start_logger(Default::default());

  let app_root = application_root_dir()?;
  let assets_dir = app_root.join("examples").join("assets");

  let display_config_path = assets_dir.join("display.ron");

  let binding_path = assets_dir.join("bindings.ron");
  let input_bundle = InputBundle::<StringBindings>::new().with_bindings_from_file(binding_path)?;

  let render_bund = RenderingBundle::<DefaultBackend>::new()
    // The RenderToWindow plugin provides all the scaffolding for opening a window and drawing on it
    .with_plugin(
      RenderToWindow::from_config_path(display_config_path)?.with_clear([0.0, 0.0, 0.0, 1.0]),
    )
    .with_plugin(RenderShaded3D::default())
    .with_plugin(RenderUi::default())
    .with_plugin(RenderSkybox::with_colors(
      Srgb::new(0.82, 0.51, 0.50),
      Srgb::new(0.18, 0.11, 0.85),
    ));

  let game_data = GameDataBuilder::default()
    .with_bundle(render_bund)?
    // With transform systems for position tracking
    .with_bundle(TransformBundle::new())?
    .with_bundle(FlyControlBundle::<StringBindings>::new(
      Some(String::from("right")),
      Some(String::from("up")),
      Some(String::from("forward")),
    ))?
    .with_bundle(input_bundle)?
    .with_bundle(UiBundle::<StringBindings>::new())?;

  let mut game = Application::new(assets_dir, GameLoad::new(), game_data)?;

  game.run();
  Ok(())
}
