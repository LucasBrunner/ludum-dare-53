use bevy::{prelude::*, utils::HashSet};
use bevy_ecs_tilemap::prelude::*;

use crate::{
  camera::prelude::CursorPos,
  conveyor::{
    placement::{PlaceTile, PreviousTilePlaceAttempt},
    ConveyorDirection,
  },
  vec2_traits::{AsIVec2, ToTilePos},
};

use super::{placement::UpdateTile, ConveyorTileLayer};

pub mod systems {
  pub use super::conveyor_tile_update;
  pub use super::detect_tile_place;
  pub use super::update_tile_direction;
}

pub fn detect_tile_place(
  cursor_pos: ResMut<CursorPos>,
  mouse_click: ResMut<Input<MouseButton>>,
  mut previous_tile_place_position: ResMut<PreviousTilePlaceAttempt>,
  mut place_tile_event: EventWriter<PlaceTile>,
  mut tilemaps: Query<(&TilemapGridSize, &Transform, &ConveyorTileLayer)>,
) {
  let Ok((grid_size, map_transform, _)) = tilemaps.get_single_mut() else { return; };
  let tile_pos = (cursor_pos.to_map_pos(map_transform) / Vec2::new(grid_size.x, grid_size.y)
    + Vec2::new(0.5, 0.5))
  .as_ivec2();
  if mouse_click.pressed(MouseButton::Left) {
    match previous_tile_place_position.0 {
      Some(previous_tile_place) => {
        if tile_pos != previous_tile_place {
          place_tile_event.send(PlaceTile::new(previous_tile_place, tile_pos));
        }
      }
      None => place_tile_event.send(PlaceTile::new_single_pos(tile_pos)),
    }
    previous_tile_place_position.0 = Some(tile_pos);
  } else {
    previous_tile_place_position.0 = None;
  }
}

#[derive(Debug)]
pub struct ChangeConveyorDirection {
  pub entity: Entity,
  pub direction: ConveyorDirection,
}

pub fn update_tile_direction(
  mut change_conveyor_detection: EventReader<ChangeConveyorDirection>,
  mut conveyor_tile_updates: EventWriter<UpdateTile>,
  mut tiles: Query<(Entity, &mut ConveyorDirection, &TilePos)>,
) {
  if change_conveyor_detection.len() != 0 {}

  for change_conveyor_direction in change_conveyor_detection.iter() {
    let Ok(mut tile) = tiles.get_mut(change_conveyor_direction.entity) else {
      continue;
    };
    conveyor_tile_updates.send(UpdateTile {
      pos: *tile.2,
      // entity: tile.0,
    });
    *tile.1 = change_conveyor_direction.direction;
  }
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

pub fn conveyor_tile_update(
  mut conveyor_tile_updates: EventReader<UpdateTile>,
  tilemaps: Query<(&mut TileStorage, &TilemapSize, &ConveyorTileLayer)>,
  mut tiles: Query<(Entity, &mut TileTextureIndex, &ConveyorDirection)>,
) {
  // get all conveyors which need updating
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
            if pos.min_element() < 0 {
              return None;
            }
            let pos = pos.as_uvec2();
            if pos.x >= tilemap_size.x || pos.y >= tilemap_size.y {
              None
            } else {
              Some(pos.to_tile_pos())
            }
          })
      })
      .fold(HashSet::new(), |mut acc, poses| {
        acc.extend(poses);
        acc
      });

    // calculate the correct texture for all conveyors
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
          let Some(tile) = tile_store.get(&(direction.offset() + tile_pos.as_ivec2()).as_uvec2().to_tile_pos()) else {
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
