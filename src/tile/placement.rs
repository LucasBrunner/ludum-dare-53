use bevy::prelude::*;
use bevy_ecs_tilemap::{
  prelude::{TilemapId, TilemapSize},
  tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
};

use crate::{input::chained_tile::ChainedTilePlaceDirection, vec2_traits::*};

use super::prelude::*;

pub mod plugin_exports {
  pub use super::place_tile;
  pub use super::PreviousPlaceAttempt;
}

pub fn spawn_tile(
  commands: &mut Commands,
  position: TilePos,
  tile_storage: &mut TileStorage,
  tilemap_entity: Entity,
  direction: ConveyorDirection,
  placed_tiles: &mut EventWriter<UpdatedTile>,
) -> Entity {
  let tile_entity = commands
    .spawn(TileBundle {
      position,
      tilemap_id: TilemapId(tilemap_entity),
      texture_index: TileTextureIndex(direction.texture_index()),
      ..Default::default()
    })
    .insert(direction)
    .id();
  tile_storage.set(&position, tile_entity);
  placed_tiles.send(UpdatedTile { pos: position });
  tile_entity
}

#[derive(Debug, Resource, Clone, Reflect, Default)]
pub struct PreviousPlaceAttempt {
  pub position: IVec2,
  pub direction: ConveyorDirection,
}

fn update_tile_direction(
  commands: &mut Commands,
  position: TilePos,
  tile_storage: &TileStorage,
  tile_direction: ConveyorDirection,
  placed_tiles: &mut EventWriter<UpdatedTile>,
) {
  let Some(tile_entity) = tile_storage.get(&position) else { return; };
  commands.entity(tile_entity).insert(tile_direction);
  placed_tiles.send(UpdatedTile { pos: position });
}

pub fn place_tile(
  mut commands: &mut Commands,
  new_tile_position: IVec2,
  tile_storage: &mut TileStorage,
  tilemap_entity: Entity,
  tilemap_size: &TilemapSize,
  previous_place_attempt: &mut PreviousPlaceAttempt,
  mut placed_tiles: &mut EventWriter<UpdatedTile>,
  place_direction: ChainedTilePlaceDirection,
  selected_tile_direction: &mut ConveyorDirection,
  chain_with_previous_tile: bool,
) {
  if chain_with_previous_tile {
    let offset = new_tile_position - previous_place_attempt.position;
    if let Some(direction_moved) = ConveyorDirection::from_ivec2(offset) {
      let changed_direction = direction_moved != previous_place_attempt.direction;
      let reversed = place_direction == ChainedTilePlaceDirection::Revesed;
      let opposite_direction_as_previous = direction_moved == previous_place_attempt.direction;

      if (changed_direction && !reversed) || (opposite_direction_as_previous && reversed) {
        let previous_tile_position = previous_place_attempt.position.to_tile_pos(&tilemap_size);
        if let Ok(previous_tile_position) = previous_tile_position {
          update_tile_direction(
            &mut commands,
            previous_tile_position,
            &tile_storage,
            direction_moved.apply_place_direction(place_direction),
            &mut placed_tiles,
          );
        }
      }
      *selected_tile_direction = direction_moved.apply_place_direction(place_direction);
    }
  }

  if let Ok(new_tile_position) = new_tile_position.to_tile_pos(&tilemap_size) {
    spawn_tile(
      commands,
      new_tile_position,
      tile_storage,
      tilemap_entity,
      *selected_tile_direction,
      &mut placed_tiles,
    );
  }
  *previous_place_attempt = PreviousPlaceAttempt {
    position: new_tile_position,
    direction: *selected_tile_direction,
  };
}
