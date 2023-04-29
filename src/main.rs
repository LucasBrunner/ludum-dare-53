mod camera;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_pixel_camera::{PixelCameraPlugin, PixelCameraBundle};

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands
    .spawn(PixelCameraBundle::from_zoom(4));

  let texture_handle: Handle<Image> = asset_server.load("tiles.png");

  let map_size = TilemapSize { x: 32, y: 32 };

  let tilemap_entity = commands.spawn_empty().id();

  let mut tile_storage = TileStorage::empty(map_size);

  for x in 0..map_size.x {
    for y in 0..map_size.y {
      let tile_pos = TilePos { x, y };
      let tile_entity = commands
        .spawn(TileBundle {
          position: tile_pos,
          tilemap_id: TilemapId(tilemap_entity),
          texture_index: TileTextureIndex(((x % 6) + (y % 6)) % 6),
          ..default()
        })
        .id();
      tile_storage.set(&tile_pos, tile_entity);
    }
  }

  let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
  let grid_size = tile_size.into();
  let map_type = TilemapType::Square;

  commands.entity(tilemap_entity).insert(TilemapBundle {
    grid_size,
    map_type,
    size: map_size,
    storage: tile_storage,
    texture: TilemapTexture::Single(texture_handle),
    tile_size,
    transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
    ..default()
  });
}

fn swap_texture_or_hide(
  asset_server: Res<AssetServer>,
  keyboard_input: Res<Input<KeyCode>>,
  mut query: Query<(&mut TilemapTexture, &mut Visibility)>,
) {
  if keyboard_input.just_pressed(KeyCode::Space) {
    let texture_a = TilemapTexture::Single(asset_server.load("tiles.png"));
    let texture_b = TilemapTexture::Single(asset_server.load("tiles2.png"));
    for (mut tilemap_tex, _) in &mut query {
      if *tilemap_tex == texture_a {
        *tilemap_tex = texture_b.clone();
      } else {
        *tilemap_tex = texture_a.clone();
      }
    }
  }
  if keyboard_input.just_pressed(KeyCode::H) {
    for (_, mut visibility) in &mut query {
      *visibility = match *visibility {
        Visibility::Inherited | Visibility::Visible => Visibility::Hidden,
        Visibility::Hidden => Visibility::Visible,
      };
    }
  }
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_plugin(TilemapPlugin)
    .add_plugin(PixelCameraPlugin)
    .add_startup_system(startup)
    .add_system(swap_texture_or_hide)
    .add_system(camera::movement)
    .run();
}
