use bevy::{prelude::*, math::Vec4Swizzles};

use super::CameraMoved;

#[derive(Resource)]
pub struct CursorPos(Vec2, Vec2);
impl Default for CursorPos {
  fn default() -> Self {
    // Initialize the cursor pos at some far away place. It will get updated
    // correctly when the cursor moves.
    Self(Vec2::new(-1000.0, -1000.0), Vec2::ZERO)
  }
}

impl CursorPos {
  pub fn to_map_pos(&self, map_transform: &Transform) -> Vec2 {
    // Grab the cursor position from the `Res<CursorPos>`
    let cursor_pos: Vec2 = self.0;
    // We need to make sure that the cursor's world position is correct relative to the map
    // due to any map transformation.
    let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
    let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
    cursor_in_map_pos.xy()
  }
}

// We need to keep the cursor position updated based on any `CursorMoved` events.
pub fn update_cursor_pos(
  camera_q: Query<(&GlobalTransform, &Camera)>,
  mut cursor_moved_events: EventReader<CursorMoved>,
  camera_moved_events: EventReader<CameraMoved>,
  mut cursor_pos: ResMut<CursorPos>,
) {
  if cursor_moved_events.len() == 0 && camera_moved_events.len() != 0 {
    for (cam_t, cam) in camera_q.iter() {
      if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_pos.1) {
        *cursor_pos = CursorPos(pos, cursor_pos.1);
      }
    }
    return;
  }

  for cursor_moved in cursor_moved_events.iter() {
    // To get the mouse's world position, we have to transform its window position by
    // any transforms on the camera. This is done by projecting the cursor position into
    // camera space (world space).
    for (cam_t, cam) in camera_q.iter() {
      if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
        *cursor_pos = CursorPos(pos, cursor_moved.position);
      }
    }
  }
}