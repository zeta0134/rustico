use application::RuntimeState;
use drawing::Color;
use drawing::SimpleBuffer;
use events::Event;
use panel::Panel;

use rusticnes_core::nes::NesState;
use rusticnes_core::palettes::NTSC_PAL;

pub struct GameWindow {
    pub canvas: SimpleBuffer,
    pub counter: u8,
    pub shown: bool,
    pub scale: u32,
    pub display_overscan: bool,
}

impl GameWindow {
    pub fn new() -> GameWindow {
        return GameWindow {
            canvas: SimpleBuffer::new(240, 224),
            counter: 0,
            shown: true,
            scale: 2,
            display_overscan: false,
        };
    }

    fn draw(&mut self, nes: &NesState) {
        let overscan: u32 = if self.display_overscan {0} else {8};

        // Update the game screen
        for x in overscan .. 256 - overscan {
            for y in overscan .. 240 - overscan {
                let palette_index = ((nes.ppu.screen[(y * 256 + x) as usize]) as usize) * 3;
                self.canvas.put_pixel(
                    x - overscan, 
                    y - overscan, 
                    Color::rgb(
                        NTSC_PAL[palette_index + 0],
                        NTSC_PAL[palette_index + 1],
                        NTSC_PAL[palette_index + 2])
                );
            }
        }
    }

    fn increase_scale(&mut self) {
        if self.scale < 5 {
            self.scale += 1;
        }
    }

    fn decrease_scale(&mut self) {
        if self.scale > 1 {
            self.scale -= 1;
        }
    }

    fn toggle_overscan(&mut self) {
        if self.display_overscan {
            // Hide overscan:
            self.canvas = SimpleBuffer::new(240, 224);
            self.display_overscan = false;
        } else {
            // Show overscan
            self.canvas = SimpleBuffer::new(256, 240);
            self.display_overscan = true;
        }
    }
}

impl Panel for GameWindow {
    fn title(&self) -> &str {
        return "RusticNES";
    }

    fn shown(&self) -> bool {
        return self.shown;
    }

    fn handle_event(&mut self, runtime: &RuntimeState, event: Event) -> Vec<Event> {
        match event {
            Event::RequestFrame => {self.draw(&runtime.nes)},
            Event::ShowGameWindow => {self.shown = true},
            Event::CloseWindow => {self.shown = false},

            Event::GameIncreaseScale => {self.increase_scale();}
            Event::GameDecreaseScale => {self.decrease_scale();}
            Event::GameToggleOverscan => {self.toggle_overscan();}
            _ => {}
        }
        return Vec::<Event>::new();
    }
    
    fn active_canvas(&self) -> &SimpleBuffer {
        return &self.canvas;
    }

    fn scale_factor(&self) -> u32 {
        return self.scale;
    }
}