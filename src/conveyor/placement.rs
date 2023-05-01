use bevy::prelude::*;
use bevy_ecs_tilemap::{
  prelude::{TilemapId, TilemapSize},
  tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
};

use crate::{input::chained_tile::ChainedTilePlaceDirection, vec2_traits::*, ConveyorDirection};

pub mod prelude {
  pub use super::UpdatedTile;
}

pub mod plugin_exports {
  pub use super::PreviousPlaceTileAttempt;
}

#[derive(Debug, Clone)]
pub struct UpdatedTile {
  pub pos: TilePos,
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
pub struct PreviousPlaceTileAttempt {
  pub position: Option<TilePos>,
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

fn place_tile(
  mut commands: &mut Commands,
  position: IVec2,
  tile_storage: &mut TileStorage,
  tilemap_entity: Entity,
  tilemap_size: &TilemapSize,
  previous_tile: &mut PreviousPlaceTileAttempt,
  mut placed_tiles: &mut EventWriter<UpdatedTile>,
  place_direction: ChainedTilePlaceDirection,
  mut selected_tile_direction: ConveyorDirection,
  chain: bool,
) {
  if chain {
    if let Some(previous_tile_pos) = previous_tile.position {
      if let Some(direction_moved) =
        ConveyorDirection::from_ivec2(position - previous_tile_pos.as_ivec2())
      {
        if (place_direction == ChainedTilePlaceDirection::Revesed)
          == (previous_tile.direction == direction_moved)
        {
          update_tile_direction(
            &mut commands,
            previous_tile_pos,
            &tile_storage,
            direction_moved.apply_place_direction(place_direction),
            &mut placed_tiles,
          );
        }
        selected_tile_direction = direction_moved;
      }
    }
  }

  if let Ok(position) = position.to_tile_pos(&tilemap_size) {
    spawn_tile(
      commands,
      position,
      tile_storage,
      tilemap_entity,
      selected_tile_direction.apply_place_direction(place_direction),
      &mut placed_tiles,
    );
    *previous_tile = PreviousPlaceTileAttempt {
      position: Some(position),
      direction: selected_tile_direction.apply_place_direction(place_direction),
    };
  }
}
