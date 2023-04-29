use bevy::prelude::*;
use bevy_ecs_tilemap::{
  prelude::{TilemapId, TilemapSize},
  tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
};

use crate::{vec2_traits::*, ChangeConveyorDirection, ConveyorDirection};

#[derive(Debug, Resource, Clone)]
pub struct PreviouslyPlacedTile {
  tile_pos: TilePos,
  entity: Entity,
  direction: ConveyorDirection,
}

#[derive(Debug)]
pub struct PlaceTile {
  pub from: IVec2,
  pub to: IVec2,
}

impl PlaceTile {
  pub fn new(from: IVec2, to: IVec2) -> PlaceTile {
    PlaceTile { from, to }
  }

  pub fn new_single_pos(pos: IVec2) -> PlaceTile {
    PlaceTile { from: pos, to: pos }
  }
}

pub fn spawn_tile(
  commands: &mut Commands,
  position: TilePos,
  tile_storage: &mut TileStorage,
  tilemap_entity: Entity,
  texture_index: TileTextureIndex,
) -> Entity {
  let tile_entity = commands
    .spawn(TileBundle {
      position,
      tilemap_id: TilemapId(tilemap_entity),
      texture_index,
      ..Default::default()
    })
    .id();
  tile_storage.set(&position, tile_entity);
  tile_entity
}

trait OptionalResource<T> {
  fn resource_as_option(&self) -> Option<T>;
}

impl OptionalResource<PreviouslyPlacedTile> for Option<Res<'_, PreviouslyPlacedTile>> {
  fn resource_as_option(&self) -> Option<PreviouslyPlacedTile> {
    match self {
      Some(res) => Some(PreviouslyPlacedTile {
        tile_pos: res.tile_pos,
        entity: res.entity,
        direction: res.direction,
      }),
      None => None,
    }
  }
}

pub fn place_tile(
  commands: &mut Commands,
  position: IVec2,
  tile_storage: &mut TileStorage,
  tilemap_entity: Entity,
  tilemap_size: &TilemapSize,
  previous_tile: &Option<PreviouslyPlacedTile>,
  change_conveyor_detection: &mut EventWriter<ChangeConveyorDirection>,
) -> Option<PreviouslyPlacedTile> {
  let mut input_direction = None;
  if let Some(previous_tile) = previous_tile {
    let diff = position - previous_tile.tile_pos.as_ivec2();
    print!(", diff: {}", diff);
    let direction = match (diff.x, diff.y) {
      (0, 1) => Some(ConveyorDirection::North),
      (1, 0) => Some(ConveyorDirection::East),
      (0, -1) => Some(ConveyorDirection::South),
      (-1, 0) => Some(ConveyorDirection::West),
      _ => None,
    };
    if let Some(direction) = direction {
      print!("\nchange direction of tile");
      change_conveyor_detection.send(ChangeConveyorDirection {
        entity: previous_tile.entity,
        direction,
      });
      input_direction = Some(direction);
    } else {
      input_direction = Some(previous_tile.direction);
    }
  }

  if position.min_element() >= 0
    && position.x < tilemap_size.x as i32
    && position.y < tilemap_size.y as i32
  {
    let direction = input_direction.unwrap_or(ConveyorDirection::North);
    let tile_pos = position.as_uvec2().to_tile_pos();
    let tile_entity = spawn_tile(
      commands,
      tile_pos,
      tile_storage,
      tilemap_entity,
      TileTextureIndex(direction.texture_index()),
    );
    Some(PreviouslyPlacedTile {
      tile_pos,
      entity: tile_entity,
      direction,
    })
  } else {
    None
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
  previous_tile: &mut Option<PreviouslyPlacedTile>,
) {
  print!("\nPlacing tiles from {} to {}", from, to);
  let mut current_offset = IVec2::ZERO;
  let direction = IVec2::new((to.x - from.x).signum(), (to.y - from.y).signum());
  let vector = IVec2::new(to.x - from.x, to.y - from.y);
  let vector_f32 = vector.as_vec2();
  let mut x_delta_max = vector_f32.x / vector_f32.y;
  let mut y_delta_max = vector_f32.y / vector_f32.x;
  let x_delta = (vector_f32 / vector_f32.x).length();
  let y_delta = (vector_f32 / vector_f32.y).length();

  for _ in 0..(vector.x.abs() + vector.y.abs()) {
    if (x_delta_max < y_delta_max && vector.x != 0) || vector.y == 0 {
      x_delta_max += x_delta;
      current_offset.x += direction.x;
    } else {
      y_delta_max += y_delta;
      current_offset.y += direction.y;
    }
    let next_pos = from + current_offset;
    print!("\n{:?}", previous_tile);

    *previous_tile = place_tile(
      &mut commands,
      next_pos,
      &mut tile_storage,
      tilemap_entity,
      &tilemap_size,
      previous_tile,
      &mut change_conveyor_detection,
    );
  }

  println!();
  match previous_tile.as_ref() {
    Some(previous_tile) => commands.insert_resource(previous_tile.clone()),
    None => commands.remove_resource::<PreviouslyPlacedTile>(),
  }
}

pub fn place_tiles_drag(
  mut commands: Commands,
  mut place_tile_event: EventReader<PlaceTile>,
  mut change_conveyor_detection: EventWriter<ChangeConveyorDirection>,
  mut tilemaps: Query<(Entity, &mut TileStorage, &TilemapSize)>,
  previous_tile: Option<Res<PreviouslyPlacedTile>>,
) {
  let Ok((tilemap_entity, mut tile_storage, tilemap_size)) = tilemaps.get_single_mut() else { return; };
  let mut previous_tile = previous_tile.resource_as_option();
  for place_tile_event in place_tile_event.iter() {
    let (to, from) = (place_tile_event.to, place_tile_event.from);

    if to == from {
      if from.min_element() >= 0 && from.x < tilemap_size.x as i32 && from.y < tilemap_size.y as i32
      {
        previous_tile = place_tile(
          &mut commands,
          from,
          &mut tile_storage,
          tilemap_entity,
          &tilemap_size,
          &previous_tile,
          &mut change_conveyor_detection,
        );

        match previous_tile.as_ref() {
          Some(previous_tile) => commands.insert_resource(previous_tile.clone()),
          None => commands.remove_resource::<PreviouslyPlacedTile>(),
        }
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
        tilemap_size,
        &mut previous_tile,
      )
    }
  }
}
