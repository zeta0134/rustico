use application::RuntimeState;
use drawing;
use drawing::Font;
use drawing::SimpleBuffer;
use events::Event;
use panel::Panel;

use rusticnes_core::apu::ApuState;

pub struct ApuWindow {
    pub canvas: SimpleBuffer,
    pub font: Font,
    pub shown: bool,
}

impl ApuWindow {
    pub fn new() -> ApuWindow {
        let font = Font::from_raw(include_bytes!("assets/8x8_font.png"), 8);

        return ApuWindow {
            canvas: SimpleBuffer::new(256, 240),
            font: font,
            shown: false,
        };
    }

    pub fn draw_waveform(&mut self, audiobuffer: &[i16], start_index: usize, color: &[u8], x: u32, y: u32, width: u32, height: u32, sample_min: i16, sample_max: i16) {
        let mut last_y = 0;
        for dx in x .. (x + width) {
            let sample_index = (start_index + dx as usize) % audiobuffer.len();
            let sample = audiobuffer[sample_index];
            let current_x = dx as u32;
            let range = (sample_max as u32) - (sample_min as u32);
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

    pub fn draw_audio_samples(&mut self, apu: &ApuState) {
        // Background
        // TODO: Optimize this somewhat

        struct ChannelDefinition<'a> {
            buffer: &'a [i16],
            disabled: bool,
            background_color: &'a [u8],
            foreground_color: &'a [u8],
            min: i16,
            max: i16,
        };

        let audio_buffers = [
            ChannelDefinition {
                buffer: &apu.pulse_1.debug_buffer, 
                disabled: apu.pulse_1.debug_disable,
                background_color: &[32,  8,  8, 255], 
                foreground_color: &[192,  32,  32, 255],
                min: 0, max: 16
            },
            ChannelDefinition {
                buffer: &apu.pulse_2.debug_buffer,
                disabled: apu.pulse_2.debug_disable,
                background_color: &[32, 16,  8, 255],
                foreground_color: &[192,  96,  32, 255],
                min: 0, max: 16
            },
            ChannelDefinition {
                buffer: &apu.triangle.debug_buffer, 
                disabled: apu.triangle.debug_disable,
                background_color: &[ 8, 32,  8, 255],
                foreground_color: &[32, 192,  32, 255],
                min: 0, max: 16
            },
            ChannelDefinition {
                buffer: &apu.noise.debug_buffer, 
                disabled: apu.noise.debug_disable,
                background_color: &[ 8, 16, 32, 255],
                foreground_color: &[32,  96, 192, 255],
                min: 0, max: 16
            },
            ChannelDefinition {
                buffer: &apu.dmc.debug_buffer, 
                disabled: apu.dmc.debug_disable,
                background_color: &[ 16, 8, 32, 255],
                foreground_color: &[96,  32, 192, 255],
                min: 0, max: 128
            },
            ChannelDefinition {
                buffer: &apu.sample_buffer, 
                disabled: false,
                background_color: &[16, 16, 16, 255],
                foreground_color: &[192, 192, 192, 255],
                min: -16384, max: 16383
            },
        ];

        drawing::rect(&mut self.canvas, 0, 0, 256, 240, &[8,  8,  8, 255]);

        for i in 0 .. audio_buffers.len() {
            let y = (i * 40) as u32;
            if !audio_buffers[i].disabled {
                drawing::rect(&mut self.canvas, 0, y, 256, 40, audio_buffers[i].background_color);
                self.draw_waveform(audio_buffers[i].buffer,
                    apu.buffer_index, audio_buffers[i].foreground_color, 
                    0,   y + 8, 256,  32, 
                    audio_buffers[i].min, audio_buffers[i].max);
            }
        }
    }

    pub fn draw_channel_text(&mut self, apu: &ApuState) {
        drawing::text(&mut self.canvas, &self.font, 0, 0, 
            &format!("Pulse 1 - {}{:03X} {}{:02X} {}{:02X}  {:08b}",
            if apu.pulse_1.sweep_enabled {if apu.pulse_1.sweep_negate {"-"} else {"+"}} else {" "}, apu.pulse_1.period_initial,
            if apu.pulse_1.envelope.looping {"L"} else {" "}, apu.pulse_1.envelope.current_volume(),
            if apu.pulse_1.length_counter.length == 0 {"M"} else {" "}, apu.pulse_1.length_counter.length,
            apu.pulse_1.duty),
            &[192,  32,  32, 255]);

        drawing::text(&mut self.canvas, &self.font, 0, 40, 
            &format!("Pulse 2 - {}{:03X} {}{:02X} {}{:02X}  {:08b}",
            if apu.pulse_2.sweep_enabled {if apu.pulse_2.sweep_negate {"-"} else {"+"}} else {" "}, apu.pulse_2.period_initial,
            if apu.pulse_2.envelope.looping {"L"} else {" "}, apu.pulse_2.envelope.current_volume(),
            if apu.pulse_2.length_counter.length == 0 {"M"} else {" "}, apu.pulse_2.length_counter.length,
            apu.pulse_2.duty),
            &[192,  96,  32, 255]);

        drawing::text(&mut self.canvas, &self.font, 0, 80, 
            &format!("Triangle - {:03X}     {}{:02X}        {:02X}", 
            apu.triangle.period_initial,
            if apu.triangle.length_counter.length == 0 {"M"} else {" "}, apu.triangle.length_counter.length,
            apu.triangle.sequence_counter), 
            &[ 32, 192,  32, 255]);

        drawing::text(&mut self.canvas, &self.font, 0, 120, 
            &format!("Noise -    {:03X} {}{:02X} {}{:02X}        {:02X}",
            apu.noise.period_initial,
            if apu.noise.envelope.looping {"L"} else {" "}, apu.noise.envelope.current_volume(),
            if apu.noise.length_counter.length == 0 {"M"} else {" "}, apu.noise.length_counter.length,
            apu.noise.mode),
            &[ 32,  96, 192, 255]);

        drawing::text(&mut self.canvas, &self.font, 0, 160, 
            &format!("DMC -      {:03X}     {}{:02X}  {:04X}  {:02X}",
            apu.dmc.period_initial,
            if apu.triangle.length_counter.length == 0 {"M"} else {" "}, apu.triangle.length_counter.length,
            apu.dmc.starting_address, apu.dmc.output_level),
            &[ 96,  32, 192, 255]);
        
        drawing::text(&mut self.canvas, &self.font, 0, 200, "Final",    &[192, 192, 192, 255]);
    }

    pub fn draw(&mut self, apu: &ApuState) {
        self.draw_audio_samples(apu);
        self.draw_channel_text(apu);
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
            Event::RequestFrame => {self.draw(&runtime.nes.apu)},
            Event::ShowApuWindow => {self.shown = true},
            Event::CloseWindow => {self.shown = false},
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