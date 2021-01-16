use panel::Panel;
use drawing::SimpleBuffer;

use events::Event;

pub struct TestWindow {
    pub canvas: SimpleBuffer,
    pub counter: u8
}

impl TestWindow {
    pub fn new() -> TestWindow {
        return TestWindow {
            canvas: SimpleBuffer::new(256, 256),
            counter: 0,
        };
    }

    fn update(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }

    fn draw(&mut self) {
        for x in 0 ..= 255 {
            for y in 0 ..= 255 {
                let r = x;
                let g = y;
                let b = self.counter.wrapping_add(x ^ y);
                self.canvas.put_pixel(x as u32, y as u32, &[r, g, b, 255]);
            }
        }
    }
}

impl Panel for TestWindow {
    fn title(&self) -> &str {
        return "Hello World!";
    }

    fn handle_event(&mut self, event: Event) -> Vec<Event> {
        match event {
            Event::Update => {self.update()},
            Event::RequestFrame => {self.draw()},
            //_ => {}
        }
        return Vec::<Event>::new();
    }
    
    fn active_canvas(&self) -> &SimpleBuffer {
        return &self.canvas;
    }
}