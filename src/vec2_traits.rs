use bevy::prelude::{UVec2, IVec2};
use bevy_ecs_tilemap::tiles::TilePos;

pub trait SetUVec2Values {
  fn set(&mut self, x: u32, y: u32);
  fn set_reverse(&mut self, y: u32, x: u32) {
    self.set(x, y)
  }
}

impl SetUVec2Values for UVec2 {
  fn set(&mut self, x: u32, y: u32) {
    self.x = x;
    self.y = y;
  }
}

pub trait AsIVec2 {
  fn as_ivec2(&self) -> IVec2;
}

impl AsIVec2 for TilePos {
  fn as_ivec2(&self) -> IVec2 {
    IVec2 {
      x: self.x as i32,
      y: self.y as i32,
    }
  }
}

pub trait AddUVec2Values {
  fn add(&mut self, x: i32, y: i32);
  fn add_reverse(&mut self, y: i32, x: i32) {
    self.add(x, y)
  }
}

impl AddUVec2Values for TilePos {
  fn add(&mut self, x: i32, y: i32) {
    self.x = (self.x as i32 + x) as u32;
    self.y = (self.y as i32 + y) as u32;
  }
}

pub trait DistanceFromZero {
  fn closest_to_zero(&self) -> i32;
  fn farthest_from_zero(&self) -> i32;
}

impl DistanceFromZero for IVec2 {
  fn closest_to_zero(&self) -> i32 {
    if self.x.abs() > self.y.abs() {
      self.y
    } else {
      self.x
    }
  }

  fn farthest_from_zero(&self) -> i32 {
    if self.x.abs() < self.y.abs() {
      self.y
    } else {
      self.x
    }
  }
}

pub trait ToTilePos {
  fn to_tile_pos(&self) -> TilePos;
}

impl ToTilePos for UVec2 {
  fn to_tile_pos(&self) -> TilePos {
    TilePos {
      x: self.x,
      y: self.y,
    }
  }
}