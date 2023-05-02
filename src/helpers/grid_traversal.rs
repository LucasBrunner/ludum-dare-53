use bevy::prelude::{IVec2, Vec2};

pub enum GridTraversal {
  Static {
    position: IVec2,
    remaining: usize,
  },
  Orthogonal {
    current_position: IVec2,
    delta: IVec2,
    remaining: usize,
  },
  NonOrthoginal {
    current_position: IVec2,
    delta_max: Vec2,
    delta: Vec2,
    remaining: usize,
    direction: IVec2,
  },
}

impl GridTraversal {
  pub fn new(start: IVec2, end: IVec2) -> GridTraversal {
    if start == end {
      return GridTraversal::Static {
        position: start,
        remaining: 0,
      };
    }
    if start.x == end.x {
      let diff = end.y - start.y;
      return GridTraversal::Orthogonal {
        current_position: start,
        delta: IVec2::new(0, diff.signum()),
        remaining: diff.abs() as usize,
      };
    }
    if start.y == end.y {
      let diff = end.x - start.x;
      return GridTraversal::Orthogonal {
        current_position: start,
        delta: IVec2::new(diff.signum(), 0),
        remaining: diff.abs() as usize,
      };
    }
    let vector = end - start;
    let vector_f32 = vector.as_vec2();
    GridTraversal::NonOrthoginal {
      current_position: start,
      direction: IVec2::new((end.x - start.x).signum(), (end.y - start.y).signum()),
      delta_max: Vec2 {
        x: vector_f32.x / vector_f32.y,
        y: vector_f32.y / vector_f32.x,
      },
      delta: Vec2 {
        x: (vector_f32 / vector_f32.x).length(),
        y: (vector_f32 / vector_f32.y).length(),
      },
      remaining: vector.x.abs() as usize + vector.y.abs() as usize,
    }
  }

  pub fn extend(mut self, extend_amount: usize) -> GridTraversal {
    match &mut self {
      GridTraversal::Static {
        position: _,
        remaining,
      } => *remaining += extend_amount,
      GridTraversal::Orthogonal {
        current_position: _,
        delta: _,
        remaining,
      } => *remaining += extend_amount,
      GridTraversal::NonOrthoginal {
        current_position: _,
        delta_max: _,
        delta: _,
        remaining,
        direction: _,
      } => *remaining += extend_amount,
    };
    self
  }
}

impl Iterator for GridTraversal {
  type Item = IVec2;

  fn next(&mut self) -> Option<Self::Item> {
    match self {
      GridTraversal::Static {
        position,
        remaining,
      } => {
        if *remaining == 0 {
          return None;
        }
        *remaining -= 1;
        Some(*position)
      }
      GridTraversal::Orthogonal {
        current_position,
        delta,
        remaining,
      } => {
        if *remaining == 0 {
          return None;
        }
        *remaining -= 1;
        let output = *current_position;
        *current_position += *delta;
        Some(output)
      }
      GridTraversal::NonOrthoginal {
        current_position,
        delta_max,
        delta,
        remaining,
        direction,
      } => {
        if *remaining == 0 {
          return None;
        }
        *remaining -= 1;
        let output = *current_position;
        if delta_max.x < delta_max.y {
          delta_max.x += delta.x;
          current_position.x += direction.x;
        } else {
          delta_max.y += delta.y;
          current_position.y += direction.y;
        }
        Some(output)
      }
    }
  }
}

#[cfg(test)]
mod grid_traversal_tests {
  use bevy::prelude::IVec2;

  use super::GridTraversal;

  #[test]
  fn zero_traversal() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::ZERO);
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn zero_extend_one_traversal() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::ZERO).extend(1);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn zero_extend_many_traversal() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::ZERO).extend(5);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn horizontal_traversal() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::X * 5);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 1, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 2, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 3, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 4, y: 0 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn vertical_traversal() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::Y * 5);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 1 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 4 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn diagonal_traversal() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::ONE * 5);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 1 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 1, y: 1 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 1, y: 2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 2, y: 2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 2, y: 3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 3, y: 3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 3, y: 4 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 4, y: 4 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 4, y: 5 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn horizontal_traversal_reverse() {
    let mut traverser = GridTraversal::new(IVec2::X * 5, IVec2::ZERO);
    assert_eq!(traverser.next(), Some(IVec2 { x: 5, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 4, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 3, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 2, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 1, y: 0 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn vertical_traversal_reverse() {
    let mut traverser = GridTraversal::new(IVec2::Y * 5, IVec2::ZERO);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 5 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 4 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 1 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn diagonal_traversal_reverse() {
    let mut traverser = GridTraversal::new(IVec2::ONE * 5, IVec2::ZERO);
    assert_eq!(traverser.next(), Some(IVec2 { x: 5, y: 5 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 5, y: 4 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 4, y: 4 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 4, y: 3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 3, y: 3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 3, y: 2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 2, y: 2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 2, y: 1 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 1, y: 1 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 1, y: 0 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn horizontal_traversal_negative() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::NEG_X * 5);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: -1, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: -2, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: -3, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: -4, y: 0 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn vertical_traversal_negative() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::NEG_Y * 5);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: -1 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: -2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: -3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: -4 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn diagonal_traversal_negative() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::NEG_ONE * 5);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: -1 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: -1, y: -1 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: -1, y: -2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: -2, y: -2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: -2, y: -3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: -3, y: -3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: -3, y: -4 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: -4, y: -4 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: -4, y: -5 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn horizontal_extend_one_traversal() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::X * 5).extend(1);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 1, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 2, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 3, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 4, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 5, y: 0 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn vertical_extend_one_traversal() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::Y * 5).extend(1);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 1 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 4 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 5 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn diagonal_extend_one_traversal() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::ONE * 5).extend(1);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 1 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 1, y: 1 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 1, y: 2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 2, y: 2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 2, y: 3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 3, y: 3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 3, y: 4 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 4, y: 4 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 4, y: 5 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 5, y: 5 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn horizontal_extend_many_traversal() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::X).extend(5);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 1, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 2, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 3, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 4, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 5, y: 0 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn vertical_extend_many_traversal() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::Y).extend(5);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 1 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 4 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 5 }));
    assert_eq!(traverser.next(), None);
  }

  #[test]
  fn diagonal_extend_many_traversal() {
    let mut traverser = GridTraversal::new(IVec2::ZERO, IVec2::ONE).extend(9);
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 0 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 0, y: 1 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 1, y: 1 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 1, y: 2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 2, y: 2 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 2, y: 3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 3, y: 3 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 3, y: 4 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 4, y: 4 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 4, y: 5 }));
    assert_eq!(traverser.next(), Some(IVec2 { x: 5, y: 5 }));
    assert_eq!(traverser.next(), None);
  }
}
