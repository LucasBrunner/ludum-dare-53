use std::fmt::Display;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_egui::egui::{Align2, Id, Pos2, Style};
use bevy_egui::{egui, EguiContexts};

use crate::camera::prelude::*;
use crate::helpers::grid_traversal::GridTraversal;
use crate::input::chained_tile::{ChainedTileChangeEvent, ChainedTilePlaceDirection, ChainedTileChangePosition};
use crate::input::prelude::tile_rotation::prelude::*;
use crate::GameSystemSet;
use crate::vec2_traits::TilePosFromSigned;

use self::placement::plugin_exports::*;
use self::removal::plugin_exports::*;
use self::update_graphics::systems::*;

pub mod placement;
pub mod removal;
pub mod update_graphics;

pub mod prelude {
  pub use super::ConveyorBuildPlugin;
  pub use super::ConveyorDirection;
  pub use super::UpdatedTile;
  pub use super::PlayfieldSize;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Component, Reflect)]
pub enum ConveyorDirection {
  North,
  South,
  East,
  West,
}

impl Default for ConveyorDirection {
  fn default() -> Self {
    ConveyorDirection::North
  }
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

  pub fn reverse(&self, reverse: bool) -> ConveyorDirection {
    match reverse {
        true => self.opposite(),
        false => *self,
    }
  }

  pub fn apply_place_direction(&self, direction: ChainedTilePlaceDirection) -> ConveyorDirection {
    self.reverse(direction == ChainedTilePlaceDirection::Revesed)
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

  pub fn rotate_clockwise(&self) -> ConveyorDirection {
    match self {
        ConveyorDirection::North => ConveyorDirection::East,
        ConveyorDirection::South => ConveyorDirection::West,
        ConveyorDirection::East => ConveyorDirection::South,
        ConveyorDirection::West => ConveyorDirection::North,
    }
  }

  pub fn rotate_counterclockwise(&self) -> ConveyorDirection {
    match self {
        ConveyorDirection::North => ConveyorDirection::West,
        ConveyorDirection::East => ConveyorDirection::North,
        ConveyorDirection::South => ConveyorDirection::East,
        ConveyorDirection::West => ConveyorDirection::South,
    }
  }

  fn from_x_y(x: i32, y: i32) -> Option<ConveyorDirection> {
    match (x, y) {
      (0, 1) => Some(ConveyorDirection::North),
      (1, 0) => Some(ConveyorDirection::East),
      (0, -1) => Some(ConveyorDirection::South),
      (-1, 0) => Some(ConveyorDirection::West),
      _ => None
    }
  }

  pub fn from_vec2(vector: Vec2) -> Option<ConveyorDirection> {
    Self::from_x_y(vector.x.signum() as i32, vector.y.signum() as i32)
  }

  pub fn from_ivec2(vector: IVec2) -> Option<ConveyorDirection> {
    Self::from_x_y(vector.x.signum(), vector.y.signum())
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
      let texture_index = match (
        x == 0,
        y == 0,
        x == background_map_size.x - 1,
        y == background_map_size.y - 1,
      ) {
        (true, _, _, true) => 13,
        (true, true, _, _) => 12,
        (_, true, true, _) => 11,
        (_, _, true, true) => 10,
        (_, _, true, _) => 9,
        (_, _, _, true) => 8,
        (true, _, _, _) => 7,
        (_, true, _, _) => 6,
        _ => 1,
      };
      let position = TilePos { x, y };
      let tile_entity = commands
        .spawn(TileBundle {
          position,
          tilemap_id: TilemapId(background_tilemap),
          texture_index: TileTextureIndex(texture_index),
          ..default()
        })
        .id();
      background_storage.set(&position, tile_entity);
    }
  }

  commands
    .entity(background_tilemap)
    .insert(TilemapBundle {
      grid_size,
      map_type,
      size: background_map_size,
      storage: background_storage,
      texture: TilemapTexture::Single(background_texture_handle),
      tile_size,
      transform: get_tilemap_center_transform(&background_map_size, &grid_size, &map_type, 0.0),
      ..default()
    })
    .insert(BackgroundTileLayer);

  let playfield_tilemap = commands.spawn_empty().id();
  let playfield_texture_handle: Handle<Image> = asset_server.load("conveyor.png");
  let playfield_map_size = TilemapSize {
    x: playfield_size.0.x,
    y: playfield_size.0.y,
  };
  let playfield_storage = TileStorage::empty(background_map_size);

  commands
    .entity(playfield_tilemap)
    .insert(TilemapBundle {
      grid_size,
      map_type,
      size: playfield_map_size,
      storage: playfield_storage,
      texture: TilemapTexture::Single(playfield_texture_handle),
      tile_size,
      transform: get_tilemap_center_transform(&playfield_map_size, &grid_size, &map_type, 10.0),
      ..default()
    })
    .insert(ConveyorTileLayer);
}

pub struct ConveyorBuildPlugin {
  pub playfield_size: PlayfieldSize,
}

impl ConveyorBuildPlugin {
  pub fn new(playfield_size: PlayfieldSize) -> ConveyorBuildPlugin {
    ConveyorBuildPlugin { playfield_size }
  }
}

#[derive(Default, Resource)]
struct UiState {
  // egui_texture_handle: Option<egui::TextureHandle>,
  conveyor_atlas: Option<Handle<TextureAtlas>>,
}

fn setup_conveyor_ui(
  mut texture_atlases: ResMut<Assets<TextureAtlas>>,
  asset_server: Res<AssetServer>,
  mut ui_state: ResMut<UiState>,
) {
  let conveyor_texture = asset_server.load("conveyor.png");
  let texture_atlas =
    TextureAtlas::from_grid(conveyor_texture, Vec2::new(16.0, 16.0), 29, 1, None, None);
  let texture_atlas_handle = texture_atlases.add(texture_atlas);
  ui_state.conveyor_atlas = Some(texture_atlas_handle);
}

fn conveyor_window(
  tile_rotation: Res<SelectedTileDirection>,
  mut contexts: EguiContexts,
  asset_server: Res<AssetServer>,
) {
  let image = contexts.add_image(asset_server.load("conveyor.png"));

  let ctx = contexts.ctx_mut();

  let offset = 16.0 * tile_rotation.direction.texture_index() as f32;
  let uv = egui::Rect::from_two_pos(
    Pos2::new(offset / 464.0, 0.0),
    Pos2::new((16.0 + offset) / 464.0, 1.0),
  );

  egui::Area::new(Id::null())
    .anchor(Align2::RIGHT_BOTTOM, egui::Vec2::ZERO)
    .show(ctx, |ui| {
      egui::Frame::side_top_panel(&Style::default()).show(ui, |ui| {
        ui.add(egui::widgets::Image::new(image, [64.0, 64.0]).uv(uv));
      })
    });
}

impl Plugin for ConveyorBuildPlugin {
  fn build(&self, app: &mut bevy::prelude::App) {
    app
      .init_resource::<PreviousPlaceTileAttempt>()
      .insert_resource(self.playfield_size.clone())
      .add_event::<UpdatedTile>()
      .add_event::<ChainedTileChangeEvent>()
      .add_startup_system(setup_conveyor)
      .add_systems(
        (
          catch_chained_tile_change_events,
          apply_system_buffers,
          // update_conveyor_direction,
          conveyor_tile_update_graphics,
        )
          .after(update_cursor_pos)
          .in_set(GameSystemSet::Conveyor)
          .chain(),
      )
      .add_system(conveyor_window.after(catch_chained_tile_change_events));
  }
}

#[derive(Debug, Clone)]
pub struct UpdatedTile {
  pub pos: TilePos,
}

pub fn catch_chained_tile_change_events(
  mut commands: Commands,
  mut place_tile_events: EventReader<ChainedTileChangeEvent>,
  mut placed_tiles: EventWriter<UpdatedTile>,
  mut tilemap: Query<(Entity, &mut TileStorage, &TilemapSize, &ConveyorTileLayer)>,
  mut previous_tile_attempt: ResMut<PreviousPlaceTileAttempt>,
  selected_tile_rotation: Res<SelectedTileDirection>,
) {
  let Ok((tilemap_entity, mut tile_storage, tilemap_size, _)) = tilemap.get_single_mut() else { 
    error!(
      "Tilemap query for the conveyor layer returned {} items when it only should have returned 1.", 
      tilemap.iter().len(),
    );
    return; 
  };

  for place_tile_event in place_tile_events.iter() {   
    let positions = match place_tile_event.position {
      ChainedTileChangePosition::Single(position) => GridTraversal::new(position, position).add_iterations(1),
      ChainedTileChangePosition::StraightLine { start, end } => GridTraversal::new(start, end),
    };

    for position in positions {
      match place_tile_event.change_type {
        crate::input::chained_tile::ChainedTileChangeType::Put { tile_type: _tile_type, chain, direction } => {
          place_tile(&mut commands, position, &mut tile_storage, tilemap_entity, tilemap_size, &mut previous_tile_attempt, &mut placed_tiles, direction, selected_tile_rotation.direction, chain);
        },
        crate::input::chained_tile::ChainedTileChangeType::Delete => {
          if let Ok(position) = position.to_tile_pos(&tilemap_size) {
            despawn_conveyor(&mut commands, position, &mut tile_storage, &mut placed_tiles);
          }
        },
      }
    }
  }
}
