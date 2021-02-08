use application::RuntimeState;
use drawing;
use drawing::SimpleBuffer;
use events::Event;
use panel::Panel;

use rusticnes_core::apu::ApuState;
use rusticnes_core::apu::AudioChannelState;
use rusticnes_core::apu::PlaybackRate;
use rusticnes_core::apu::Volume;
use rusticnes_core::apu::Timbre;

use std::collections::VecDeque;


pub struct ChannelSlice {
    pub visible: bool,
    pub y: f64,
    pub thickness: f64,
    pub color: [u8; 4],
}

impl ChannelSlice {
    fn none() -> ChannelSlice {
        return ChannelSlice{
            visible: false,
            y: 0.0,
            thickness: 0.0,
            color: [0,0,0,0]
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
    pub roll: VecDeque<ChannelSlice>,
}

impl PianoRollWindow {
    pub fn new() -> PianoRollWindow {
        return PianoRollWindow {
            canvas: SimpleBuffer::new(256, 240),            
            shown: true,
            keys: 88,
            key_height: 2,
            roll_width: 240,
            lowest_frequency: 27.5, // ~A0
            highest_frequency: 4434.922, // ~C#8
            roll: VecDeque::new(),

        };
    }

    fn draw_right_white_key(&mut self, y: u32, color: &[u8]) {
        drawing::rect(&mut self.canvas, 248, y + 1, 8, 1, color);
        drawing::rect(&mut self.canvas, 240, y, 16, 1, color);
    }

    fn draw_center_white_key(&mut self, y: u32, color: &[u8]) {
        drawing::rect(&mut self.canvas, 240, y, 16, 1, color);
        drawing::rect(&mut self.canvas, 248, y - 1, 8, 3, color);
    }

    fn draw_left_white_key(&mut self, y: u32, color: &[u8]) {
        drawing::rect(&mut self.canvas, 248, y - 1, 8, 1, color);
        drawing::rect(&mut self.canvas, 240, y, 16, 1, color);
    }

    fn draw_black_key(&mut self, y: u32, color: &[u8]) {
        drawing::rect(&mut self.canvas, 241, y - 1, 7, 3, color);
    }

    fn draw_piano_strings(&mut self) {
        let white_string = [0x0C, 0x0C, 0x0C, 0xFF];
        let black_string = [0x06, 0x06, 0x06, 0xFF];

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
            drawing::rect(&mut self.canvas, 0, y, 240, 1, &string_color);
        }
    }

    fn draw_piano_keys(&mut self) {
        let white_key_border = [0x40, 0x40, 0x40, 0xFF];
        let white_key = [0x50, 0x50, 0x50, 0xFF];
        let black_key = [0x00, 0x00, 0x00, 0xFF];
        let black_key_border = [0x18, 0x18, 0x18, 0xFF];

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

        for y in 0 .. self.keys * self.key_height {
            let pixel_index = y % upper_key_pixels.len() as u32;
            drawing::rect(&mut self.canvas, 240, y, 8, 1, &upper_key_pixels[pixel_index as usize]);
            drawing::rect(&mut self.canvas, 248, y, 8, 1, &lower_key_pixels[pixel_index as usize]);
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

    fn slice_from_channel(&self, channel: &dyn AudioChannelState) -> ChannelSlice {
        if !channel.playing() {
            return ChannelSlice::none();
        }

        let mut y: f64 = 0.0;
        let mut thickness: f64 = 5.0;
        let mut color = [0xFF,0xFF,0xFF,0xFF];

        match channel.rate() {
            PlaybackRate::FundamentalFrequency{frequency} => {
                y = self.frequency_to_coordinate(frequency);
            }
            _ => {
                // We don't know how to draw this. Bail.
                return ChannelSlice::none();
            }
        }

        match channel.volume() {
            Some(Volume::VolumeIndex{index, max}) => {
                thickness = (index as f64) / (max as f64) * 8.0;
            },
            None => {}
        }
        /*
        match channel.timbre() {
            Some(Timbre::DutyIndex{index, max}) => {
                
            },
            None => {return ChannelSlice::none()},
            _ => {return ChannelSlice::none()}
        }*/

        return ChannelSlice{
            visible: true,
            y: y,
            thickness: thickness,
            color: color
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
            blended_color[3] = (alpha * 255.0) as u8;
            canvas.blend_pixel(x, top_floor as u32, &blended_color);
            return;
        }
        // Alpha blend the edges
        let top_alpha = 1.0 - (top_edge - top_floor);
        blended_color[3] = (top_alpha * 255.0) as u8;
        canvas.blend_pixel(x, top_floor as u32, &blended_color);

        let bottom_alpha = bottom_edge - bottom_floor;
        blended_color[3] = (bottom_alpha * 255.0) as u8;
        canvas.blend_pixel(x, bottom_floor as u32, &blended_color);

        // If there is any distance at all between the edges, draw a solid color
        // line between them
        for y in (top_floor as u32) + 1 .. bottom_floor as u32 {
            canvas.put_pixel(x, y, &slice.color);
        }
    }

    fn draw_slices(&mut self, num_channels: usize) {
        let mut x = 0;
        let mut channel_index = 0;
        for channel_slice in self.roll.iter() {
            PianoRollWindow::draw_slice(&mut self.canvas, &channel_slice, x);
            channel_index += 1;
            if channel_index >= num_channels {
                channel_index = 0;
                x += 1;
            }
        }
    }

    fn update(&mut self, apu: &ApuState) {
        let channels = apu.channels();
        let channel_len = channels.len();
        for channel in channels {
            self.roll.push_back(self.slice_from_channel(channel));
        }

        while self.roll.len() > channel_len * self.roll_width as usize {
            self.roll.pop_front();
        }
    }

    fn draw(&mut self, apu: &ApuState) {
        drawing::rect(&mut self.canvas, 0, 0, 256, 240, &[0,0,0,0]);
        self.draw_piano_strings();
        self.draw_piano_keys();
        let channels = apu.channels();
        self.draw_slices(channels.len());
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
                    self.update(&runtime.nes.apu);
                }
            },
            Event::RequestFrame => {self.draw(&runtime.nes.apu)},
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