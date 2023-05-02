use std::fmt::Display;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::helpers::grid_traversal::GridTraversal;
use crate::input::chained_tile::{ChainedTileChangeEvent, ChainedTilePlaceDirection, ChainedTileChangePosition};
use crate::input::prelude::*;
use crate::GameSystemSet;
use crate::vec2_traits::TilePosFromSigned;

use self::background::plugin_exports::*;
use self::placement::plugin_exports::*;
use self::removal::plugin_exports::*;
use self::update_graphics::systems::*;
use self::playfield::plugin_exports::*;

pub mod placement;
pub mod removal;
pub mod update_graphics;
mod background;
mod playfield;

pub mod prelude {
  pub use super::ConveyorBuildPlugin;
  pub use super::ConveyorDirection;
  pub use super::UpdatedTile;
  pub use super::playfield::prelude::*;
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

pub struct ConveyorBuildPlugin {
  pub playfield_size: PlayfieldSize, include_background: bool, include_textures: bool,
}

impl ConveyorBuildPlugin {
  pub fn new(playfield_size: PlayfieldSize) -> ConveyorBuildPlugin {
    ConveyorBuildPlugin { playfield_size, include_background: true, include_textures: true, }
  }

  pub fn new_no_background(playfield_size: PlayfieldSize) -> ConveyorBuildPlugin {
    ConveyorBuildPlugin { playfield_size, include_background: false, include_textures: true, }
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

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum TileSetupSystemSet {
  SpawnTilemaps,
  InsertTileData,
}

impl Plugin for ConveyorBuildPlugin {
  fn build(&self, app: &mut bevy::prelude::App) {
    app
      .init_resource::<PreviousPlaceTileAttempt>()
      .insert_resource(self.playfield_size.clone())
      .add_event::<UpdatedTile>()
      .add_event::<ChainedTileChangeEvent>()
      .add_startup_system(setup_playfield.in_set(TileSetupSystemSet::SpawnTilemaps))
      .add_startup_system(apply_system_buffers.after(TileSetupSystemSet::SpawnTilemaps).before(TileSetupSystemSet::InsertTileData))
      .add_systems(
        (
          catch_chained_tile_change_events,
          apply_system_buffers,
          conveyor_tile_update_graphics,
        )
          .in_set(GameSystemSet::TilePlacing)
          .chain()
      );

    if self.include_background {
      app.add_startup_system(setup_background_tilemap.in_set(TileSetupSystemSet::SpawnTilemaps));
      if self.include_textures {
        app.add_startup_systems((insert_background_texture, place_background_tiles).in_set(TileSetupSystemSet::InsertTileData));
      }
    }

    if self.include_textures {
      app.add_startup_system(insert_playfield_texture.in_set(TileSetupSystemSet::InsertTileData));
    }

    if !app.world.is_resource_added::<SelectedTileDirection>() {
      app.init_resource::<SelectedTileDirection>();
    }
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

#[cfg(test)]
mod tile_test {
  use bevy::prelude::*;

  use crate::input::chained_tile::{ChainedTileChangeType, TileType};

  use super::*;

  #[test]
  fn place_single_conveyor() {
    let mut app = App::new();
  
    app.add_plugin(ConveyorBuildPlugin { playfield_size: PlayfieldSize(UVec2::new(8, 8)), include_background: false, include_textures: false});

    app.setup();
  
    app.update();
  
    app.world.send_event(ChainedTileChangeEvent {
      position: ChainedTileChangePosition::Single(IVec2::ONE),
      change_type: ChainedTileChangeType::Put { tile_type: TileType::Conveyor, chain: false, direction: ChainedTilePlaceDirection::Normal }
    });
  
    app.update();

    let mut tiles = app.world.query::<(&TilePos, &TileTextureIndex, &ConveyorDirection)>();

    let (tile_pos, tile_texture_index, conveyor_direction) = tiles.get_single_mut(&mut app.world).unwrap();

    assert_eq!(*tile_pos, TilePos { x: 1, y:1 });
    assert_eq!(tile_texture_index.0, ConveyorDirection::North.texture_index());
    assert_eq!(*conveyor_direction, ConveyorDirection::North);
  }

  #[test]
  fn place_conveyor_line() {
    let mut app = App::new();
  
    app.add_plugin(ConveyorBuildPlugin { playfield_size: PlayfieldSize(UVec2::new(8, 8)), include_background: false, include_textures: false});

    app.setup();
  
    app.update();
  
    app.world.send_event(ChainedTileChangeEvent {
      position: ChainedTileChangePosition::StraightLine { start: IVec2::ONE, end: IVec2::new(1, 6) },
      change_type: ChainedTileChangeType::Put { tile_type: TileType::Conveyor, chain: false, direction: ChainedTilePlaceDirection::Normal }
    });
  
    app.update();

    let mut tile_query = app.world.query::<(&TilePos, &TileTextureIndex, &ConveyorDirection)>();
    let mut tiles = tile_query.iter(&app.world);

    for i in 0..5 {
      let (tile_pos, tile_texture_index, conveyor_direction) = tiles.next().unwrap();

      assert_eq!(*tile_pos, TilePos { x: 1, y: 2 + i as u32 });
      assert_eq!(tile_texture_index.0, ConveyorDirection::North.texture_index());
      assert_eq!(*conveyor_direction, ConveyorDirection::North);
    }

    assert!(tiles.next().is_none());
  }
}
