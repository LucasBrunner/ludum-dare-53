pub fn catch_chained_tile_change_events(
  mut commands: Commands,
  mut place_tile_events: EventReader<ChainedTileChangeEvent>,
  mut placed_tiles: EventWriter<UpdatedTile>,
  mut tilemap: Query<(Entity, &mut TileStorage, &TilemapSize, &ConveyorTileLayer)>,
  mut previous_tile_attempt: ResMut<PreviousPlaceTileAttempt>,
  selected_tile_rotation: Res<SelectedTileDirection>,
) {
  let Ok((tilemap_entity, mut tile_storage, tilemap_size, _)) = tilemap.get_single_mut() else { 
    error!(
      "Tilemap query for the conveyor layer returned {} items when it only should have returned 1.", 
      tilemap.iter().len(),
    );
    return; 
  };

  for place_tile_event in place_tile_events.iter() {   
    let positions = match place_tile_event.position {
      ChainedTileChangePosition::Single(position) => GridTraversal::new(position, position).add_iterations(1),
      ChainedTileChangePosition::StraightLine { start, end } => GridTraversal::new(start, end),
    };

    for position in positions {
      match place_tile_event.change_type {
        crate::input::chained_tile::ChainedTileChangeType::Put { tile_type: _tile_type, chain, direction } => {
          place_tile(&mut commands, position, &mut tile_storage, tilemap_entity, tilemap_size, &mut previous_tile_attempt, &mut placed_tiles, direction, selected_tile_rotation.direction, chain);
        },
        crate::input::chained_tile::ChainedTileChangeType::Delete => {
          if let Ok(position) = position.to_tile_pos(&tilemap_size) {
            despawn_conveyor(&mut commands, position, &mut tile_storage, &mut placed_tiles);
          }
        },
      }
    }
  }
}
