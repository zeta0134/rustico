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
            canvas: SimpleBuffer::new(256, 192),
            font: font,
            shown: false,
        };
    }

    pub fn draw_channel_waveform(&mut self, audiobuffer: &[i16], start_index: usize, color: &[u8], x: u32, y: u32, width: u32, height: u32, scale: u32) {
        let mut last_y = 0;
        for dx in x .. (x + width) {
            let sample_index = (start_index + dx as usize) % audiobuffer.len();
            let sample = audiobuffer[sample_index];
            let current_x = dx as u32;
            let mut current_y = ((sample as u64 * height as u64) / scale as u64) as u32;
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

    pub fn draw_final_waveform(&mut self, audiobuffer: &[i16], start_index: usize, color: &[u8], x: u32, y: u32, width: u32, height: u32, scale: u32) {
        let mut last_y = 0;
        for dx in x .. (x + width) {
            let sample_index = (start_index + dx as usize) % audiobuffer.len();
            let sample = audiobuffer[sample_index];
            let current_x = dx as u32;
            let mut current_y = (((sample as i64 + (scale as i64 / 2)) * height as i64) / scale as i64) as u32;
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
        for x in 0 .. 256 {
            for y in   0 ..  192 { self.canvas.put_pixel(x, y, &[8,  8,  8, 255]); }
            if !(apu.pulse_1.debug_disable) {
                for y in   0 ..  32 { self.canvas.put_pixel(x, y, &[32,  8,  8, 255]); }
            }
            if !(apu.pulse_2.debug_disable) {
                for y in  32 ..  64 { self.canvas.put_pixel(x, y, &[32, 16,  8, 255]); }
            }
            if !(apu.triangle.debug_disable) {
                for y in  64 ..  96 { self.canvas.put_pixel(x, y, &[ 8, 32,  8, 255]); }
            }
            if !(apu.noise.debug_disable) {
                for y in  96 .. 128 { self.canvas.put_pixel(x, y, &[ 8, 16, 32, 255]); }
            }
            if !(apu.dmc.debug_disable) {
                for y in  128 .. 160 { self.canvas.put_pixel(x, y, &[ 16, 8, 32, 255]); }
            }
            for y in 160 .. 192 { self.canvas.put_pixel(x, y, &[16, 16, 16, 255]); }
        }

        if !(apu.pulse_1.debug_disable) {
            self.draw_channel_waveform(&apu.pulse_1.debug_buffer,
                apu.buffer_index, &[192,  32,  32, 255], 0,   0, 256,  32, 16);
        }
        if !(apu.pulse_2.debug_disable) {
            self.draw_channel_waveform(&apu.pulse_2.debug_buffer,
                apu.buffer_index, &[192,  96,  32, 255], 0,  32, 256,  32, 16);
        }
        if !(apu.triangle.debug_disable) {
            self.draw_channel_waveform(&apu.triangle.debug_buffer,
                apu.buffer_index, &[32, 192,  32, 255], 0,  64, 256,  32, 16);
        }
        if !(apu.noise.debug_disable) {
            self.draw_channel_waveform(&apu.noise.debug_buffer,
                apu.buffer_index, &[32,  96, 192, 255], 0,  96, 256,  32, 16);
        }
        if !(apu.dmc.debug_disable) {
            self.draw_channel_waveform(&apu.dmc.debug_buffer,
                apu.buffer_index, &[96,  32, 192, 255], 0, 128, 256,  32, 128);
        }
        self.draw_final_waveform(&apu.sample_buffer,
            apu.buffer_index, &[192, 192, 192, 255], 0, 160, 256,  32, 65536);

        drawing::text(&mut self.canvas, &self.font, 0, 32  - 8, 
            &format!("Pulse 1 - {}{:03X} {}{:02X} {}{:02X}  {:08b}",
            if apu.pulse_1.sweep_enabled {if apu.pulse_1.sweep_negate {"-"} else {"+"}} else {" "}, apu.pulse_1.period_initial,
            if apu.pulse_1.envelope.looping {"L"} else {" "}, apu.pulse_1.envelope.current_volume(),
            if apu.pulse_1.length_counter.length == 0 {"M"} else {" "}, apu.pulse_1.length_counter.length,
            apu.pulse_1.duty),
            &[192,  32,  32, 255]);

        drawing::text(&mut self.canvas, &self.font, 0, 64  - 8, 
            &format!("Pulse 2 - {}{:03X} {}{:02X} {}{:02X}  {:08b}",
            if apu.pulse_2.sweep_enabled {if apu.pulse_2.sweep_negate {"-"} else {"+"}} else {" "}, apu.pulse_2.period_initial,
            if apu.pulse_2.envelope.looping {"L"} else {" "}, apu.pulse_2.envelope.current_volume(),
            if apu.pulse_2.length_counter.length == 0 {"M"} else {" "}, apu.pulse_2.length_counter.length,
            apu.pulse_2.duty),
            &[192,  96,  32, 255]);

        drawing::text(&mut self.canvas, &self.font, 0, 96  - 8, 
            &format!("Triangle - {:03X}     {}{:02X}        {:02X}", 
            apu.triangle.period_initial,
            if apu.triangle.length_counter.length == 0 {"M"} else {" "}, apu.triangle.length_counter.length,
            apu.triangle.sequence_counter), 
            &[ 32, 192,  32, 255]);

        drawing::text(&mut self.canvas, &self.font, 0, 128 - 8, 
            &format!("Noise -    {:03X} {}{:02X} {}{:02X}        {:02X}",
            apu.noise.period_initial,
            if apu.noise.envelope.looping {"L"} else {" "}, apu.noise.envelope.current_volume(),
            if apu.noise.length_counter.length == 0 {"M"} else {" "}, apu.noise.length_counter.length,
            apu.noise.mode),
            &[ 32,  96, 192, 255]);

        drawing::text(&mut self.canvas, &self.font, 0, 160 - 8, 
            &format!("DMC -      {:03X}     {}{:02X}  {:04X}  {:02X}",
            apu.dmc.period_initial,
            if apu.triangle.length_counter.length == 0 {"M"} else {" "}, apu.triangle.length_counter.length,
            apu.dmc.starting_address, apu.dmc.output_level),
            &[ 96,  32, 192, 255]);
        
        drawing::text(&mut self.canvas, &self.font, 0, 192 - 8, "Final",    &[192, 192, 192, 255]);
    }

    pub fn draw(&mut self, apu: &ApuState) {
        self.draw_audio_samples(apu);
    }
}



impl Panel for ApuWindow {
    fn title(&self) -> &str {
        return "APU Surfing";
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