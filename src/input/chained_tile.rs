use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilemapGridSize;

use crate::{camera::prelude::CursorPos, tile::ConveyorTileLayer, vec2_traits::ToVec2};

pub mod prelude {
  pub use super::ChainedTileChangePosition;
  pub use super::ChainedTileChangeEvent;
}

pub mod plugin_exports {
  pub use super::catch_chained_tile_input;
  pub use super::ChainedTileChangeEvent;
  pub use super::ChainedTileResource;
}

#[derive(Debug, Clone, Copy)]
pub enum TileType {
  Conveyor,
}

#[derive(Debug, Clone, Copy)]
pub enum ChainedTileChangePosition {
  Single(IVec2),
  StraightLine{
    start: IVec2,
    end: IVec2,
  },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainedTilePlaceDirection {
  Normal,
  Revesed,
}

impl ChainedTilePlaceDirection {
  fn new(reversed: bool) -> ChainedTilePlaceDirection {
    match reversed {
      true => ChainedTilePlaceDirection::Revesed,
      false => ChainedTilePlaceDirection::Normal,
    }
  }
}

#[derive(Debug)]
pub enum ChainedTileChangeType {
  Put{
    tile_type: TileType,
    chain: bool,
    direction: ChainedTilePlaceDirection,
  },
  Delete,
}

impl ChainedTileChangeType {
  fn put(
    tile_type: TileType,
    chain: bool,
    direction: ChainedTilePlaceDirection,
  ) -> ChainedTileChangeType {
    ChainedTileChangeType::Put { tile_type, chain, direction }
  }
}

#[derive(Debug)]
pub struct ChainedTileChangeEvent {
  pub position: ChainedTileChangePosition,
  pub change_type: ChainedTileChangeType,
}

impl ChainedTileChangeEvent {
  fn new(
    position: ChainedTileChangePosition,
    change_type: ChainedTileChangeType,
  ) -> ChainedTileChangeEvent {
    ChainedTileChangeEvent { position, change_type }
  }
}

#[derive(Debug, Resource, Default, Reflect)]
pub struct ChainedTileResource {
  mouse_state: (bool, bool),
  cursor_tile_position: IVec2,
}

pub fn catch_chained_tile_input(
  mut chained_tile_event_writer: EventWriter<ChainedTileChangeEvent>,
  mut previous_frame_data: ResMut<ChainedTileResource>,
  keyboard_input: Res<Input<KeyCode>>,
  mouse_input: ResMut<Input<MouseButton>>,
  cursor_pos: Res<CursorPos>,
  tilemap: Query<(&TilemapGridSize, &Transform, &ConveyorTileLayer)>,
) {
  // get the tilemap
  let Ok((tilemap_grid_size, tilemap_transform, _)) = tilemap.get_single() else { 
    error!(
      "Tilemap query for the conveyor layer returned {} items when it only should have returned 1.", 
      tilemap.iter().len(),
    );
    return;
  };
  // convert cursor position coordinates to tilemap units
  let mut cursor_tile_position = cursor_pos.to_map_pos(tilemap_transform) / tilemap_grid_size.to_vec2();
  // account for tile offset
  cursor_tile_position += Vec2::new(0.5, 0.5); 
  // cursor position in tile-space
  let cursor_tile_position = cursor_tile_position.floor().as_ivec2();

  let mouse_state = (
    mouse_input.pressed(MouseButton::Left),
    mouse_input.pressed(MouseButton::Right),
  );
  
  if cursor_tile_position != previous_frame_data.cursor_tile_position || mouse_state != previous_frame_data.mouse_state {
    let change_type = match mouse_state {
      (true, false) => Some(ChainedTileChangeType::put(TileType::Conveyor, previous_frame_data.mouse_state.0, ChainedTilePlaceDirection::new(keyboard_input.pressed(KeyCode::LShift)))),
      (false, true) => Some(ChainedTileChangeType::Delete),
      _ => None,
    };

    if let Some(change_type) = change_type {
      let position = match cursor_tile_position == previous_frame_data.cursor_tile_position {
        true => ChainedTileChangePosition::Single(cursor_tile_position),
        false => ChainedTileChangePosition::StraightLine {
          start: previous_frame_data.cursor_tile_position, 
          end: cursor_tile_position, 
        },
      };

      chained_tile_event_writer.send(ChainedTileChangeEvent::new(position, change_type));
    }
  }

  // save data to use next frame
  *previous_frame_data = ChainedTileResource {
    mouse_state,
    cursor_tile_position,
  };
}