use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use super::UpdatedTile;

pub mod plugin_exports {
  pub use super::despawn_conveyor;
}

pub fn despawn_conveyor(
  commands: &mut Commands,
  position: TilePos,
  tile_storage: &mut TileStorage,
  removed_tiles: &mut EventWriter<UpdatedTile>,
) {
  if let Some(tile_entity) = tile_storage.get(&position) {
    commands.entity(tile_entity).despawn_recursive();
    tile_storage.remove(&position);
    removed_tiles.send(UpdatedTile { pos: position })
  }
}
