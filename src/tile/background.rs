use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use super::{PlayfieldSize};

pub mod plugin_exports {
  pub use super::*;
}

#[derive(Debug, Component)]
pub struct BackgroundTileLayer;

pub fn setup_background_tilemap(
  mut commands: Commands,
  playfield_size: Res<PlayfieldSize>,
) {
  let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
  let grid_size = tile_size.into();
  let map_type = TilemapType::Square;

  let background_tilemap = commands.spawn_empty().id();
  let background_map_size = TilemapSize {
    x: playfield_size.0.x + 2,
    y: playfield_size.0.y + 2,
  };

  commands
    .entity(background_tilemap)
    .insert(TilemapBundle {
      grid_size,
      map_type,
      size: background_map_size,
      storage: TileStorage::empty(background_map_size),
      tile_size,
      transform: get_tilemap_center_transform(&background_map_size, &grid_size, &map_type, 0.0),
      ..default()
    })
    .insert(BackgroundTileLayer);
}

pub fn place_background_tiles(
  mut commands: Commands,
  mut background_layer: Query<(Entity, &mut TileStorage, &TilemapSize, &BackgroundTileLayer)>,
) {
  let Ok((background_entity, mut background_storage, background_size, _)) = background_layer.get_single_mut() else {
    error!(
      "Tilemap query for the background layer returned {} items when it only should have returned 1.", 
      background_layer.iter().len(),
    );
    return; 
  };

  for x in 0..background_size.x {
    for y in 0..background_size.y {
      let texture_index = match (
        x == 0,
        y == 0,
        x == background_size.x - 1,
        y == background_size.y - 1,
      ) {
        (true, _, _, true) => 13,
        (true, true, _, _) => 12,
        (_, true, true, _) => 11,
        (_, _, true, true) => 10,
        (_, _, true, _) => 9,
        (_, _, _, true) => 8,
        (true, _, _, _) => 7,
        (_, true, _, _) => 6,
        _ => 1,
      };
      let position = TilePos { x, y };
      let tile_entity = commands
        .spawn(TileBundle {
          position,
          tilemap_id: TilemapId(background_entity),
          texture_index: TileTextureIndex(texture_index),
          ..default()
        })
        .id();
      background_storage.set(&position, tile_entity);
    }
  }
}

pub fn insert_background_texture(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  background_layer: Query<(Entity, &BackgroundTileLayer)>,
) {
  let Ok((background_entity, _)) = background_layer.get_single() else {
    error!(
      "Tilemap query for the background layer returned {} items when it only should have returned 1.", 
      background_layer.iter().len(),
    );
    return; 
  };

  commands.entity(background_entity).insert(TilemapTexture::Single(asset_server.load("background.png")));
}