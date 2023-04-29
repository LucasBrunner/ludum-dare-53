use std::fmt::Display;

use bevy::prelude::{Plugin, apply_system_buffers, IntoSystemConfigs};

use crate::camera::prelude::update_cursor_pos;

use self::update::systems::*;
use self::placement::systems::*;

pub mod placement;
pub mod update;

pub mod prelude {
  pub use super::ConveyorDirection;
  pub use super::ConveyorBuildPlugin;
}

pub struct ConveyorBuildPlugin;

impl Plugin for ConveyorBuildPlugin {
  fn build(&self, app: &mut bevy::prelude::App) {
    app
    .init_resource::<placement::PreviousTilePlaceAttempt>()
    .add_event::<placement::PlaceTile>()
    .add_event::<update::ChangeConveyorDirection>()
    .add_systems(
      (
        detect_tile_place,
        place_tiles_drag,
        apply_system_buffers,
        update_tile_direction,
      ).after(update_cursor_pos)
        .chain(),
    );
  }
}

#[derive(Debug, Clone, Copy)]
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
      ConveyorDirection::South => "South",
      ConveyorDirection::East => "East",
      ConveyorDirection::West => "West",
    }
  }
}
