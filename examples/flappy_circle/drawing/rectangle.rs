use super::{Vec2, line::Line};

#[derive(Debug)]
pub struct Rect {
    width: f32,
    height: f32,
    // Centered at bottom left
    pos: Vec2,
}

impl Rect {
    pub fn new(width: f32, height: f32, pos: Vec2) -> Self {
        Self { width, height, pos }
    }

    pub fn intersection_with_circle(&self, circ: &super::circle::Circle) -> bool {
        let centre = self.pos.add_x(self.width / 2.0).add_y(self.height / 2.0);

        let circ_dist = Vec2 {
            x: (circ.pos.x - centre.x).abs(),
            y: (circ.pos.y - centre.y).abs(),
        };

        if circ_dist.x > (self.width / 2.0 + circ.radius) {
            return false;
        }
        if circ_dist.y > (self.height / 2.0 + circ.radius) {
            return false;
        }

        if circ_dist.x <= (self.width / 2.0) {
            return true;
        }
        if circ_dist.y <= (self.height / 2.0) {
            return true;
        }

        let corner_dist_sq =
            (circ_dist.x - self.width / 2.0).powi(2) + (circ_dist.y - self.height / 2.0).powi(2);

        corner_dist_sq <= (circ.radius.powi(2))
    }
}

impl super::Drawable for Rect {
    fn draw(&self, buf: &mut [u8], frame_info: &wlib::WindowSize) {
        let l1 = Line::new(self.pos, self.pos.add_x(self.width));
        let l2 = Line::new(self.pos, self.pos.add_y(self.height));
        let l3 = Line::new(
            self.pos.add_y(self.height),
            self.pos.add_x(self.width).add_y(self.height),
        );
        let l4 = Line::new(
            self.pos.add_x(self.width),
            self.pos.add_x(self.width).add_y(self.height),
        );

        l1.draw(buf, frame_info);
        l2.draw(buf, frame_info);
        l3.draw(buf, frame_info);
        l4.draw(buf, frame_info);
    }
}
