mod camera;

use bevy::{math::Vec4Swizzles, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use bevy_pixel_camera::{PixelCameraBundle, PixelCameraPlugin};
use camera::CameraMoved;

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

#[derive(Resource)]
pub struct CursorPos(Vec2, Vec2);
impl Default for CursorPos {
  fn default() -> Self {
    // Initialize the cursor pos at some far away place. It will get updated
    // correctly when the cursor moves.
    Self(Vec2::new(-1000.0, -1000.0), Vec2::ZERO)
  }
}

impl CursorPos {
  pub fn to_map_pos(&self, map_transform: &Transform) -> Vec2 {
    // Grab the cursor position from the `Res<CursorPos>`
    let cursor_pos: Vec2 = self.0;
    // We need to make sure that the cursor's world position is correct relative to the map
    // due to any map transformation.
    let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
    let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
    cursor_in_map_pos.xy()
  }
}

// We need to keep the cursor position updated based on any `CursorMoved` events.
pub fn update_cursor_pos(
  camera_q: Query<(&GlobalTransform, &Camera)>,
  mut cursor_moved_events: EventReader<CursorMoved>,
  camera_moved_events: EventReader<CameraMoved>,
  mut cursor_pos: ResMut<CursorPos>,
) {
  if cursor_moved_events.len() == 0 && camera_moved_events.len() != 0 {
    for (cam_t, cam) in camera_q.iter() {
      if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_pos.1) {
        *cursor_pos = CursorPos(pos, cursor_pos.1);
      }
    }
    return;
  }

  for cursor_moved in cursor_moved_events.iter() {
    // To get the mouse's world position, we have to transform its window position by
    // any transforms on the camera. This is done by projecting the cursor position into
    // camera space (world space).
    for (cam_t, cam) in camera_q.iter() {
      if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
        *cursor_pos = CursorPos(pos, cursor_moved.position);
      }
    }
  }
}

fn place_tile(
  cursor_pos: ResMut<CursorPos>,
  mut tilemaps: Query<(
    Entity,
    &mut TileStorage,
    &TilemapGridSize,
    &TilemapType,
    &TilemapSize,
    &Transform,
  )>,
  mut commands: Commands,
) {
  let Ok((tilemap_entity, mut tile_storage, grid_size, map_type, map_size, map_transform)) = tilemaps.get_single_mut() else { return; };
  let Some(tile_pos) = TilePos::from_world_pos(&cursor_pos.to_map_pos(map_transform), map_size, grid_size, map_type) else { return; };

  let tile_entity = commands
    .spawn(TileBundle {
      position: tile_pos,
      tilemap_id: TilemapId(tilemap_entity),
      texture_index: TileTextureIndex(1),
      ..Default::default()
    })
    .id();
  tile_storage.set(&tile_pos, tile_entity);
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_plugin(TilemapPlugin)
    .add_plugin(PixelCameraPlugin)
    .init_resource::<CursorPos>()
    .add_event::<CameraMoved>()
    .add_startup_system(startup)
    .add_system(camera::movement)
    .add_system(update_cursor_pos)
    .add_system(place_tile.after(update_cursor_pos))
    .run();
}
