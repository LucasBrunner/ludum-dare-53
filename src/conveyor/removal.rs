use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
  helpers::grid_traversal::GridTraversal,
  vec2_traits::{TilePosFromSigned, ToTilePos},
};

use super::{placement::TileUpdate, ConveyorTileLayer};

pub fn despawn_conveyor(
  commands: &mut Commands,
  position: TilePos,
  tile_storage: &mut TileStorage,
  removed_tiles: &mut EventWriter<TileUpdate>,
) {
  if let Some(tile_entity) = tile_storage.get(&position) {
    commands.entity(tile_entity).despawn_recursive();
    tile_storage.remove(&position);
    removed_tiles.send(TileUpdate { pos: position })
  }
}

fn despawn_conveyor_line(
  from: IVec2,
  to: IVec2,
  commands: &mut Commands,
  tilemap_size: &TilemapSize,
  tile_storage: &mut TileStorage,
  removed_tiles: &mut EventWriter<TileUpdate>,
) {
  for position in GridTraversal::new(from, to) {
    if position.min_element() >= 0
      && position.x < tilemap_size.x as i32
      && position.y < tilemap_size.y as i32
    {
      let position = position.as_uvec2().to_tile_pos();
      despawn_conveyor(commands, position, tile_storage, removed_tiles);
    }
  }
}

pub fn remove_conveyors_drag(
  mut commands: Commands,
  mut remove_conveyor_event: EventReader<RemoveConveyor>,
  mut removed_tiles: EventWriter<TileUpdate>,
  mut tilemaps: Query<(Entity, &mut TileStorage, &TilemapSize, &ConveyorTileLayer)>,
) {
  let Ok((_, mut tile_storage, tilemap_size, _)) = tilemaps.get_single_mut() else { return; };
  for event in remove_conveyor_event.iter() {
    if event.from == event.to {
      if let Ok(position) = event.from.to_tile_pos(&tilemap_size) {
        despawn_conveyor(
          &mut commands,
          position,
          &mut tile_storage,
          &mut removed_tiles,
        );
      }
      continue;
    }
    despawn_conveyor_line(
      event.from,
      event.to,
      &mut commands,
      tilemap_size,
      &mut tile_storage,
      &mut removed_tiles,
    )
  }
}

#[derive(Debug)]
pub struct RemoveConveyor {
  pub from: IVec2,
  pub to: IVec2,
}

impl RemoveConveyor {
  pub fn new(from: IVec2, to: IVec2) -> RemoveConveyor {
    RemoveConveyor { from, to }
  }

  pub fn new_single_pos(pos: IVec2) -> RemoveConveyor {
    RemoveConveyor { from: pos, to: pos }
  }
}
