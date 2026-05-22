use super::{Drawable, plot_point_screenspace};
use crate::math::Vec2;

#[derive(Debug)]
pub struct Line {
    pub p1: Vec2,
    pub p2: Vec2,
}

impl Line {
    pub fn new(p1: Vec2, p2: Vec2) -> Self {
        Line { p1, p2 }
    }
}

impl Drawable for Line {
    fn draw(&self, buf: &mut [u8], frame_info: &wlib::WindowSize) {
        if (self.p2.y - self.p1.y).abs() < (self.p2.x - self.p1.x).abs() {
            if self.p1.x > self.p2.x {
                Self::plot_line_low(self.p2, self.p1, buf, frame_info);
            } else {
                Self::plot_line_low(self.p1, self.p2, buf, frame_info);
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if self.p1.y > self.p2.y {
                Self::plot_line_high(self.p2, self.p1, buf, frame_info);
            } else {
                Self::plot_line_high(self.p1, self.p2, buf, frame_info);
            }
        }
    }
}

impl Line {
    fn plot_line_low(p1: Vec2, p2: Vec2, buf: &mut [u8], frame_info: &wlib::WindowSize) {
        let dx = p2.x - p1.x;
        let mut dy = p2.y - p1.y;

        let mut yi = 1.0;

        if dy < 0.0 {
            yi = -1.0;
            dy = -dy;
        }

        let mut d = (2.0 * dy) - dx;
        let mut y = p1.y;

        for x in (p1.x as i32)..(p2.x as i32) {
            plot_point_screenspace(buf, frame_info, Vec2::new(x as f32, y));

            if d > 0.0 {
                y += yi;
                d += 2.0 * (dy - dx);
            } else {
                d += 2.0 * dy;
            }
        }
    }

    fn plot_line_high(p1: Vec2, p2: Vec2, buf: &mut [u8], frame_info: &wlib::WindowSize) {
        let mut dx = p2.x - p1.x;
        let dy = p2.y - p1.y;

        let mut xi = 1.0;

        if dx < 0.0 {
            xi = -1.0;
            dx = -dx;
        }

        let mut d = (2.0 * dx) - dy;
        let mut x = p1.x;

        for y in (p1.y as i32)..(p2.y as i32) {
            plot_point_screenspace(buf, frame_info, Vec2::new(x, y as f32));

            if d > 0.0 {
                x += xi;
                d += 2.0 * (dx - dy);
            } else {
                d += 2.0 * dx;
            }
        }
    }
}

#[allow(unused)]
impl Line {
    pub fn intersection(&self, other: &Self) -> Option<Vec2> {
        // https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection#Given_two_points_on_each_line_segment
        let t_top = (self.p1.x - other.p1.x) * (other.p1.y - other.p2.y)
            - (self.p1.y - other.p1.y) * (other.p1.x - other.p2.x);

        let bottom = (self.p1.x - self.p2.x) * (other.p1.y - other.p2.y)
            - (self.p1.y - self.p2.y) * (other.p1.x - other.p2.x);

        let t = t_top / bottom;

        let u_top = (self.p1.x - self.p2.x) * (self.p1.y - other.p1.y)
            - (self.p1.y - self.p2.y) * (self.p1.x - other.p1.x);

        let u = -(u_top / bottom);

        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
            let x = self.p1.x + t * (self.p2.x - self.p1.x);
            let y = self.p1.y + t * (self.p2.y - self.p1.y);
            Some(Vec2::new(x, y))
        } else {
            None
        }
    }
}
