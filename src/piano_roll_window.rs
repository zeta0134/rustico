use application::RuntimeState;
use drawing;
use drawing::Color;
use drawing::SimpleBuffer;
use events::Event;
use panel::Panel;

use rusticnes_core::apu::ApuState;
use rusticnes_core::apu::AudioChannelState;
use rusticnes_core::apu::PlaybackRate;
use rusticnes_core::apu::Timbre;
use rusticnes_core::mmc::mapper::Mapper;

use std::collections::VecDeque;

pub enum NoteType {
    Frequency,
    Noise,
    Waveform
}

pub struct ChannelSlice {
    pub visible: bool,
    pub y: f64,
    pub thickness: f64,
    pub color: Color,
    pub note_type: NoteType,

}

impl ChannelSlice {
    fn none() -> ChannelSlice {
        return ChannelSlice{
            visible: false,
            y: 0.0,
            thickness: 0.0,
            color: Color::rgb(0,0,0),
            note_type: NoteType::Frequency,
        };
    }
}

pub struct PianoRollWindow {
    pub canvas: SimpleBuffer,
    pub shown: bool,
    pub keys: u32,
    pub key_height: u32,
    pub roll_width: u32,
    pub lowest_frequency: f64,
    pub highest_frequency: f64,
    pub time_slices: VecDeque<Vec<ChannelSlice>>,
}

impl PianoRollWindow {
    pub fn new() -> PianoRollWindow {
        return PianoRollWindow {
            canvas: SimpleBuffer::new(256, 240),            
            shown: false,
            keys: 109,
            key_height: 2,
            roll_width: 240,
            lowest_frequency: 8.176, // ~C0
            highest_frequency: 4434.92209563, // ~C#8
            time_slices: VecDeque::new(),

        };
    }

    fn draw_right_white_key(canvas: &mut SimpleBuffer, y: u32, color: Color) {
        drawing::blend_rect(canvas, 248, y + 1, 8, 1, color);
        drawing::blend_rect(canvas, 240, y, 16, 1, color);
    }

    fn draw_center_white_key(canvas: &mut SimpleBuffer, y: u32, color: Color) {
        drawing::blend_rect(canvas, 240, y, 16, 1, color);
        drawing::blend_rect(canvas, 248, y - 1, 8, 1, color);
        drawing::blend_rect(canvas, 248, y + 1, 8, 1, color);
    }

    fn draw_left_white_key(canvas: &mut SimpleBuffer, y: u32, color: Color) {
        drawing::blend_rect(canvas, 248, y - 1, 8, 1, color);
        drawing::blend_rect(canvas, 240, y, 16, 1, color);
    }

    fn draw_black_key(canvas: &mut SimpleBuffer, y: u32, color: Color) {
        drawing::blend_rect(canvas, 241, y - 1, 7, 1, color);
        drawing::blend_rect(canvas, 240, y, 8, 1, color);
        drawing::blend_rect(canvas, 241, y + 1, 7, 1, color);
    }

    fn draw_speaker_key(canvas: &mut SimpleBuffer, color: Color) {
        drawing::blend_rect(canvas, 240, 228, 2, 1, color);
        drawing::blend_rect(canvas, 242, 226, 3, 5, color);
        drawing::blend_rect(canvas, 245, 225, 1, 7, color);
        drawing::blend_rect(canvas, 246, 224, 1, 9, color);
        drawing::blend_rect(canvas, 247, 223, 1, 11, color);
        drawing::blend_rect(canvas, 248, 222, 1, 13, color);
        drawing::blend_rect(canvas, 250, 226, 1, 5, color);
        drawing::blend_rect(canvas, 252, 224, 1, 9, color);
    }

    fn draw_piano_strings(&mut self) {
        let white_string = Color::rgb(0x0C, 0x0C, 0x0C);
        let black_string = Color::rgb(0x06, 0x06, 0x06);

        let string_colors = [
            white_string, //C
            white_string, //B
            black_string, //Bb
            white_string, //A
            black_string, //Ab
            white_string, //G
            black_string, //Gb
            white_string, //F
            white_string, //E
            black_string, //Eb
            white_string, //D
            black_string, //Db
        ];

        for i in 0 .. self.keys {
            let string_color = string_colors[(i % 12) as usize];
            let y = i * self.key_height;
            drawing::rect(&mut self.canvas, 0, y, 240, 1, string_color);
        }

        // Draw one extra string for the waveform display
        drawing::rect(&mut self.canvas, 0, 228, 240, 1, black_string);
    }

    fn draw_piano_keys(&mut self) {
        let white_key_border = Color::rgb(0x1C, 0x1C, 0x1C);
        let white_key = Color::rgb(0x20, 0x20, 0x20);
        let black_key = Color::rgb(0x00, 0x00, 0x00);
        let top_edge = Color::rgb(0x0A, 0x0A, 0x0A);

        let upper_key_pixels = [
          white_key, // C
          white_key_border, 
          white_key, // B
          black_key, black_key, black_key, // Bb
          white_key, // A
          black_key, black_key, black_key, // Ab
          white_key, // G
          black_key, black_key, black_key, // Gb
          white_key, // F
          white_key_border,
          white_key, // E
          black_key, black_key, black_key, // Eb
          white_key, // D
          black_key, black_key, black_key, // Db
        ];

        let lower_key_pixels = [
          white_key, // C (bottom half)
          white_key_border,
          white_key, white_key, // B
          white_key_border, 
          white_key, white_key, white_key, // A
          white_key_border, 
          white_key, white_key, white_key, // G
          white_key_border,
          white_key, white_key, // F
          white_key_border,
          white_key, white_key, // E
          white_key_border, 
          white_key, white_key, white_key, // D
          white_key_border,
          white_key, // C (top half)
        ];

        drawing::rect(&mut self.canvas, 240, 0, 16, 240, top_edge);
        for y in 0 .. self.keys * self.key_height {
            let pixel_index = y % upper_key_pixels.len() as u32;
            drawing::rect(&mut self.canvas, 240, y, 8, 1, upper_key_pixels[pixel_index as usize]);
            drawing::rect(&mut self.canvas, 248, y, 8, 1, lower_key_pixels[pixel_index as usize]);
        }
        drawing::rect(&mut self.canvas, 240, 0, 1, 240, top_edge);
        PianoRollWindow::draw_speaker_key(&mut self.canvas, black_key);
    }

    fn draw_key_spot(canvas: &mut SimpleBuffer, slice: &ChannelSlice, key_height: u32) {
        if !slice.visible {return;}

        match slice.note_type {
            NoteType::Waveform => {
                let mut base_color = slice.color;
                let volume_percent = slice.thickness / 6.0;
                base_color.set_alpha((volume_percent * 255.0) as u8);
                PianoRollWindow::draw_speaker_key(canvas, base_color);
            },
            _ => {
                let key_drawing_functions = [
                    PianoRollWindow::draw_left_white_key,   //C
                    PianoRollWindow::draw_right_white_key,  //B
                    PianoRollWindow::draw_black_key,        //Bb
                    PianoRollWindow::draw_center_white_key, //A
                    PianoRollWindow::draw_black_key,        //Ab
                    PianoRollWindow::draw_center_white_key, //G
                    PianoRollWindow::draw_black_key,        //Gb
                    PianoRollWindow::draw_left_white_key,   //F
                    PianoRollWindow::draw_right_white_key,  //E
                    PianoRollWindow::draw_black_key,        //Eb
                    PianoRollWindow::draw_center_white_key, //D
                    PianoRollWindow::draw_black_key,        //Db
                ];

                let mut base_color = slice.color;

                let note_key = ((slice.y + 1.5) / key_height as f64) - 1.0;
                let base_key = note_key.floor();
                let adjacent_key = note_key.ceil();

                let base_volume_percent = slice.thickness / 6.0;
                let adjusted_volume_percent = 0.05 + base_volume_percent * 0.95;
                let base_percent = (1.0 - (note_key % 1.0)) * adjusted_volume_percent;
                let adjacent_percent = (note_key % 1.0) * adjusted_volume_percent;

                let base_y = base_key * key_height as f64;
                base_color.set_alpha((base_percent * 255.0) as u8);
                key_drawing_functions[base_key as usize % 12](canvas, base_y as u32, base_color);

                let adjacent_y = adjacent_key * key_height as f64;
                base_color.set_alpha((adjacent_percent * 255.0) as u8);
                key_drawing_functions[adjacent_key as usize % 12](canvas, adjacent_y as u32, base_color);
            }
        }        
    }

    fn frequency_to_coordinate(&self, note_frequency: f64) -> f64 {
        let highest_log = self.highest_frequency.ln();
        let lowest_log = self.lowest_frequency.ln();
        let range = highest_log - lowest_log;
        let note_log = note_frequency.ln();
        let piano_roll_height = (self.keys * self.key_height) as f64;
        let coordinate = (note_log - lowest_log) * piano_roll_height / range;
        return piano_roll_height - coordinate - 1.5;
    }

    pub fn channel_colors(channel: &dyn AudioChannelState) -> Vec<Color> {
        if channel.muted() {
            return vec!(Color::rgb(32, 32, 32));
        }
        return match channel.chip().as_str() {
            "2A03" => match channel.name().as_str() {
                "Pulse 1"  => {
                    vec!(                        
                        Color::rgb(0xFF, 0xA0, 0xA0), // 12.5
                        Color::rgb(0xFF, 0x40, 0xFF), // 25
                        Color::rgb(0xFF, 0x40, 0x40), // 50
                        Color::rgb(0xFF, 0x40, 0xFF)) // 75 (same as 25)
                },
                "Pulse 2"  => {
                    vec!(                        
                        Color::rgb(0xFF, 0xE0, 0xA0), // 12.5
                        Color::rgb(0xFF, 0xC0, 0x40), // 25
                        Color::rgb(0xFF, 0xFF, 0x40), // 50
                        Color::rgb(0xFF, 0xC0, 0x40)) // 75 (same as 25)
                },
                "Triangle" => {vec!(Color::rgb(0x40, 0xFF, 0x40))},
                "Noise"    => {
                    vec!(
                        Color::rgb(192, 192, 192),
                        Color::rgb(192, 255, 255)
                    )
                },
                "DMC"      => {vec!(Color::rgb(96,  32, 192))},
                _ => {vec!(Color::rgb(192,  192, 192))} // default, should be unreachable
            },
            "MMC5" => match channel.name().as_str() {
                "Pulse 1" => {vec!(Color::rgb(224, 24, 64))},
                "Pulse 2" => {vec!(Color::rgb(180, 12, 40))},
                "PCM" => {vec!(Color::rgb(224, 24, 64))},
                _ => {vec!(Color::rgb(192,  192, 192))} // default, should be unreachable
            },
            "YM2149F" => match channel.name().as_str() {
                "A" => {vec!(Color::rgb(32, 144, 204))},
                "B" => {vec!(Color::rgb(24, 104, 228))},
                "C" => {vec!(Color::rgb(16, 64, 248))},
                _ => {vec!(Color::rgb(192,  192, 192))} // default, should be unreachable
            },
            "VRC6" => match channel.name().as_str() {
                "Pulse 1"  => {
                    vec!(                        
                        Color::rgb(0xf2, 0xbb, 0xd8), // 6.25%
                        Color::rgb(0xdb, 0xa0, 0xbf), // 12.5%
                        Color::rgb(0xc4, 0x86, 0xa6), // 18.75%
                        Color::rgb(0xad, 0x6c, 0x8d), // 25%
                        Color::rgb(0x97, 0x51, 0x74), // 31.25%
                        Color::rgb(0x80, 0x37, 0x5b), // 37.5%
                        Color::rgb(0x69, 0x1d, 0x42), // 43.75%
                        Color::rgb(0x53, 0x03, 0x2a)) // 50%
                },
                "Pulse 2"  => {
                    vec!(                        
                        Color::rgb(0xe8, 0xa7, 0xe7), // 6.25%
                        Color::rgb(0xd2, 0x8f, 0xd1), // 12.5%
                        Color::rgb(0xbd, 0x78, 0xbb), // 18.75%
                        Color::rgb(0xa7, 0x60, 0xa6), // 25%
                        Color::rgb(0x92, 0x49, 0x90), // 31.25%
                        Color::rgb(0x7c, 0x31, 0x7b), // 37.5%
                        Color::rgb(0x67, 0x1a, 0x65), // 43.75%
                        Color::rgb(0x52, 0x03, 0x50)) // 50%
                },
                "Sawtooth" => {
                    vec!(
                        Color::rgb(0x07, 0x7d, 0x5a),   // Normal
                        Color::rgb(0x9f, 0xb8, 0xed))
                }, // Distortion
                _ => {vec!(Color::rgb(192,  192, 192))} // default, should be unreachable
            },
            _ => {
                // Unknown expansion audio, we'll default it to grey
                vec!(Color::rgb(224, 224, 224))
            } 
        };
    }

    fn slice_from_channel(&self, channel: &dyn AudioChannelState) -> ChannelSlice {
        if !channel.playing() {
            return ChannelSlice::none();
        }

        let y: f64;
        let thickness: f64 = channel.amplitude() * 6.0;
        let colors = PianoRollWindow::channel_colors(channel);
        let mut color = colors[0]; // default to the first color
        let note_type: NoteType;

        match channel.rate() {
            PlaybackRate::FundamentalFrequency{frequency} => {
                y = self.frequency_to_coordinate(frequency);
                note_type = NoteType::Frequency;
            },
            PlaybackRate::LfsrRate{index, max} => {
                note_type = NoteType::Noise;

                // Arbitrarily map all noise frequencies to 16 "strings" since this is what the
                // base 2A03 uses. Accuracy is much less important here.
                let string_coord = (index as f64 / (max + 1) as f64) * 16.0;
                let key_offset = string_coord * self.key_height as f64;

                // Hrm. Experiment: let's try basing it around C0, so the notes match what you might
                // enter in FamiTracker
                let base_freq = 8.176;
                let base_y = self.frequency_to_coordinate(base_freq);
                y = base_y - key_offset;

            },
            PlaybackRate::SampleRate{frequency: _} => {
                y = 228.5;
                note_type = NoteType::Waveform;
            }
        }
        
        match channel.timbre() {
            Some(Timbre::DutyIndex{index, max}) => {
                let weight = index as f64 / (max + 1) as f64;
                color = drawing::apply_gradient(colors, weight);
            },
            Some(Timbre::LsfrMode{index, max}) => {
                let weight = index as f64 / (max + 1) as f64;
                color = drawing::apply_gradient(colors, weight);  
            },
            None => {},
            _ => {}
        }

        return ChannelSlice{
            visible: true,
            y: y,
            thickness: thickness,
            color: color,
            note_type: note_type
        };
    }

    fn draw_slice(canvas: &mut SimpleBuffer, slice: &ChannelSlice, x: u32) {
        if !slice.visible {return;}
        let top_edge = slice.y - (slice.thickness / 2.0);
        let bottom_edge = slice.y + (slice.thickness / 2.0);
        let top_floor = top_edge.floor();
        let bottom_floor = bottom_edge.floor();

        // sanity range check:
        if top_edge < 0.0 || bottom_edge > canvas.height as f64 {
            return;
        }

        let mut blended_color = slice.color;
        if top_floor == bottom_floor {
            // Special case: alpha here will be related to their distance. Draw one
            // blended point and exit
            let alpha = bottom_edge - top_edge;
            blended_color.set_alpha((alpha * 255.0) as u8);
            canvas.blend_pixel(x, top_floor as u32, blended_color);
            return;
        }
        // Alpha blend the edges
        let top_alpha = 1.0 - (top_edge - top_floor);
        blended_color.set_alpha((top_alpha * 255.0) as u8);
        canvas.blend_pixel(x, top_floor as u32, blended_color);

        let bottom_alpha = bottom_edge - bottom_floor;
        blended_color.set_alpha((bottom_alpha * 255.0) as u8);
        canvas.blend_pixel(x, bottom_floor as u32, blended_color);

        // If there is any distance at all between the edges, draw a solid color
        // line between them
        for y in (top_floor as u32) + 1 .. bottom_floor as u32 {
            canvas.put_pixel(x, y, slice.color);
        }
    }

    fn draw_slices(&mut self) {
        let mut x = 239;
        for channel_slice in self.time_slices.iter() {
            for note in channel_slice.iter() {
                PianoRollWindow::draw_slice(&mut self.canvas, &note, x);    
            }
            if x == 0 {
                return; //bail! don't draw offscreen
            }
            x -= 1;
        }
    }

    fn draw_key_spots(&mut self) {
        for note in self.time_slices.front().unwrap_or(&Vec::new()) {
            PianoRollWindow::draw_key_spot(&mut self.canvas, &note, self.key_height);
        }
    }

    fn update(&mut self, apu: &ApuState, mapper: &dyn Mapper) {
        let mut channels = apu.channels();
        channels.extend(mapper.channels());
        let mut frame_notes: Vec<ChannelSlice> = Vec::new();
        for channel in channels {
            frame_notes.push(self.slice_from_channel(channel));
        }
        self.time_slices.push_front(frame_notes);

        while self.time_slices.len() > self.roll_width as usize {
            self.time_slices.pop_back();
        }
    }

    fn draw(&mut self) {
        drawing::rect(&mut self.canvas, 0, 0, 256, 240, Color::rgb(0,0,0));
        self.draw_piano_strings();
        self.draw_piano_keys();
        self.draw_slices();
        self.draw_key_spots();
    }
}

impl Panel for PianoRollWindow {
    fn title(&self) -> &str {
        return "Piano Roll";
    }

    fn shown(&self) -> bool {
        return self.shown;
    }

    fn scale_factor(&self) -> u32 {
        return 3;
    }

    fn handle_event(&mut self, runtime: &RuntimeState, event: Event) -> Vec<Event> {
        match event {
            Event::Update => {
                if runtime.running {
                    self.update(&runtime.nes.apu, &*runtime.nes.mapper);
                }
            },
            Event::RequestFrame => {self.draw()},
            Event::ShowPianoRollWindow => {self.shown = true},
            Event::CloseWindow => {self.shown = false},
            _ => {}
        }
        return Vec::<Event>::new();
    }
    
    fn active_canvas(&self) -> &SimpleBuffer {
        return &self.canvas;
    }
}