mod camera;
mod tile_placement;
mod vec2_traits;

use std::fmt::Display;

use tile_placement::*;

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

#[derive(Debug, Resource)]
pub struct PreviousTilePlacePosition(Option<IVec2>);

impl FromWorld for PreviousTilePlacePosition {
  fn from_world(_world: &mut World) -> Self {
    PreviousTilePlacePosition(None)
  }
}

fn detect_tile_place(
  cursor_pos: ResMut<CursorPos>,
  mouse_click: ResMut<Input<MouseButton>>,
  mut previous_tile_place_position: ResMut<PreviousTilePlacePosition>,
  mut place_tile_event: EventWriter<PlaceTile>,
  mut tilemaps: Query<(&TilemapGridSize, &Transform)>,
) {
  let Ok((grid_size, map_transform)) = tilemaps.get_single_mut() else { return; };
  let tile_pos = (cursor_pos.to_map_pos(map_transform) / Vec2::new(grid_size.x, grid_size.y)
    + Vec2::new(0.5, 0.5))
  .as_ivec2();
  if mouse_click.pressed(MouseButton::Left) {
    match previous_tile_place_position.0 {
      Some(previous_tile_place) => {
        if tile_pos != previous_tile_place {
          place_tile_event.send(PlaceTile::new(previous_tile_place, tile_pos));
        }
      }
      None => place_tile_event.send(PlaceTile::new_single_pos(tile_pos)),
    }
    previous_tile_place_position.0 = Some(tile_pos);
  } else {
    previous_tile_place_position.0 = None;
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ConveyorDirection {
  North,
  South,
  East,
  West,
}

impl Display for ConveyorDirection {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.name())
  }
}

impl ConveyorDirection {
  pub fn texture_index(&self) -> u32 {
    match self {
      ConveyorDirection::North => 1,
      ConveyorDirection::East => 2,
      ConveyorDirection::South => 3,
      ConveyorDirection::West => 4,
    }
  }

  pub fn opposite(&self) -> ConveyorDirection {
    match self {
      ConveyorDirection::North => ConveyorDirection::South,
      ConveyorDirection::East => ConveyorDirection::West,
      ConveyorDirection::South => ConveyorDirection::North,
      ConveyorDirection::West => ConveyorDirection::East,
    }
  }

  pub fn name(&self) -> &'static str {
    match self {
      ConveyorDirection::North => "North",
      ConveyorDirection::South => "South",
      ConveyorDirection::East => "East",
      ConveyorDirection::West => "West",
    }
  }
}

#[derive(Debug)]
pub struct ChangeConveyorDirection {
  pub entity: Entity,
  pub direction: ConveyorDirection,
}

fn update_tile_direction(
  mut change_conveyor_detection: EventReader<ChangeConveyorDirection>,
  mut tiles: Query<(Entity, &mut TileTextureIndex, &TilePos)>,
) {
  if change_conveyor_detection.len() != 0 {
    println!("changes to make: {}", change_conveyor_detection.len());
  }

  for change_conveyor_direction in change_conveyor_detection.iter() {
    let Ok(mut tile) = tiles.get_mut(change_conveyor_direction.entity) else {
      println!("could not find tile");
      continue;
    };
    println!(
      "Updating tile at pos {:?} to direction {}",
      tile.2, change_conveyor_direction.direction,
    );
    *tile.1 = TileTextureIndex(change_conveyor_direction.direction.texture_index());
  }
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_plugin(TilemapPlugin)
    .add_plugin(PixelCameraPlugin)
    .init_resource::<CursorPos>()
    .init_resource::<PreviousTilePlacePosition>()
    .add_event::<CameraMoved>()
    .add_event::<PlaceTile>()
    .add_event::<ChangeConveyorDirection>()
    .add_startup_system(startup)
    .add_system(camera::movement)
    .add_systems(
      (
        update_cursor_pos,
        detect_tile_place,
        place_tiles_drag,
        apply_system_buffers,
        update_tile_direction,
      )
        .chain(),
    )
    .run();
}
