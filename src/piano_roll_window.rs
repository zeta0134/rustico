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
use rusticnes_core::mmc::mapper::Mapper;

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
    pub time_slices: VecDeque<Vec<ChannelSlice>>,
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
            highest_frequency: 4434.92209563, // ~C#8
            time_slices: VecDeque::new(),

        };
    }

    fn draw_right_white_key(canvas: &mut SimpleBuffer, y: u32, color: &[u8]) {
        drawing::blend_rect(canvas, 248, y + 1, 8, 1, color);
        drawing::blend_rect(canvas, 240, y, 16, 1, color);
    }

    fn draw_center_white_key(canvas: &mut SimpleBuffer, y: u32, color: &[u8]) {
        drawing::blend_rect(canvas, 240, y, 16, 1, color);
        drawing::blend_rect(canvas, 248, y - 1, 8, 1, color);
        drawing::blend_rect(canvas, 248, y + 1, 8, 1, color);
    }

    fn draw_left_white_key(canvas: &mut SimpleBuffer, y: u32, color: &[u8]) {
        drawing::blend_rect(canvas, 248, y - 1, 8, 1, color);
        drawing::blend_rect(canvas, 240, y, 16, 1, color);
    }

    fn draw_black_key(canvas: &mut SimpleBuffer, y: u32, color: &[u8]) {
        drawing::blend_rect(canvas, 241, y - 1, 7, 1, color);
        drawing::blend_rect(canvas, 240, y, 8, 1, color);
        drawing::blend_rect(canvas, 241, y + 1, 7, 1, color);
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
        let black_key_border = [0x10, 0x10, 0x10, 0xFF];

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
        drawing::rect(&mut self.canvas, 240, 0, 1, self.keys * self.key_height, &black_key_border);
    }

    fn draw_key_spot(canvas: &mut SimpleBuffer, slice: &ChannelSlice, key_height: u32) {
        if !slice.visible {return;}

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

        let volume_percent = slice.thickness / 6.0;
        let base_percent = (1.0 - (note_key % 1.0)) * volume_percent;
        let adjacent_percent = (note_key % 1.0) * volume_percent;

        let base_y = base_key * key_height as f64;
        base_color[3] = (base_percent * 255.0) as u8;
        key_drawing_functions[base_key as usize % 12](canvas, base_y as u32, &base_color);

        let adjacent_y = adjacent_key * key_height as f64;
        base_color[3] = (adjacent_percent * 255.0) as u8;
        key_drawing_functions[adjacent_key as usize % 12](canvas, adjacent_y as u32, &base_color);                
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

    pub fn channel_color(channel: &dyn AudioChannelState) -> &[u8] {
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
                &[224, 24, 64, 255]
            } 
        };
    }

    fn slice_from_channel(&self, channel: &dyn AudioChannelState) -> ChannelSlice {
        if !channel.playing() {
            return ChannelSlice::none();
        }

        let y: f64;
        let mut thickness: f64 = 4.0;
        let channel_color = PianoRollWindow::channel_color(channel);
        let color = [
            channel_color[0],
            channel_color[1],
            channel_color[2],
            channel_color[3],
        ];
        

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
                thickness = (index as f64) / (max as f64) * 6.0;
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
        drawing::rect(&mut self.canvas, 0, 0, 256, 240, &[0,0,0,0]);
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