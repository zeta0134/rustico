use application::RuntimeState;
use drawing;
use drawing::Color;
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

    pub fn draw_waveform(&mut self, channel: &dyn AudioChannelState, color: Color, x: u32, y: u32, width: u32, height: u32, align: bool) {
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

    pub fn channel_color(channel: &dyn AudioChannelState, index: u32) -> Color {
        if channel.muted() {
            return Color::rgb(32, 32, 32);
        }
        return match channel.name().as_str() {
            "[2A03] Pulse 1" => {Color::rgb(192,  32,  32)},
            "[2A03] Pulse 2" => {Color::rgb(192,  96,  32)},
            "[2A03] Triangle" => {Color::rgb(32, 192,  32)},
            "[2A03] Noise" => {Color::rgb(32,  96, 192)},
            "[2A03] DMC" => {Color::rgb(96,  32, 192)},
            "Final Mix" => {Color::rgb(192,  192, 192)},
            _ => {
                // Mapper audio, which is definitely pink
                if index % 2 != 0 {
                    Color::rgb(224, 24, 64)
                } else {
                    Color::rgb(180, 12, 40)
                }
            } 
        };
    }

    pub fn background_color(foreground_color: Color) -> Color {
        return Color::rgb(
            foreground_color.r() / 4,
            foreground_color.g() / 4,
            foreground_color.b() / 4
        );
    }

    pub fn draw_channel(&mut self, x: u32, y: u32, channel: &dyn AudioChannelState) {
        let index = y / self.channel_height();
        let foreground_color = ApuWindow::channel_color(channel, index);
        let background_color = ApuWindow::background_color(foreground_color);

        let canvas_width = self.canvas.width;
        let channel_height = self.channel_height();
        drawing::rect(&mut self.canvas, x, y, canvas_width, channel_height, background_color);
        drawing::text(&mut self.canvas, &self.font, x, y + 1, channel.name().as_str(), foreground_color);

        self.draw_waveform(channel,
            foreground_color, 
            0,   y + self.text_height, canvas_width,  self.waveform_height, 
            true);
    }

    pub fn collect_channels<'a>(apu: &'a ApuState, mapper: &'a dyn Mapper) -> Vec<&'a dyn AudioChannelState> {
        let mut channels: Vec<& dyn AudioChannelState> = Vec::new();
        channels.extend(apu.channels());
        channels.extend(mapper.channels());
        channels.push(apu);
        return channels;
    }

    pub fn draw(&mut self, apu: &ApuState, mapper: &dyn Mapper) {
        let channels = ApuWindow::collect_channels(apu, mapper);

        let mut dy = self.spacing;
        for channel in channels {
            self.draw_channel(0, dy, channel);
            dy = dy + self.channel_height() + self.spacing;
        }
    }

    pub fn resize_panel(&mut self, apu: &ApuState, mapper: &dyn Mapper) {
        let channels = ApuWindow::collect_channels(apu, mapper);

        self.canvas.height = ((self.channel_height() + self.spacing) * channels.len() as u32) + self.spacing;
        let canvas_width = self.canvas.width;
        let canvas_height = self.canvas.height;
        drawing::rect(&mut self.canvas, 0, 0, canvas_width, canvas_height, Color::rgb(12, 12, 12));
    }

    pub fn mouse_mutes_channel(&mut self, apu: &ApuState, mapper: &dyn Mapper, my: i32) -> Vec<Event> {
        let mut events: Vec<Event> = Vec::new();
        let channels = ApuWindow::collect_channels(apu, mapper);
        let channel_index = ((my as u32) / (self.channel_height() + self.spacing)) as usize;
        if channel_index < (channels.len() - 1) { // do not attempt to mute the final mix
            if channels[channel_index].muted() {
                events.push(Event::UnmuteChannel(channel_index))
            } else {
                events.push(Event::MuteChannel(channel_index))
            }
        }
        return events;
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
        let mut events: Vec<Event> = Vec::new();
        match event {
            Event::RequestFrame => {self.draw(&runtime.nes.apu, &*runtime.nes.mapper)},
            Event::ShowApuWindow => {self.shown = true},
            Event::CloseWindow => {self.shown = false},
            Event::CartridgeLoaded(_id) => {self.resize_panel(&runtime.nes.apu, &*runtime.nes.mapper)},
            Event::MouseClick(_x, y) => {events.extend(self.mouse_mutes_channel(&runtime.nes.apu, &*runtime.nes.mapper, y));},
            _ => {}
        }
        return events;
    }
    
    fn active_canvas(&self) -> &SimpleBuffer {
        return &self.canvas;
    }

    fn scale_factor(&self) -> u32 {
        return 2;
    }
}