#![allow(dead_code)]

mod camera;
mod conveyor;
mod vec2_traits;

use conveyor::prelude::*;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_pixel_camera::{PixelCameraBundle, PixelCameraPlugin};
use camera::prelude::*;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn(PixelCameraBundle::from_zoom(6));

  let texture_handle: Handle<Image> = asset_server.load("conveyor.png");

  let map_size = TilemapSize { x: 32, y: 32 };

  let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
  let grid_size = tile_size.into();
  let map_type = TilemapType::Square;

  commands.spawn(TilemapBundle {
    grid_size,
    map_type,
    size: map_size,
    storage: TileStorage::empty(map_size),
    texture: TilemapTexture::Single(texture_handle),
    tile_size,
    transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
    ..default()
  });
}

pub trait OptionalResource<T> {
  fn resource_as_option(&self) -> Option<T>;
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_plugin(TilemapPlugin)
    .add_plugin(PixelCameraPlugin)
    .add_plugin(ConveyorBuildPlugin)
    .init_resource::<CursorPos>()
    .add_event::<CameraMoved>()
    .add_startup_system(startup)
    .add_system(movement)
    .add_system(update_cursor_pos)
    .run();
}
