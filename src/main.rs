struct State {
    pos_x: u32,
    pos_y: u32,
}

impl wlib::WindowAble for State {
    fn draw(&mut self, buffer: &mut [u8], width: u32, height: u32) {
        for (index, chunk) in buffer.chunks_exact_mut(4).enumerate() {
            let x = ((index as u32) % width) + 1;
            let y = (index as u32 / width) + 1;

            if x == self.pos_x || y == self.pos_y {
                let color: u32 = 0;

                let array: &mut [u8; 4] = chunk.try_into().unwrap();
                *array = color.to_le_bytes();
                continue;
            }

            let a = 0xFF;
            let r = u32::min(((width - x) * 0xFF) / width, ((height - y) * 0xFF) / height);
            let g = u32::min((x * 0xFF) / width, ((height - y) * 0xFF) / height);
            let b = u32::min(((width - x) * 0xFF) / width, (y * 0xFF) / height);
            let color = (a << 24) + (r << 16) + (g << 8) + b;

            let array: &mut [u8; 4] = chunk.try_into().unwrap();
            *array = color.to_le_bytes();
        }
    }

    fn event(&mut self, event: wlib::Event) {
        println!("event: {event:#?}");
    }

    fn update(&mut self, context: wlib::Context) {
        println!("event: {context:#?}");
        for keysym in context.pressed_keys.iter() {
            if *keysym == wlib::keyboard::Keysym::from_char('w') && self.pos_y > 2 {
                self.pos_y -= 2
            }
            if *keysym == wlib::keyboard::Keysym::from_char('s') {
                self.pos_y += 2
            }

            if *keysym == wlib::keyboard::Keysym::from_char('a') && self.pos_x > 2 {
                self.pos_x -= 2
            }
            if *keysym == wlib::keyboard::Keysym::from_char('d') {
                self.pos_x += 2
            }
        }
    }
}

fn main() {
    wlib::run(Box::new(State { pos_x: 0, pos_y: 0 }), 200, 200);
}
