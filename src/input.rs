pub mod chained_tile;
mod egui_check;
pub mod tile_rotation;

use bevy::prelude::{IntoSystemConfig, Plugin};

use crate::GameSystemSet;

use self::{
  chained_tile::plugin_exports::*, egui_check::plugin_exports::*, tile_rotation::plugin_exports::*,
};

pub mod prelude {
  pub use super::egui_check::prelude::*;
  pub use super::InputPlugin;
  pub use super::tile_rotation;
}

#[derive(Debug, Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
  fn build(&self, app: &mut bevy::prelude::App) {
    app
      // egui capture
      .init_resource::<EguiCapturedResources>()
      .add_system(check_egui_captured_resources.in_set(GameSystemSet::PreInputCollection))
      // tile placing
      .init_resource::<ChainedTileResource>()
      .add_event::<ChainedTileChangeEvent>()
      .add_system(catch_chained_tile_input.in_set(GameSystemSet::InputCollection))
      // tile rotation
      .init_resource::<SelectedTileDirection>()
      .add_system(change_selected_tile_direction.in_set(GameSystemSet::InputCollection));
  }
}
