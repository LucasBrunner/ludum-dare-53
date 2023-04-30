use bevy::prelude::{IntoSystemConfig, Plugin};

use crate::GameSystemSet;

use self::egui_check::*;

mod egui_check;

pub mod prelude {
  pub use super::egui_check::EguiCapturedResources;
}

#[derive(Debug, Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
  fn build(&self, app: &mut bevy::prelude::App) {
    app
      .add_system(check_egui_captured_resources.in_set(GameSystemSet::PreInputCollection))
      .init_resource::<EguiCapturedResources>();
  }
}
