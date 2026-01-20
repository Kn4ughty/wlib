struct State {
    width: u32,
    height: u32,
}

impl wlib::WindowAble for State {
    fn draw(&mut self, buffer: &mut [u8]) {
        buffer
            .chunks_exact_mut(4)
            .enumerate()
            .for_each(|(index, chunk)| {
                let x = ((index as u32) % self.width) + 1;
                let y = (index as u32 / self.width) + 1;
                // println!("{x}, {y}");

                let a = 0xFF;
                let r = u32::min(
                    ((self.width - x) * 0xFF) / self.width,
                    ((self.height - y) * 0xFF) / self.height,
                );
                let g = u32::min(
                    (x * 0xFF) / self.width,
                    ((self.height - y) * 0xFF) / self.height,
                );
                let b = u32::min(
                    ((self.width - x) * 0xFF) / self.width,
                    (y * 0xFF) / self.height,
                );
                let color = (a << 24) + (r << 16) + (g << 8) + b;

                let array: &mut [u8; 4] = chunk.try_into().unwrap();
                *array = color.to_le_bytes();
            });
    }

    fn key_press(&mut self, event: smithay_client_toolkit::seat::keyboard::KeyEvent) {
        println!("kp: {:?}", event);
    }
    fn key_release(&mut self, event: smithay_client_toolkit::seat::keyboard::KeyEvent) {
        println!("kr: {:?}", event);
    }
    fn mouse_event(&mut self, event: smithay_client_toolkit::seat::pointer::PointerEvent) {
        println!("me: {:?}", event);
    }

    fn update_window_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}

fn main() {
    let state = State {
        width: 200,
        height: 200,
    };
    wlib::init(Box::new(state), 200, 200);
}
