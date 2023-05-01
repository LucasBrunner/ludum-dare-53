use bevy::{prelude::{Query, ResMut, Resource, Res}, reflect::Reflect};
use bevy_egui::EguiContextQuery;

pub mod prelude {
  pub use super::mouse_captured;
  pub use super::keyboard_captured;
}

#[derive(Debug, Resource, Default, Reflect)]
pub struct EguiCapturedResources {
  mouse: bool,
  keyboard: bool,
}

impl EguiCapturedResources {
  pub fn mouse_captured(&self) -> bool {
    self.mouse
  }
  pub fn keyboard_captured(&self) -> bool {
    self.keyboard
  }
}

pub fn check_egui_captured_resources(
  mut egui: Query<EguiContextQuery>,
  mut captured_resources: ResMut<EguiCapturedResources>,
) {
  *captured_resources =
    egui
      .iter_mut()
      .fold(EguiCapturedResources::default(), |mut acc, mut ctx| {
        let egu_ctx = ctx.ctx.get_mut();
        acc.mouse |= egu_ctx.wants_pointer_input();
        acc.mouse |= egu_ctx.is_pointer_over_area();
        acc.keyboard |= egu_ctx.wants_keyboard_input();
        acc
      })
}

pub fn mouse_captured(captured_resources: Res<EguiCapturedResources>) -> bool {
  captured_resources.mouse
}

pub fn keyboard_captured(captured_resources: Res<EguiCapturedResources>) -> bool {
  captured_resources.mouse
}
