use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    pub fn add_x(&self, add: f32) -> Self {
        let mut new = *self;
        new.x += add;
        new
    }

    pub fn add_y(&self, add: f32) -> Self {
        let mut new = *self;
        new.y += add;
        new
    }
}

impl std::convert::From<(f64, f64)> for Vec2 {
    fn from(value: (f64, f64)) -> Self {
        Vec2 {
            x: value.0 as f32,
            y: value.1 as f32,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<f32> for Vec2 {
    type Output = Self;
    fn sub(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<f32> for Vec2 {
    type Output = Self;
    fn add(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}
