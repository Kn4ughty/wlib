pub mod circle;
pub mod line;
pub mod rectangle;

use crate::math::Vec2;

fn plot_point_screenspace(buf: &mut [u8], frame_info: &wlib::WindowSize, point: Vec2) {
    if point.x > (frame_info.width as f32)
        || point.x < 0.0
        || point.y > (frame_info.height as f32)
        || point.y < 0.0
    {
        return;
    }

    let index = (point.y as u32 * frame_info.width) + point.x as u32;

    if let Some(p) = buf.chunks_mut(4).nth(index as usize) {
        p[0] = 255;
        p[2] = 255;
    }
}

pub trait Drawable {
    fn draw(&self, buf: &mut [u8], frame_info: &wlib::WindowSize);
}
