use bevy::{prelude::*, utils::HashSet};
use bevy_ecs_tilemap::prelude::*;

use crate::vec2_traits::{AsIVec2, TilePosFromSigned};

use super::{prelude::*, ConveyorTileLayer};

pub mod systems {
  pub use super::conveyor_tile_update_graphics;
}

#[derive(Debug)]
enum ConveyorNeighbor {
  Output,
  Input,
  None,
}

trait ToTileTextureIndex<T, U> {
  fn get_tile_texture_index(&self, input: T) -> U;
}

mod ttti {
  use super::ConveyorNeighbor::*;
  use super::*;
  impl ToTileTextureIndex<&[ConveyorNeighbor; 3], TileTextureIndex> for ConveyorDirection {
    fn get_tile_texture_index(&self, input: &[ConveyorNeighbor; 3]) -> TileTextureIndex {
      let base = match input {
        [Input, Input, Input] => Some(17),
        [Input, Input, None] => Some(25),
        [Input, None, Input] => Some(13),
        [Input, None, None] => Some(9),
        [None, Input, Input] => Some(21),
        [None, Input, None] => Some(1),
        [None, None, Input] => Some(5),
        [None, None, None] => Some(1),
        _ => Option::None,
      };

      let Some(mut base) = base else {
        return TileTextureIndex(0);
      };

      base += match self {
        ConveyorDirection::North => 0,
        ConveyorDirection::East => 1,
        ConveyorDirection::South => 2,
        ConveyorDirection::West => 3,
      };
      return TileTextureIndex(base);
    }
  }
}

pub fn conveyor_tile_update_graphics(
  mut conveyor_tile_updates: EventReader<UpdatedTile>,
  tilemaps: Query<(&mut TileStorage, &TilemapSize, &ConveyorTileLayer)>,
  mut tiles: Query<(Entity, &mut TileTextureIndex, &ConveyorDirection)>,
) {
  // get the position of all conveyors which need updating
  let conveyor_tile_updates: Vec<_> = conveyor_tile_updates.into_iter().collect();
  for (tile_store, tilemap_size, _) in tilemaps.iter() {
    let conveyors_to_update: HashSet<_> = conveyor_tile_updates
      .iter()
      .map(|update| update.pos)
      .collect();
    let secondary_conveyors = conveyors_to_update
      .iter()
      .map(|pos| {
        ConveyorDirection::DIRECTION_VALUES
          .iter()
          .filter_map(|offset| {
            let pos = pos.as_ivec2() + *offset;
            if let Ok(tile_pos) = pos.to_tile_pos(&tilemap_size) {
              Some(tile_pos)
            } else {
              None
            }
          })
      })
      .fold(HashSet::new(), |mut acc, poses| {
        acc.extend(poses);
        acc
      });

    // fetch the entities and calculate the correct texture for all conveyors
    let texture_updates: Vec<_> = conveyors_to_update
      .union(&secondary_conveyors)
      .filter_map(|tile_pos| {
        let Some(tile_entity) = tile_store.get(tile_pos) else {
          return None;
        };
        let Ok((_, _, conveyor_direction)) = tiles.get(tile_entity) else {
          return None;
        };

        let side_states: [ConveyorNeighbor; 3] = conveyor_direction.neighbors_to_check_for_connections()
        .iter()
        .map(|direction| {
          let Ok(tile_pos) = (direction.offset() + tile_pos.as_ivec2()).to_tile_pos(&tilemap_size) else {
            return ConveyorNeighbor::None;
          };
          let Some(tile) = tile_store.get(&tile_pos) else {
            return ConveyorNeighbor::None;
          };
          let Ok((_, _, neighbor_direction)) =  tiles.get(tile) else {
            return ConveyorNeighbor::None;
          };
          match *neighbor_direction == direction.opposite() {
            true => ConveyorNeighbor::Input,
            false => ConveyorNeighbor::None,
          }
        }).collect::<Vec<ConveyorNeighbor>>().try_into().unwrap();

        Some((tile_entity, conveyor_direction.get_tile_texture_index(&side_states)))
      }).collect();

    // apply each conveyor's texture
    for (entity, texture) in texture_updates {
      let Ok((_, mut tile_texture, _)) = tiles.get_mut(entity) else {
        continue;
      };
      *tile_texture = texture;
    }
  }
}
