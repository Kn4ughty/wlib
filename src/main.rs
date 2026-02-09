use wlib::{WindowAble, keys};

struct State {
    pos_x: f64,
    pos_y: f64,
    is_mouse_mode: bool,
    close_requested: bool,
}

impl WindowAble for State {
    fn draw(&mut self, buffer: &mut [u8], frame: wlib::WindowSize) {
        let width = frame.width;
        let height = frame.height;

        for x in 0..frame.width {
            for y in 0..frame.height {
                let index = ((y * frame.width + x) * 4) as usize;
                if x == self.pos_x as u32 || y == self.pos_y as u32 {
                    buffer[index + 0] = 0;
                    buffer[index + 1] = 0;
                    buffer[index + 2] = 0;
                    buffer[index + 3] = 0;
                    continue;
                }

                let r = if self.close_requested {
                    255
                } else {
                    u32::min(((width - x) * 0xFF) / width, ((height - y) * 0xFF) / height)
                };
                let g = u32::min((x * 0xFF) / width, ((height - y) * 0xFF) / height);
                let b = u32::min(((width - x) * 0xFF) / width, (y * 0xFF) / height);
                let a = 255;

                buffer[index + 0] = b as u8;
                buffer[index + 1] = g as u8;
                buffer[index + 2] = r as u8;
                buffer[index + 3] = a as u8;
            }
        }
    }

    fn update(&mut self, context: wlib::Context) -> Option<wlib::WLibRequest> {
        self.close_requested = context.close_requested;

        if self.close_requested && context.pressed_keys.contains_key(&wlib::keys::KEY_Y) {
            return Some(wlib::WLibRequest::CloseAccepted);
        }

        if context.event_queue.iter().any(|event| {
            matches!(
                event,
                wlib::Event::KeyPress(wlib::keyboard::KeyEvent {
                    raw_code: keys::KEY_M,
                    ..
                })
            )
        }) {
            self.is_mouse_mode = !self.is_mouse_mode;
        }

        let speed = 200.0 * context.delta_time.as_secs_f64();

        // Handle keyboard input
        if context.pressed_keys.contains_key(&keys::KEY_W) && self.pos_y > 0.0 {
            self.pos_y -= speed;
        }
        if context.pressed_keys.contains_key(&keys::KEY_S) {
            self.pos_y += speed;
        }
        if context.pressed_keys.contains_key(&keys::KEY_A) && self.pos_x > 0.0 {
            self.pos_x -= speed;
        }
        if context.pressed_keys.contains_key(&keys::KEY_D) {
            self.pos_x += speed;
        }

        if self.is_mouse_mode {
            (self.pos_x, self.pos_y) = context.mouse_state.position;
        }

        None
    }
}

fn main() {
    wlib::run(
        Box::new(State {
            pos_x: 10.0,
            pos_y: 10.0,
            is_mouse_mode: false,
            close_requested: false,
        }),
        wlib::WLibSettings::new().with_static_size(wlib::WindowSize {
            width: 400,
            height: 400,
        }),
    );
}
