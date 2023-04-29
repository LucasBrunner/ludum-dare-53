use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_tilemap::prelude::*;

use crate::{
  camera::prelude::CursorPos,
  conveyor::{
    placement::{PlaceTile, PreviousTilePlaceAttempt},
    ConveyorDirection,
  },
  vec2_traits::{AsIVec2, ToTilePos},
};

use super::placement::UpdateTile;

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
  mut tilemaps: Query<(&TilemapGridSize, &Transform)>,
) {
  let Ok((grid_size, map_transform)) = tilemaps.get_single_mut() else { return; };
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
      entity: tile.0,
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
      match self {
        ConveyorDirection::North => match input {
          [Input, Input, Input] => TileTextureIndex(17),
          [Input, Input, None] => TileTextureIndex(25),
          [Input, None, Input] => TileTextureIndex(13),
          [Input, None, None] => TileTextureIndex(9),
          [None, Input, Input] => TileTextureIndex(21),
          [None, Input, None] => TileTextureIndex(1),
          [None, None, Input] => TileTextureIndex(5),
          [None, None, None] => TileTextureIndex(1),
          _ => TileTextureIndex(0),
        },
        ConveyorDirection::East => match input {
          [Input, Input, Input] => TileTextureIndex(17 + 1),
          [Input, Input, None] => TileTextureIndex(25 + 1),
          [Input, None, Input] => TileTextureIndex(13 + 1),
          [Input, None, None] => TileTextureIndex(9 + 1),
          [None, Input, Input] => TileTextureIndex(21 + 1),
          [None, Input, None] => TileTextureIndex(1 + 1),
          [None, None, Input] => TileTextureIndex(5 + 1),
          [None, None, None] => TileTextureIndex(1 + 1),
          _ => TileTextureIndex(0),
        },
        ConveyorDirection::South => match input {
          [Input, Input, Input] => TileTextureIndex(17 + 2),
          [Input, Input, None] => TileTextureIndex(25 + 2),
          [Input, None, Input] => TileTextureIndex(13 + 2),
          [Input, None, None] => TileTextureIndex(9 + 2),
          [None, Input, Input] => TileTextureIndex(21 + 2),
          [None, Input, None] => TileTextureIndex(1 + 2),
          [None, None, Input] => TileTextureIndex(5 + 2),
          [None, None, None] => TileTextureIndex(1 + 2),
          _ => TileTextureIndex(0),
        },
        ConveyorDirection::West => match input {
          [Input, Input, Input] => TileTextureIndex(17 + 3),
          [Input, Input, None] => TileTextureIndex(25 + 3),
          [Input, None, Input] => TileTextureIndex(13 + 3),
          [Input, None, None] => TileTextureIndex(9 + 3),
          [None, Input, Input] => TileTextureIndex(21 + 3),
          [None, Input, None] => TileTextureIndex(1 + 3),
          [None, None, Input] => TileTextureIndex(5 + 3),
          [None, None, None] => TileTextureIndex(1 + 3),
          _ => TileTextureIndex(0),
        },
      }
    }
  }
}

fn copy_tiles(
  tiles: &Query<(Entity, &mut TileTextureIndex, &ConveyorDirection)>,
  conveyor_tile_updates: &Vec<&UpdateTile>,
) -> HashMap<Entity, (TilePos, ConveyorDirection)> {
  conveyor_tile_updates
    .iter()
    .filter_map(|update| {
      let tile = tiles.get(update.entity);
      match tile {
        Ok(tile) => Some((update.entity, (update.pos.clone(), *tile.2))),
        Err(_) => None,
      }
    })
    .collect()
}

pub fn conveyor_tile_update(
  mut conveyor_tile_updates: EventReader<UpdateTile>,
  tilemaps: Query<&mut TileStorage>,
  mut tiles: Query<(Entity, &mut TileTextureIndex, &ConveyorDirection)>,
) {
  let conveyor_tile_updates: Vec<_> = conveyor_tile_updates.into_iter().collect();
  for tile_store in tilemaps.iter() {
    let mut tile_copies = copy_tiles(&tiles, &conveyor_tile_updates);

    let secondary_tile_copies: HashMap<_, _> = tile_copies
      .iter()
      .map(|(_, (update_tile_pos, _))| {
        ConveyorDirection::DIRECTION_VALUES
          .iter()
          .filter_map(|offset| {
            let pos = (update_tile_pos.as_ivec2() + *offset)
              .as_uvec2()
              .to_tile_pos();
            let Some(entity) = tile_store.get(&pos) else { return None; };
            let Ok((entity, _, conveyor_direction)) = tiles.get(entity) else { return None; };
            Some((entity, (pos, *conveyor_direction)))
          })
      })
      .fold(HashMap::new(), |mut acc, secodondary_tiles| {
        acc.extend(secodondary_tiles);
        acc
      });

    tile_copies.extend(secondary_tile_copies.iter());
    let directions: Vec<_> = tile_copies.iter().map(|(conveyor_entity, (update_tile_pos, conveyor_direction))| {
      println!("Updating tile at {} with a direction of {:?}", update_tile_pos.as_ivec2(), conveyor_direction);
      let side_states: [ConveyorNeighbor; 3] = conveyor_direction.neighbors_to_check_for_connections().iter().map(|direction| {
        if direction == conveyor_direction {
          return ConveyorNeighbor::Output;
        }
        let Some(tile) = tile_store.get(&(direction.offset() + update_tile_pos.as_ivec2()).as_uvec2().to_tile_pos()) else {
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
      (side_states, conveyor_direction, conveyor_entity)
    }).collect();

    let textures = directions
      .iter()
      .map(|(side_states, direction, conveyor_entity)| {
        (
          direction.get_tile_texture_index(side_states),
          conveyor_entity,
        )
      });

    for (texture, entity) in textures {
      let Ok((_, mut tile_texture, _)) = tiles.get_mut(**entity) else {
        continue;
      };
      *tile_texture = texture;
    }
  }
}
