use bevy::prelude::*;
use bevy_egui::{egui::{self, Align2, Pos2, Id}, EguiContexts};

use crate::input::prelude::*;

pub mod plugin_exports {
  pub use super::conveyor_window;
}

pub fn conveyor_window(
  tile_rotation: Option<Res<SelectedTileDirection>>,
  mut contexts: EguiContexts,
  asset_server: Res<AssetServer>,
) {
  let Some(tile_rotation) = tile_rotation else { return; };
  let image = contexts.add_image(asset_server.load("conveyor.png"));

  let ctx = contexts.ctx_mut();

  let offset = 16.0 * tile_rotation.direction.texture_index() as f32;
  let uv = egui::Rect::from_two_pos(
    Pos2::new(offset / 464.0, 0.0),
    Pos2::new((16.0 + offset) / 464.0, 1.0),
  );

  egui::Area::new(Id::null())
    .anchor(Align2::RIGHT_BOTTOM, egui::Vec2::ZERO)
    .show(ctx, |ui| {
      egui::Frame::side_top_panel(&egui::Style::default()).show(ui, |ui| {
        ui.add(egui::widgets::Image::new(image, [64.0, 64.0]).uv(uv));
      })
    });
}
