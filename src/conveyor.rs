use std::fmt::Display;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_egui::egui::{Align2, Id, Pos2, Style};
use bevy_egui::{egui, EguiContexts};

use crate::camera::prelude::*;
use crate::input::prelude::*;
use crate::GameSystemSet;

use self::placement::{systems::*, PreviouslyPlacedTile};
use self::removal::remove_conveyors_drag;
use self::update::systems::*;

pub mod placement;
pub mod removal;
pub mod update;

pub mod prelude {
  pub use super::ConveyorBuildPlugin;
  pub use super::ConveyorDirection;
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
  previous_tile: Res<PreviouslyPlacedTile>,
  mut contexts: EguiContexts,
  asset_server: Res<AssetServer>,
) {
  let image = contexts.add_image(asset_server.load("conveyor.png"));

  let ctx = contexts.ctx_mut();

  let offset = 16.0 * previous_tile.direction.texture_index() as f32;
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
      .init_resource::<placement::PreviousMouseConveyorInput>()
      .init_resource::<PreviouslyPlacedTile>()
      .insert_resource(self.playfield_size.clone())
      .add_event::<placement::TileUpdate>()
      .add_event::<placement::PlaceConveyor>()
      .add_event::<removal::RemoveConveyor>()
      .add_event::<update::ChangeConveyorDirection>()
      .add_startup_system(setup_conveyor)
      .add_systems(
        (
          detect_conveyor_input.run_if(not(mouse_captured)),
          rotate_conveyor_placement.run_if(not(keyboard_captured)),
        )
          .in_set(GameSystemSet::InputCollection),
      )
      .add_systems(
        (
          place_conveyors_drag,
          remove_conveyors_drag,
          apply_system_buffers,
          update_conveyor_direction,
          conveyor_tile_update,
        )
          .after(update_cursor_pos)
          .in_set(GameSystemSet::Conveyor)
          .chain(),
      )
      .add_system(conveyor_window.after(place_conveyors_drag));
  }
}
