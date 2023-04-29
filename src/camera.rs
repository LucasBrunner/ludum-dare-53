use bevy::{input::Input, math::Vec3, prelude::*, render::camera::Camera};
use bevy_pixel_camera::PixelProjection;

pub struct CameraMoved();

// A simple camera system for moving and zooming the camera.
#[allow(dead_code)]
pub fn movement(
  time: Res<Time>,
  keyboard_input: Res<Input<KeyCode>>,
  mut camera_move_event: EventWriter<CameraMoved>,
  mut query: Query<(&mut Transform, &mut PixelProjection), With<Camera>>,
) {
  for (mut transform, mut ortho) in query.iter_mut() {
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::A) {
      direction -= Vec3::new(1.0, 0.0, 0.0);
    }

    if keyboard_input.pressed(KeyCode::D) {
      direction += Vec3::new(1.0, 0.0, 0.0);
    }

    if keyboard_input.pressed(KeyCode::W) {
      direction += Vec3::new(0.0, 1.0, 0.0);
    }

    if keyboard_input.pressed(KeyCode::S) {
      direction -= Vec3::new(0.0, 1.0, 0.0);
    }

    let mut round_transform = false;
    if keyboard_input.just_pressed(KeyCode::Z) {
      ortho.zoom += 1;
      round_transform = true;
    }

    if keyboard_input.just_pressed(KeyCode::X) {
      ortho.zoom -= 1;
      round_transform = true;
    }

    if ortho.zoom < 1 {
      ortho.zoom = 1;
    }

    if round_transform || direction.x != 0.0 || direction.y != 0.0 {
      camera_move_event.send(CameraMoved());
    }

    let z = transform.translation.z;
    transform.translation += time.delta_seconds() * direction * 500.;
    if round_transform {
      transform.translation.round();
    }
    // Important! We need to restore the Z values when moving the camera around.
    // Bevy has a specific camera setup and this can mess with how our layers are shown.
    transform.translation.z = z;
  }
}
