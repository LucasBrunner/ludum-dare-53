use bevy::prelude::*;
use bevy_ecs_tilemap::{
  prelude::{TilemapId, TilemapSize},
  tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
};

use crate::{helpers::grid_traversal::GridTraversal, vec2_traits::*, ConveyorDirection, input::{chained_tile::{prelude::*, ChainedTilePlaceDirection}, tile_rotation::SelectedTileDirection,},};

use super::{ConveyorTileLayer, removal::despawn_conveyor};

pub mod prelude {
  pub use super::UpdatedTile;
}

pub mod plugin_exports {
  pub use super::catch_chained_tile_change_events;
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
      if let Some(direction_moved) = ConveyorDirection::from_ivec2(position - previous_tile_pos.as_ivec2()) {
        if (place_direction == ChainedTilePlaceDirection::Revesed) == (previous_tile.direction == direction_moved) {
          update_tile_direction(&mut commands, previous_tile_pos, &tile_storage, direction_moved.apply_place_direction(place_direction), &mut placed_tiles);
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
