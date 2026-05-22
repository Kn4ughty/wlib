mod drawing;
use drawing::{Drawable, circle::Circle, rectangle::Rect};

mod math;
use math::Vec2;

use rand::RngExt;
use wlib::{WindowAble, keys};

#[derive(Debug)]
struct Game {
    pipes: Vec<Pipe>,
    bird: Bird,
    score: u32,
    time_since_last_pipe: std::time::Instant,
    rng: rand::rngs::ThreadRng,
}

#[derive(Debug)]
struct Bird {
    size: f32,
    pos: Vec2,
    vel: Vec2,
}

impl Drawable for Bird {
    fn draw(&self, buf: &mut [u8], frame_info: &wlib::WindowSize) {
        self.to_circ().draw(buf, frame_info);
    }
}

impl Bird {
    fn to_circ(&self) -> Circle {
        Circle::new(self.pos, self.size)
    }
}

#[derive(Debug, Clone, Copy)]
struct Pipe {
    pos: Vec2,
    gap: f32,
    width: f32,
    score_checked: bool,
}

impl Drawable for Pipe {
    fn draw(&self, buf: &mut [u8], frame_info: &wlib::WindowSize) {
        let (r1, r2) = self.to_two_rectangles(frame_info.height);
        r1.draw(buf, frame_info);
        r2.draw(buf, frame_info);
    }
}

impl Pipe {
    fn to_two_rectangles(self, frame_height: u32) -> (Rect, Rect) {
        // let height = (frame_info.height / 2) as f32 - self.gap;

        // top pipe
        let r1p = Vec2::new(self.pos.x, 0.0);

        let r1 = Rect::new(self.width, self.pos.y - self.gap, r1p);

        // bottom pipe
        let r2p = self.pos.add_y(self.gap);

        let r2 = Rect::new(self.width, frame_height as f32 - self.pos.y - self.gap, r2p);

        (r1, r2)
    }

    fn circle_intersection(&self, circ: Circle, frame_height: u32) -> bool {
        let (r1, r2) = self.to_two_rectangles(frame_height);

        // interesection with bottom pipe
        r1.intersection_with_circle(&circ) || r2.intersection_with_circle(&circ)
    }
}

impl WindowAble for Game {
    fn draw(&mut self, buf: &mut [u8], frame_info: wlib::WindowSize) {
        for chunk in buf.chunks_exact_mut(4) {
            let array: &mut [u8; 4] = chunk.try_into().unwrap();
            *array = [0, 0, 0, 255];
        }

        self.bird.draw(buf, &frame_info);
        self.pipes.iter().for_each(|p| p.draw(buf, &frame_info));
    }

    fn update(&mut self, context: wlib::Context) -> Option<wlib::WLibRequest> {
        if context.close_requested {
            return Some(wlib::WLibRequest::CloseAccepted);
        }

        // println!("keys: {:#?}", context.pressed_keys);
        // println!("self: {:#?}", self);

        let speed = 100.0 * context.delta_time.as_secs_f32();

        if self.bird.vel.y < 20.0 {
            self.bird.vel.y += 1.0;
        }

        self.bird.pos = self.bird.pos + self.bird.vel;

        for event in &context.event_queue {
            match event {
                wlib::Event::KeyPress(wlib::keyboard::KeyEvent {
                    raw_code: keys::KEY_SPACE,
                    ..
                })
                | wlib::Event::PointerEvent(wlib::PointerEvent {
                    kind:
                        wlib::PointerEventKind::Press {
                            time: _,
                            button: _,
                            serial: _,
                        },
                    ..
                }) => {
                    self.bird.vel.y = -8.5 * speed;
                }
                _ => {}
            }
        }

        for pipe in self.pipes.iter_mut() {
            pipe.pos.x -= 2.0 * speed;

            if !pipe.score_checked && self.bird.pos.x > pipe.pos.x + pipe.width {
                pipe.score_checked = true;
                self.score += 1;
                println!("Your score is now: {}", self.score)
            }

            if pipe.circle_intersection(self.bird.to_circ(), context.window_size.height) {
                println!("uh oh you died");
                println!("Your final score was: {}", self.score);
                return Some(wlib::WLibRequest::CloseAccepted);
            }
        }

        if self.bird.pos.y < 0.0 || self.bird.pos.y > context.window_size.height as f32 {
            println!("uh oh you died");
            println!("Your final score was: {}", self.score);
            return Some(wlib::WLibRequest::CloseAccepted);
        }

        self.pipes = self
            .pipes
            .iter()
            .filter(|p| p.pos.x > -p.width)
            .copied()
            .collect();

        if self.time_since_last_pipe.elapsed() > std::time::Duration::from_millis(1750) {
            self.spawn_pipe(&context);
            self.time_since_last_pipe = std::time::Instant::now();
        }

        None
    }
}

impl Game {
    fn spawn_pipe(&mut self, context: &wlib::Context) {
        let width = 100.0;

        let posy = self
            .rng
            .random_range((100.0)..((context.window_size.height as f32) - 100.0));

        self.pipes.push(Pipe {
            pos: Vec2 {
                x: context.window_size.width as f32,
                y: posy,
            },
            gap: 90.0,
            width,
            score_checked: false,
        });
    }
}

fn main() {
    wlib::run(
        Box::new(Game {
            bird: Bird {
                pos: Vec2::new(50.0, 100.0),
                vel: Vec2 { x: 0.0, y: 0.0 },
                size: 15.0,
            },
            pipes: Vec::new(),
            score: 0,
            time_since_last_pipe: std::time::Instant::now(),
            rng: rand::rng(),
        }),
        wlib::WLibSettings::new().with_title("Bouncy Circle"),
    );
}
