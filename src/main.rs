struct State {}

impl wlib::WindowAble for State {
    fn draw(&mut self, buffer: &mut [u8], width: u32, height: u32) {
        // println!("width: {width}, height: {height}");
        buffer
            .chunks_exact_mut(4)
            .enumerate()
            .for_each(|(index, chunk)| {
                let x = ((index as u32) % width) + 1;
                let y = (index as u32 / width) + 1;
                // println!("{x}, {y}");

                let a = 0xFF;
                let r = u32::min(((width - x) * 0xFF) / width, ((height - y) * 0xFF) / height);
                let g = u32::min((x * 0xFF) / width, ((height - y) * 0xFF) / height);
                let b = u32::min(((width - x) * 0xFF) / width, (y * 0xFF) / height);
                let color = (a << 24) + (r << 16) + (g << 8) + b;

                let array: &mut [u8; 4] = chunk.try_into().unwrap();
                *array = color.to_le_bytes();
            });
    }

    fn event(&mut self, event: wlib::Event) {
        println!("event: {event:#?}");
    }

    fn update(&mut self, context: wlib::Context) {
        println!("event: {context:#?}");
    }
}

fn main() {
    wlib::run(Box::new(State {}), 200, 200);
}
