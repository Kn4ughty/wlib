struct State {
    pos_x: f32,
    pos_y: f32,
    close_requested: bool,
}

impl wlib::WindowAble for State {
    fn draw(&mut self, buffer: &mut [u8], frame: wlib::WindowSize) {
        let width = frame.width;
        let height = frame.height;

        for (index, chunk) in buffer.chunks_exact_mut(4).enumerate() {
            let x = ((index as u32) % width) + 1;
            let y = (index as u32 / width) + 1;

            if x == self.pos_x as u32 || y == self.pos_y as u32 {
                let array: &mut [u8; 4] = chunk.try_into().unwrap();
                *array = [0, 0, 0, 0];
                continue;
            }

            let a = 0xFF;

            let r = if self.close_requested {
                255
            } else {
                u32::min(((width - x) * 0xFF) / width, ((height - y) * 0xFF) / height)
            };

            let g = u32::min((x * 0xFF) / width, ((height - y) * 0xFF) / height);
            let b = u32::min(((width - x) * 0xFF) / width, (y * 0xFF) / height);
            let color = (a << 24) + (r << 16) + (g << 8) + b;

            let array: &mut [u8; 4] = chunk.try_into().unwrap();
            *array = color.to_le_bytes();
        }
    }

    fn update(&mut self, context: wlib::Context) -> Option<wlib::WLibRequest> {
        println!("Context: {context:#?}");

        self.close_requested = context.close_requested;

        if self.close_requested
            && context
                .pressed_keys
                .contains(&wlib::keyboard::Keysym::from_char('y'))
        {
            return Some(wlib::WLibRequest::CloseAccepted);
        }

        let speed = 200.0 * context.delta_time.as_secs_f32();

        for keysym in context.pressed_keys {
            if keysym == wlib::keyboard::Keysym::from_char('w') && self.pos_y > 0.0 {
                self.pos_y -= speed;
            }
            if keysym == wlib::keyboard::Keysym::from_char('s') {
                self.pos_y += speed;
            }

            if keysym == wlib::keyboard::Keysym::from_char('a') && self.pos_x > 0.0 {
                self.pos_x -= speed;
            }
            if keysym == wlib::keyboard::Keysym::from_char('d') {
                self.pos_x += speed;
            }
        }

        None
    }
}

fn main() {
    wlib::run(
        Box::new(State {
            pos_x: 10.0,
            pos_y: 10.0,
            close_requested: false,
        }),
        wlib::WLibSettings::new().with_static_size(wlib::WindowSize {
            width: 400,
            height: 400,
        }),
    );
}
