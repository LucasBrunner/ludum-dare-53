use bevy::{prelude::*, utils::HashSet};
use bevy_ecs_tilemap::prelude::*;

use crate::{
  camera::prelude::CursorPos,
  conveyor::{
    placement::{PlaceConveyor, PreviousMouseConveyorInput},
    ConveyorDirection,
  },
  vec2_traits::{AsIVec2, TilePosFromSigned},
};

use super::{placement::{TileUpdate, PreviouslyPlacedTile}, removal::RemoveConveyor, ConveyorTileLayer};

pub mod systems {
  pub use super::conveyor_tile_update;
  pub use super::detect_conveyor_input;
  pub use super::rotate_conveyor_placement;
  pub use super::update_conveyor_direction;
}

pub fn rotate_conveyor_placement(
  mut previous_tile: ResMut<PreviouslyPlacedTile>,
  keyboard_input: Res<Input<KeyCode>>,
) {
  if keyboard_input.just_pressed(KeyCode::R) {
    previous_tile.direction = match keyboard_input.pressed(KeyCode::LShift) {
        true => previous_tile.direction.rotate_counterclockwise(),
        false => previous_tile.direction.rotate_clockwise(),
    }
  }
}

pub fn detect_conveyor_input(
  cursor_pos: ResMut<CursorPos>,
  mouse_click: ResMut<Input<MouseButton>>,
  mut previous_tile: ResMut<PreviouslyPlacedTile>,
  mut previous_mouse_conveyor_input: ResMut<PreviousMouseConveyorInput>,
  mut place_tile_event: EventWriter<PlaceConveyor>,
  mut remove_tile_event: EventWriter<RemoveConveyor>,
  mut tilemaps: Query<(&TilemapGridSize, &Transform, &ConveyorTileLayer)>,
) {
  let Ok((grid_size, map_transform, _)) = tilemaps.get_single_mut() else { return; };
  let cursor_pos = cursor_pos.to_map_pos(map_transform) / Vec2::new(grid_size.x, grid_size.y);
  let cursor_pos = (cursor_pos + Vec2::new(0.5, 0.5)).as_ivec2();

  match (
    mouse_click.pressed(MouseButton::Left),
    mouse_click.pressed(MouseButton::Right),
  ) {
    (true, false) => {
      match previous_mouse_conveyor_input.add_conveyor {
        Some(previous_pos) => {
          if cursor_pos != previous_pos {
            place_tile_event.send(PlaceConveyor::new(previous_pos, cursor_pos));
          }
        }
        None => place_tile_event.send(PlaceConveyor::new_single_pos(cursor_pos)),
      }
      previous_mouse_conveyor_input.add_conveyor = Some(cursor_pos);
      previous_mouse_conveyor_input.remove_conveyor = None;
    }
    (false, true) => {
      match previous_mouse_conveyor_input.remove_conveyor {
        Some(previous_pos) => {
          if cursor_pos != previous_pos {
            remove_tile_event.send(RemoveConveyor::new(previous_pos, cursor_pos));
          }
        }
        None => remove_tile_event.send(RemoveConveyor::new_single_pos(cursor_pos)),
      }
      previous_mouse_conveyor_input.add_conveyor = None;
      previous_mouse_conveyor_input.remove_conveyor = Some(cursor_pos);
    }
    _ => {
      previous_mouse_conveyor_input.add_conveyor = None;
      previous_mouse_conveyor_input.remove_conveyor = None;
      previous_tile.tile_pos = None;
    }
  }
}

#[derive(Debug)]
pub struct ChangeConveyorDirection {
  pub position: TilePos,
  pub direction: ConveyorDirection,
}

pub fn update_conveyor_direction(
  mut change_conveyor_detection: EventReader<ChangeConveyorDirection>,
  mut conveyor_tile_updates: EventWriter<TileUpdate>,
  mut tiles: Query<(Entity, &mut ConveyorDirection, &TilePos)>,
  mut tilemap: Query<(&TileStorage, &ConveyorTileLayer)>,
) {
  let Ok((tile_storage, _)) = tilemap.get_single_mut() else { return; };

  for change_conveyor_direction in change_conveyor_detection.iter() {
    let Some(conveyor_entity) = tile_storage.get(&change_conveyor_direction.position) else { continue; };
    let Ok(mut tile) = tiles.get_mut(conveyor_entity) else { continue; };
    conveyor_tile_updates.send(TileUpdate { pos: *tile.2 });
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
  mut conveyor_tile_updates: EventReader<TileUpdate>,
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
