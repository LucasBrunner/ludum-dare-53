use bevy::prelude::*;

pub use crate::tile::ConveyorDirection;

pub mod prelude {
  pub use super::SelectedTileDirection;
}

pub mod plugin_exports {
  pub use super::SelectedTileDirection;
  pub use super::change_selected_tile_direction;
}

#[derive(Debug, Resource, Default, Reflect)]
pub struct SelectedTileDirection {
  pub direction: ConveyorDirection,
}

pub fn change_selected_tile_direction(
  keyboard_input: Res<Input<KeyCode>>,
  mut selected_tile_rotation: ResMut<SelectedTileDirection>,
) {
  if keyboard_input.just_pressed(KeyCode::R) {
    selected_tile_rotation.direction = match keyboard_input.pressed(KeyCode::LShift) {
      true => selected_tile_rotation.direction.rotate_counterclockwise(),
      false => selected_tile_rotation.direction.rotate_clockwise(),
    }
  }
}
