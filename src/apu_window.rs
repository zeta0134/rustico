use application::RuntimeState;
use drawing;
use drawing::Font;
use drawing::SimpleBuffer;
use events::Event;
use panel::Panel;

use rusticnes_core::apu::ApuState;
use rusticnes_core::mmc::mapper::Mapper;
use rusticnes_core::apu::AudioChannelState;

pub struct ApuWindow {
    pub canvas: SimpleBuffer,
    pub font: Font,
    pub shown: bool,
    pub waveform_height: u32,
    pub text_height: u32,
    pub spacing: u32,
}

pub fn find_rising_edge(audiobuffer: &[i16], start_index: usize) -> usize {
    let mut last_sample = audiobuffer[start_index];
    let mut current_index = start_index;
    // look ahead 100 samples or so for an edge, from non-zero to zero. If we find one, return it
    for _ in 0 .. 1000 {
        let last_index = current_index;
        current_index = (current_index + 1) % audiobuffer.len();
        let current_sample = audiobuffer[current_index];
        if current_sample != 0 && last_sample == 0 {
            return last_index;
        }
        last_sample = current_sample;
    }
    // Couldn't find a falling edge, so return our original start index instead
    return start_index;
}

impl ApuWindow {
    pub fn new() -> ApuWindow {
        let font = Font::from_raw(include_bytes!("assets/8x8_font.png"), 8);

        return ApuWindow {
            canvas: SimpleBuffer::new(256, 1080),
            font: font,
            shown: false,
            waveform_height: 32,
            text_height: 10,
            spacing: 2,
        };
    }

    pub fn channel_height(&self) -> u32 {
        return self.waveform_height + self.text_height;
    }

    pub fn draw_waveform(&mut self, channel: &dyn AudioChannelState, color: &[u8], x: u32, y: u32, width: u32, height: u32, align: bool) {
        let audiobuffer = channel.sample_buffer().buffer();
        let mut start_index = channel.sample_buffer().index() - ((width as usize) * 2) - 1000;
        start_index = start_index % audiobuffer.len();
        if align {
            start_index = find_rising_edge(audiobuffer, start_index);
        }
        
        let sample_min = channel.min_sample();
        let sample_max = channel.max_sample() + 1;
        let range = (sample_max as u32) - (sample_min as u32);
        let mut last_y = (((audiobuffer[start_index] - sample_min) as u64 * height as u64) / range as u64) as u32;
        if last_y >= height {
            last_y = height - 1;
        }
        for dx in x .. (x + width) {
            let sample_index = (start_index + (dx * 2) as usize) % audiobuffer.len();
            let sample = audiobuffer[sample_index];
            let current_x = dx as u32;
            let mut current_y = (((sample - sample_min) as u64 * height as u64) / range as u64) as u32;
            if current_y >= height {
                current_y = height - 1;
            }
            for dy in current_y .. last_y {
                self.canvas.put_pixel(current_x, y + dy, color);
            }
            for dy in last_y .. current_y {
                self.canvas.put_pixel(current_x, y + dy, color);
            }
            last_y = current_y;
            self.canvas.put_pixel(dx, y + current_y, color);
        }
    }

    pub fn channel_color(channel: &dyn AudioChannelState, index: u32) -> &[u8] {
        if channel.muted() {
            return &[32, 32, 32, 255];
        }
        return match channel.name().as_str() {
            "[2A03] Pulse 1" => {&[192,  32,  32, 255]},
            "[2A03] Pulse 2" => {&[192,  96,  32, 255]},
            "[2A03] Triangle" => {&[32, 192,  32, 255]},
            "[2A03] Noise" => {&[32,  96, 192, 255]},
            "[2A03] DMC" => {&[96,  32, 192, 255]},
            "Final Mix" => {&[192,  192, 192, 255]},
            _ => {
                // Mapper audio, which is definitely pink
                if index % 2 != 0 {
                    &[224, 24, 64, 255]
                } else {
                    &[180, 12, 40, 255]
                }
            } 
        };
    }

    pub fn background_color(foreground_color: &[u8]) -> [u8; 4] {
        return [
            foreground_color[0] / 4,
            foreground_color[1] / 4,
            foreground_color[2] / 4,
            foreground_color[3],
        ];
    }

    pub fn draw_channel(&mut self, x: u32, y: u32, channel: &dyn AudioChannelState) {
        let index = y / self.channel_height();
        let foreground_color = ApuWindow::channel_color(channel, index);
        let background_color = &ApuWindow::background_color(foreground_color);

        let canvas_width = self.canvas.width;
        let channel_height = self.channel_height();
        drawing::rect(&mut self.canvas, x, y, canvas_width, channel_height, background_color);
        drawing::text(&mut self.canvas, &self.font, x, y + 1, channel.name().as_str(), foreground_color);

        self.draw_waveform(channel,
            foreground_color, 
            0,   y + self.text_height, canvas_width,  self.waveform_height, 
            true);
    }

    pub fn draw(&mut self, apu: &ApuState, mapper: &dyn Mapper) {
        let mut channels: Vec<& dyn AudioChannelState> = Vec::new();
        channels.extend(apu.channels());
        channels.extend(mapper.channels());
        channels.push(apu);

        let mut dy = self.spacing;
        for channel in channels {
            self.draw_channel(0, dy, channel);
            dy = dy + self.channel_height() + self.spacing;
        }
    }

    pub fn resize_panel(&mut self, apu: &ApuState, mapper: &dyn Mapper) {
        let mut channels: Vec<& dyn AudioChannelState> = Vec::new();
        channels.extend(apu.channels());
        channels.extend(mapper.channels());
        channels.push(apu);

        self.canvas.height = ((self.channel_height() + self.spacing) * channels.len() as u32) + self.spacing;
        let canvas_width = self.canvas.width;
        let canvas_height = self.canvas.height;
        drawing::rect(&mut self.canvas, 0, 0, canvas_width, canvas_height, &[12, 12, 12, 255]);
    }
}

impl Panel for ApuWindow {
    fn title(&self) -> &str {
        return "APU Surfboard";
    }

    fn shown(&self) -> bool {
        return self.shown;
    }

    fn handle_event(&mut self, runtime: &RuntimeState, event: Event) -> Vec<Event> {
        match event {
            Event::RequestFrame => {self.draw(&runtime.nes.apu, &*runtime.nes.mapper)},
            Event::ShowApuWindow => {self.shown = true},
            Event::CloseWindow => {self.shown = false},
            Event::CartridgeLoaded(_id) => {self.resize_panel(&runtime.nes.apu, &*runtime.nes.mapper)},
            _ => {}
        }
        return Vec::<Event>::new();
    }
    
    fn active_canvas(&self) -> &SimpleBuffer {
        return &self.canvas;
    }

    fn scale_factor(&self) -> u32 {
        return 2;
    }
}