use bevy::prelude::{IVec2, Vec2};

pub struct GridTraversal {
  start_point: IVec2,
  current_offset: IVec2,
  direction: IVec2,
  delta_max: Vec2,
  delta: Vec2,
  iterations_left: usize,
  vector: IVec2,
}

impl GridTraversal {
  pub fn new(start: IVec2, end: IVec2) -> GridTraversal {
    let vector = IVec2::new(start.x - end.x, start.y - end.y);
    let vector_f32 = vector.as_vec2();
    GridTraversal {
      start_point: start,
      current_offset: IVec2::ZERO,
      direction: IVec2::new((end.x - start.x).signum(), (end.y - start.y).signum()),
      delta_max: Vec2 {
        x: vector_f32.x / vector_f32.y,
        y: vector_f32.y / vector_f32.x,
      },
      delta: Vec2 {
        x: (vector_f32 / vector_f32.x).length(),
        y: (vector_f32 / vector_f32.y).length(),
      },
      iterations_left: vector.x.abs() as usize + vector.y.abs() as usize,
      vector,
    }
  }
}

impl Iterator for GridTraversal {
  type Item = IVec2;

  fn next(&mut self) -> Option<Self::Item> {
    if self.iterations_left <= 0 {
      return None;
    }
    self.iterations_left -= 1;
    if (self.delta_max.x < self.delta_max.y && self.vector.x != 0) || self.vector.y == 0 {
      self.delta_max.x += self.delta.x;
      self.current_offset.x += self.direction.x;
    } else {
      self.delta_max.y += self.delta.y;
      self.current_offset.y += self.direction.y;
    }
    Some(self.start_point + self.current_offset)
  }
}

#[cfg(test)]
mod grid_traversal_tests {
  use bevy::prelude::IVec2;

  use super::GridTraversal;

  #[test]
  fn horizontal_traversal() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::X * 5);
    assert_eq!(traverser.next(), Some(IVec2 { x: 1, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 2, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 3, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 4, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 5, y: 0 }));
    assert_eq!(traverser.next(), None);
  }
}
