#![allow(dead_code)]

mod camera;
mod conveyor;
mod vec2_traits;
mod helpers;

use bevy_egui::EguiPlugin;
use conveyor::prelude::*;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_pixel_camera::{PixelCameraBundle, PixelCameraPlugin};
use camera::prelude::*;

fn startup(mut commands: Commands) {
  commands.spawn(PixelCameraBundle::from_zoom(4));
}

pub trait OptionalResource<T> {
  fn resource_as_option(&self) -> Option<T>;
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_plugin(TilemapPlugin)
    .add_plugin(PixelCameraPlugin)
    .add_plugin(EguiPlugin)
    .add_plugin(ConveyorBuildPlugin {
      playfield_size: PlayfieldSize(UVec2 { x: 32, y: 32 }),
    })
    .insert_resource(ClearColor(Color::hex("151D28").unwrap()))
    .init_resource::<CursorPos>()
    .add_event::<CameraMoved>()
    .add_startup_system(startup)
    .add_system(movement)
    .add_system(update_cursor_pos)
    .run();
}
