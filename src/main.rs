#![allow(dead_code)]

mod camera;
mod tile;
mod helpers;
mod input;
mod vec2_traits;

use bevy_egui::EguiPlugin;
use tile::prelude::*;

use bevy::{prelude::*, ecs::schedule::SystemSetConfig};
use bevy_ecs_tilemap::prelude::*;
use bevy_pixel_camera::{PixelCameraBundle, PixelCameraPlugin};
use camera::prelude::*;
use input::prelude::*;

fn startup(mut commands: Commands) {
  commands.spawn(PixelCameraBundle::from_zoom(4));
}

pub trait OptionalResource<T> {
  fn resource_as_option(&self) -> Option<T>;
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSystemSet {
  Egui,
  PreInputCollection,
  InputCollection,
  Conveyor,
}

impl GameSystemSet {
  fn configure_sets() -> (SystemSetConfig, SystemSetConfig) {
    (
      GameSystemSet::PreInputCollection.before(GameSystemSet::InputCollection),
      GameSystemSet::InputCollection.before(GameSystemSet::Conveyor),
    )
  }
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_plugin(TilemapPlugin)
    .add_plugin(PixelCameraPlugin)
    .add_plugin(EguiPlugin)
    .add_plugin(InputPlugin)
    .add_plugin(ConveyorBuildPlugin::new(PlayfieldSize(UVec2::new(32, 32))))
    .insert_resource(ClearColor(Color::hex("151D28").unwrap()))
    .init_resource::<CursorPos>()
    .add_event::<CameraMoved>()
    .add_startup_system(startup)
    .add_system(movement)
    .add_system(update_cursor_pos.in_set(GameSystemSet::InputCollection))
    .configure_sets(GameSystemSet::configure_sets())
    .run();
}
