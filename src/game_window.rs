use application::RuntimeState;
use drawing;
use drawing::Color;
use drawing::Font;
use drawing::SimpleBuffer;
use events::Event;
use panel::Panel;

use std::time::Instant;

use rusticnes_core::nes::NesState;
use rusticnes_core::palettes::NTSC_PAL;

pub struct GameWindow {
    pub canvas: SimpleBuffer,
    pub font: Font,
    pub shown: bool,
    pub scale: u32,
    pub display_overscan: bool,
    pub ntsc_filter: bool,
    pub display_fps: bool,

    pub frame_duration: Instant,
    pub durations: [f32; 60],
    pub duration_index: usize,
    pub measured_fps: f32,
}

impl GameWindow {
    pub fn new() -> GameWindow {
        let font = Font::from_raw(include_bytes!("assets/8x8_font.png"), 8);

        return GameWindow {
            canvas: SimpleBuffer::new(240*2, 224*2),
            font: font,
            shown: true,
            scale: 1,
            display_overscan: false,
            ntsc_filter: true,
            display_fps: true,

            frame_duration: Instant::now(),
            durations: [0f32; 60],
            duration_index: 0,
            measured_fps: 0.0,
        };
    }

    fn update_fps(&mut self) {
        let time_since_last = self.frame_duration.elapsed().as_millis() as f32;
        self.frame_duration = Instant::now();
        self.durations[self.duration_index] = time_since_last;
        self.duration_index = (self.duration_index + 1) % 60;
        let average_frame_duration_millis = self.durations.iter().sum::<f32>() as f32 / (self.durations.len() as f32);
        if average_frame_duration_millis > 0.0 {
            self.measured_fps = 1000.0 / average_frame_duration_millis;
        }
    }

    fn draw(&mut self, nes: &NesState) {
        let overscan: u32 = if self.display_overscan {0} else {8};

        // Update the game screen
        for x in overscan .. 256 - overscan {
            for y in overscan .. 240 - overscan {
                if self.ntsc_filter {
                    let color_left = Color::from_raw(nes.ppu.filtered_screen[(y * 512 + x * 2) as usize]);
                    let color_right = Color::from_raw(nes.ppu.filtered_screen[(y * 512 + x * 2 + 1) as usize]);
                    self.canvas.put_pixel((x - overscan) * 2,     (y - overscan) * 2, color_left);
                    self.canvas.put_pixel((x - overscan) * 2 + 1, (y - overscan) * 2, color_right);
                    self.canvas.put_pixel((x - overscan) * 2,     (y - overscan) * 2 + 1, color_left);
                    self.canvas.put_pixel((x - overscan) * 2 + 1, (y - overscan) * 2 + 1, color_right);
                } else {
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

        if self.display_fps {
            let fps_display = format!("FPS: {:.2}", self.measured_fps);
            drawing::text(&mut self.canvas, &self.font, 5, 5, &fps_display, Color::rgba(255, 255, 255, 192));
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
            self.canvas = SimpleBuffer::new(240 * 2, 224 * 2);
            self.display_overscan = false;
        } else {
            // Show overscan
            self.canvas = SimpleBuffer::new(256 * 2, 240 * 2);
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
        let mut responses = Vec::<Event>::new();
        match event {
            Event::RequestFrame => {
                self.update_fps();
                self.draw(&runtime.nes);
                // Technically this will have us drawing one frame behind the filter. To fix
                // this, we'd need Application to manage filters instead.
                if self.ntsc_filter {
                    responses.push(Event::NesRenderNTSC(512));
                }
            },
            Event::ShowGameWindow => {self.shown = true},
            Event::CloseWindow => {self.shown = false},

            Event::GameIncreaseScale => {self.increase_scale();}
            Event::GameDecreaseScale => {self.decrease_scale();}
            Event::GameToggleOverscan => {self.toggle_overscan();}
            _ => {}
        }
        return responses;
    }
    
    fn active_canvas(&self) -> &SimpleBuffer {
        return &self.canvas;
    }

    fn scale_factor(&self) -> u32 {
        return self.scale;
    }
}