use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
  camera::prelude::CursorPos,
  conveyor::{
    placement::{PlaceTile, PreviousTilePlaceAttempt},
    ConveyorDirection,
  },
};

pub mod systems {
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
  mut tiles: Query<(Entity, &mut TileTextureIndex, &TilePos)>,
) {
  if change_conveyor_detection.len() != 0 {
    println!("changes to make: {}", change_conveyor_detection.len());
  }

  for change_conveyor_direction in change_conveyor_detection.iter() {
    let Ok(mut tile) = tiles.get_mut(change_conveyor_direction.entity) else {
      println!("could not find tile");
      continue;
    };
    println!(
      "Updating tile at pos {:?} to direction {}",
      tile.2, change_conveyor_direction.direction,
    );
    *tile.1 = TileTextureIndex(change_conveyor_direction.direction.texture_index());
  }
}
