use super::plot_point_screenspace;
use crate::math::Vec2;

#[derive(Debug)]
pub struct Circle {
    pub pos: Vec2,
    pub radius: f32,
}

impl Circle {
    pub fn new(position: Vec2, radius: f32) -> Self {
        Self {
            pos: position,
            radius,
        }
    }
}

impl super::Drawable for Circle {
    fn draw(&self, buf: &mut [u8], frame_info: &wlib::WindowSize) {
        let start = self.pos - self.radius;
        let end = self.pos + self.radius;

        for x in (start.x as i32)..(end.x as i32) {
            for y in (start.y as i32)..(end.y as i32) {
                // TODO. calc distance
                let dx = self.pos.x - x as f32;
                let dy = self.pos.y - y as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < self.radius {
                    plot_point_screenspace(buf, frame_info, Vec2::new(x as f32, y as f32));
                }
            }
        }
    }
}
