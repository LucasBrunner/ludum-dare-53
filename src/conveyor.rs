use std::fmt::Display;

use bevy::prelude::{apply_system_buffers, Component, IVec2, IntoSystemConfigs, Plugin};

use crate::camera::prelude::update_cursor_pos;

use self::placement::systems::*;
use self::update::systems::*;

pub mod placement;
pub mod update;

pub mod prelude {
  pub use super::ConveyorBuildPlugin;
  pub use super::ConveyorDirection;
}

pub struct ConveyorBuildPlugin;

impl Plugin for ConveyorBuildPlugin {
  fn build(&self, app: &mut bevy::prelude::App) {
    app
      .init_resource::<placement::PreviousTilePlaceAttempt>()
      .add_event::<placement::UpdateTile>()
      .add_event::<placement::PlaceTile>()
      .add_event::<update::ChangeConveyorDirection>()
      .add_systems(
        (
          detect_tile_place,
          place_tiles_drag,
          apply_system_buffers,
          update_tile_direction,
          conveyor_tile_update,
        )
          .after(update_cursor_pos)
          .chain(),
      );
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Component)]
pub enum ConveyorDirection {
  North,
  South,
  East,
  West,
}

impl Display for ConveyorDirection {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.name())
  }
}

impl ConveyorDirection {
  pub fn texture_index(&self) -> u32 {
    match self {
      ConveyorDirection::North => 1,
      ConveyorDirection::East => 2,
      ConveyorDirection::South => 3,
      ConveyorDirection::West => 4,
    }
  }

  pub fn opposite(&self) -> ConveyorDirection {
    match self {
      ConveyorDirection::North => ConveyorDirection::South,
      ConveyorDirection::East => ConveyorDirection::West,
      ConveyorDirection::South => ConveyorDirection::North,
      ConveyorDirection::West => ConveyorDirection::East,
    }
  }

  pub fn name(&self) -> &'static str {
    match self {
      ConveyorDirection::North => "North",
      ConveyorDirection::East => "East",
      ConveyorDirection::South => "South",
      ConveyorDirection::West => "West",
    }
  }

  pub const DIRECTION_VALUES: [IVec2; 4] = [IVec2::Y, IVec2::X, IVec2::NEG_Y, IVec2::NEG_X];

  pub fn neighbor_offsets(&self) -> [IVec2; 3] {
    match self {
      ConveyorDirection::North => [IVec2::X, IVec2::NEG_Y, IVec2::NEG_X],
      ConveyorDirection::East => [IVec2::NEG_Y, IVec2::NEG_X, IVec2::Y],
      ConveyorDirection::South => [IVec2::NEG_X, IVec2::Y, IVec2::X],
      ConveyorDirection::West => [IVec2::Y, IVec2::X, IVec2::NEG_Y],
    }
  }

  pub fn offset(&self) -> IVec2 {
    match self {
      ConveyorDirection::North => IVec2::Y,
      ConveyorDirection::East => IVec2::X,
      ConveyorDirection::South => IVec2::NEG_Y,
      ConveyorDirection::West => IVec2::NEG_X,
    }
  }

  pub fn neighbors_to_check_for_connections(&self) -> [ConveyorDirection; 3] {
    match self {
      ConveyorDirection::North => [ConveyorDirection::East, ConveyorDirection::South, ConveyorDirection::West],
      ConveyorDirection::East => [ConveyorDirection::South, ConveyorDirection::West, ConveyorDirection::North],
      ConveyorDirection::South => [ConveyorDirection::West, ConveyorDirection::North, ConveyorDirection::East],
      ConveyorDirection::West => [ConveyorDirection::North, ConveyorDirection::East, ConveyorDirection::South],
    }
  }
}
