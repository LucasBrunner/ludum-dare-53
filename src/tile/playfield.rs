use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub mod plugin_exports {
  pub use super::*;
}

pub mod prelude {
  pub use super::PlayfieldSize;
  pub use super::ConveyorTileLayer;
}

#[derive(Debug, Resource, Clone)]
pub struct PlayfieldSize(pub UVec2);

#[derive(Debug, Component)]
pub struct ConveyorTileLayer;

pub fn setup_playfield(
  playfield_size: Res<PlayfieldSize>,
  mut commands: Commands,
) {
  let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
  let grid_size = tile_size.into();
  let map_type = TilemapType::Square;

  let playfield_tilemap = commands.spawn_empty().id();
  let playfield_map_size = TilemapSize {
    x: playfield_size.0.x,
    y: playfield_size.0.y,
  };

  commands
    .entity(playfield_tilemap)
    .insert(TilemapBundle {
      grid_size,
      map_type,
      size: playfield_map_size,
      storage: TileStorage::empty(playfield_map_size),
      tile_size,
      transform: get_tilemap_center_transform(&playfield_map_size, &grid_size, &map_type, 10.0),
      ..default()
    })
    .insert(ConveyorTileLayer);
}

pub fn insert_playfield_texture(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  background_layer: Query<(Entity, &ConveyorTileLayer)>,
) {
  let Ok((background_entity, _)) = background_layer.get_single() else {
    error!(
      "Tilemap query for the playfield layer returned {} items when it only should have returned 1.", 
      background_layer.iter().len(),
    );
    return; 
  };

  commands.entity(background_entity).insert(TilemapTexture::Single(asset_server.load("conveyor.png")));
}
