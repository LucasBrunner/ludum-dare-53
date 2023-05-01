use bevy::prelude::*;
use bevy_ecs_tilemap::{
  prelude::{TilemapId, TilemapSize},
  tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
};

use crate::{helpers::grid_traversal::GridTraversal, vec2_traits::*, ConveyorDirection};

use super::{update::ChangeConveyorDirection, ConveyorTileLayer};

pub mod prelude {
  pub use super::PlaceConveyor;
  pub use super::PreviousMouseConveyorInput;
  pub use super::PreviouslyPlacedTile;
  pub use super::TileUpdate;
}

pub mod systems {
  pub use super::place_conveyors_drag;
}

#[derive(Debug, Clone)]
pub struct TileUpdate {
  pub pos: TilePos,
}

#[derive(Debug, Resource)]
pub struct PreviousMouseConveyorInput {
  pub add_conveyor: Option<IVec2>,
  pub remove_conveyor: Option<IVec2>,
}

impl FromWorld for PreviousMouseConveyorInput {
  fn from_world(_world: &mut World) -> Self {
    PreviousMouseConveyorInput {
      add_conveyor: None,
      remove_conveyor: None,
    }
  }
}

#[derive(Debug, Resource, Clone, Reflect, Default)]
pub struct PreviouslyPlacedTile {
  pub tile_pos: Option<TilePos>,
  pub direction: ConveyorDirection,
}

#[derive(Debug)]
pub struct PlaceConveyor {
  pub from: IVec2,
  pub to: IVec2,
}

impl PlaceConveyor {
  pub fn new(from: IVec2, to: IVec2) -> PlaceConveyor {
    PlaceConveyor { from, to }
  }

  pub fn new_single_pos(pos: IVec2) -> PlaceConveyor {
    PlaceConveyor { from: pos, to: pos }
  }
}

pub fn spawn_tile(
  commands: &mut Commands,
  position: TilePos,
  tile_storage: &mut TileStorage,
  tilemap_entity: Entity,
  direction: ConveyorDirection,
  placed_tiles: &mut EventWriter<TileUpdate>,
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
  placed_tiles.send(TileUpdate { pos: position });
  tile_entity
}

pub fn place_tile(
  commands: &mut Commands,
  position: IVec2,
  tile_storage: &mut TileStorage,
  tilemap_entity: Entity,
  tilemap_size: &TilemapSize,
  previous_tile: &mut PreviouslyPlacedTile,
  change_conveyor_detection: &mut EventWriter<ChangeConveyorDirection>,
  mut placed_tiles: &mut EventWriter<TileUpdate>,
  reverse_direction: bool,
) {
  let mut input_direction = previous_tile.direction;
  if let Some(previous_tile_pos) = previous_tile.tile_pos {
    let diff = position - previous_tile_pos.as_ivec2();
    let direction = match (diff.x, diff.y) {
      (0, 1) => Some(ConveyorDirection::North),
      (1, 0) => Some(ConveyorDirection::East),
      (0, -1) => Some(ConveyorDirection::South),
      (-1, 0) => Some(ConveyorDirection::West),
      _ => None,
    };
    if let Some(direction) = direction {
      if input_direction == direction || !reverse_direction {
        change_conveyor_detection.send(ChangeConveyorDirection {
          position: previous_tile_pos,
          direction: direction.reverse(reverse_direction),
        });
      }
      input_direction = direction;
    }
  }

  if position.min_element() >= 0
    && position.x < tilemap_size.x as i32
    && position.y < tilemap_size.y as i32
  {
    let position = position.as_uvec2().to_tile_pos();
    spawn_tile(
      commands,
      position,
      tile_storage,
      tilemap_entity,
      input_direction.reverse(reverse_direction),
      &mut placed_tiles,
    );
    previous_tile.tile_pos = Some(position);
    previous_tile.direction = input_direction;
  }
}

pub fn place_tile_line(
  mut commands: &mut Commands,
  to: IVec2,
  from: IVec2,
  mut change_conveyor_detection: &mut EventWriter<ChangeConveyorDirection>,
  mut tile_storage: &mut TileStorage,
  tilemap_entity: Entity,
  tilemap_size: &TilemapSize,
  mut previous_tile: &mut PreviouslyPlacedTile,
  mut placed_tiles: &mut EventWriter<TileUpdate>,
  reverse_direction: bool,
) {
  for position in GridTraversal::new(from, to) {
    place_tile(
      &mut commands,
      position,
      &mut tile_storage,
      tilemap_entity,
      &tilemap_size,
      &mut previous_tile,
      &mut change_conveyor_detection,
      &mut placed_tiles,
      reverse_direction,
    );
  }
}

pub fn place_conveyors_drag(
  mut commands: Commands,
  mut place_tile_event: EventReader<PlaceConveyor>,
  mut change_conveyor_detection: EventWriter<ChangeConveyorDirection>,
  mut placed_tiles: EventWriter<TileUpdate>,
  mut tilemaps: Query<(Entity, &mut TileStorage, &TilemapSize, &ConveyorTileLayer)>,
  mut previous_tile: ResMut<PreviouslyPlacedTile>,
  keyboard_input: Res<Input<KeyCode>>,
) {
  let Ok((tilemap_entity, mut tile_storage, tilemap_size, _)) = tilemaps.get_single_mut() else { return; };
  for place_tile_event in place_tile_event.iter() {
    let (to, from) = (place_tile_event.to, place_tile_event.from);

    if to == from {
      if from.min_element() >= 0 && from.x < tilemap_size.x as i32 && from.y < tilemap_size.y as i32
      {
        place_tile(
          &mut commands,
          from,
          &mut tile_storage,
          tilemap_entity,
          &tilemap_size,
          &mut previous_tile,
          &mut change_conveyor_detection,
          &mut placed_tiles,
          false,
        );
      }
      continue;
    } else {
      place_tile_line(
        &mut commands,
        to,
        from,
        &mut change_conveyor_detection,
        &mut tile_storage,
        tilemap_entity,
        &tilemap_size,
        &mut previous_tile,
        &mut placed_tiles,
        keyboard_input.pressed(KeyCode::LShift)
      )
    }
  }
}
