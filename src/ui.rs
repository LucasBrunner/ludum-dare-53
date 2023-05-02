pub mod tile_preview;

use bevy::prelude::*;

pub use tile_preview::plugin_exports::*;

use crate::GameSystemSet;

pub mod prelude {
  pub use super::UiPlugin;
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
  fn build(&self, app: &mut bevy::prelude::App) {
    app.add_system(conveyor_window.in_set(GameSystemSet::PostTilePlacing));
  }
}
