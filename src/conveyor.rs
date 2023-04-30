use std::fmt::Display;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::camera::prelude::update_cursor_pos;

use self::placement::systems::*;
use self::update::systems::*;

pub mod placement;
pub mod update;

pub mod prelude {
  pub use super::ConveyorBuildPlugin;
  pub use super::ConveyorDirection;
  pub use super::PlayfieldSize;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Component)]
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
      ConveyorDirection::East => "East",
      ConveyorDirection::South => "South",
      ConveyorDirection::West => "West",
    }
  }

  pub const DIRECTION_VALUES: [IVec2; 4] = [IVec2::Y, IVec2::X, IVec2::NEG_Y, IVec2::NEG_X];

  pub fn neighbor_offsets(&self) -> [IVec2; 3] {
    match self {
      ConveyorDirection::North => [IVec2::X, IVec2::NEG_Y, IVec2::NEG_X],
      ConveyorDirection::East => [IVec2::NEG_Y, IVec2::NEG_X, IVec2::Y],
      ConveyorDirection::South => [IVec2::NEG_X, IVec2::Y, IVec2::X],
      ConveyorDirection::West => [IVec2::Y, IVec2::X, IVec2::NEG_Y],
    }
  }

  pub fn offset(&self) -> IVec2 {
    match self {
      ConveyorDirection::North => IVec2::Y,
      ConveyorDirection::East => IVec2::X,
      ConveyorDirection::South => IVec2::NEG_Y,
      ConveyorDirection::West => IVec2::NEG_X,
    }
  }

  pub fn neighbors_to_check_for_connections(&self) -> [ConveyorDirection; 3] {
    match self {
      ConveyorDirection::North => [
        ConveyorDirection::East,
        ConveyorDirection::South,
        ConveyorDirection::West,
      ],
      ConveyorDirection::East => [
        ConveyorDirection::South,
        ConveyorDirection::West,
        ConveyorDirection::North,
      ],
      ConveyorDirection::South => [
        ConveyorDirection::West,
        ConveyorDirection::North,
        ConveyorDirection::East,
      ],
      ConveyorDirection::West => [
        ConveyorDirection::North,
        ConveyorDirection::East,
        ConveyorDirection::South,
      ],
    }
  }
}
#[derive(Debug, Component)]
pub struct BackgroundTileLayer;
#[derive(Debug, Component)]
pub struct ConveyorTileLayer;

#[derive(Debug, Resource, Clone)]
pub struct PlayfieldSize(pub UVec2);

fn setup_conveyor(
  playfield_size: Res<PlayfieldSize>,
  mut commands: Commands,
  asset_server: Res<AssetServer>,
) {
  let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
  let grid_size = tile_size.into();
  let map_type = TilemapType::Square;

  let background_tilemap = commands.spawn_empty().id();
  let background_texture_handle: Handle<Image> = asset_server.load("background.png");
  let background_map_size = TilemapSize {
    x: playfield_size.0.x + 2,
    y: playfield_size.0.y + 2,
  };
  let mut background_storage = TileStorage::empty(background_map_size);

  for x in 0..background_map_size.x {
    for y in 0..background_map_size.y {
      let position = TilePos { x, y };
      let tile_entity = commands
        .spawn(TileBundle {
          position,
          tilemap_id: TilemapId(background_tilemap),
          texture_index: TileTextureIndex(1),
          ..default()
        })
        .id();
      background_storage.set(&position, tile_entity);
    }
  }

  commands.entity(background_tilemap).insert(TilemapBundle {
    grid_size,
    map_type,
    size: background_map_size,
    storage: background_storage,
    texture: TilemapTexture::Single(background_texture_handle),
    tile_size,
    transform: get_tilemap_center_transform(&background_map_size, &grid_size, &map_type, 0.0),
    ..default()
  }).insert(BackgroundTileLayer);

  let playfield_tilemap = commands.spawn_empty().id();
  let playfield_texture_handle: Handle<Image> = asset_server.load("conveyor.png");
  let playfield_map_size = TilemapSize {
    x: playfield_size.0.x,
    y: playfield_size.0.y,
  };
  let mut playfield_storage = TileStorage::empty(background_map_size);

  for x in 0..playfield_map_size.x {
    for y in 0..playfield_map_size.y {
      let position = TilePos { x, y };
      let tile_entity = commands
        .spawn(TileBundle {
          position,
          tilemap_id: TilemapId(playfield_tilemap),
          ..default()
        })
        .id();
      playfield_storage.set(&position, tile_entity);
    }
  }

  commands.entity(playfield_tilemap).insert(TilemapBundle {
    grid_size,
    map_type,
    size: playfield_map_size,
    storage: playfield_storage,
    texture: TilemapTexture::Single(playfield_texture_handle),
    tile_size,
    transform: get_tilemap_center_transform(&background_map_size, &grid_size, &map_type, 10.0),
    ..default()
  }).insert(ConveyorTileLayer);
}

pub struct ConveyorBuildPlugin {
  pub playfield_size: PlayfieldSize,
}

impl Plugin for ConveyorBuildPlugin {
  fn build(&self, app: &mut bevy::prelude::App) {
    app
      .init_resource::<placement::PreviousTilePlaceAttempt>()
      .insert_resource(self.playfield_size.clone())
      .add_event::<placement::UpdateTile>()
      .add_event::<placement::PlaceTile>()
      .add_event::<update::ChangeConveyorDirection>()
      .add_startup_system(setup_conveyor)
      .add_systems(
        (
          detect_tile_place,
          place_tiles_drag,
          apply_system_buffers,
          update_tile_direction,
          conveyor_tile_update,
        )
          .after(update_cursor_pos)
          .chain(),
      );
  }
}
