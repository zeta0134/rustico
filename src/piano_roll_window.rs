extern crate sdl2;

use rusticnes_core::apu::ApuState;
use rusticnes_core::apu::PulseChannelState;
use rusticnes_core::apu::TriangleChannelState;
use rusticnes_core::nes::NesState;

use drawing;
use drawing::Font;
use drawing::SimpleBuffer;

#[derive(Clone, Copy)]
pub struct ChannelState {
  pub playing: bool,
  pub frequency: f32,
  pub volume: f32
}

pub struct PianoRollWindow {
  pub buffer: SimpleBuffer,
  pub shown: bool,
  pub font: Font,
  pub last_frame: u32,
  pub last_pulse_1: ChannelState,
  pub last_pulse_2: ChannelState,
  pub last_triangle: ChannelState,
}

// Given a note frequency, returns the y-coordinate within the specified height on a piano
// roll. Assumes the range of a standard 88-key piano.
pub fn frequency_to_coordinate(frequency: f32, height: u32) -> u32 {
  let a1 = (55.0 as f32).ln();
  let c_sharp_8 = (4434.922 as f32).ln();
  let range = c_sharp_8 - a1;
  return ((frequency.ln() - a1) * (height as f32) / range).ceil() as u32;
}

pub fn pulse_frequency(pulse_period: f32) -> f32 {
  let cpu_frequency = 1.789773 * 1024.0 * 1024.0;
  return cpu_frequency / (16.0 * (pulse_period + 1.0));
}

pub fn triangle_frequency(triangle_period: f32) -> f32 {
  let cpu_frequency = 1.789773 * 1024.0 * 1024.0;
  return cpu_frequency / (32.0 * (triangle_period + 1.0));
}

pub fn apply_brightness(color: &[u8], brightness: f32) -> [u8; 4] {
  return [
    (color[0] as f32 * brightness) as u8,
    (color[1] as f32 * brightness) as u8,
    (color[2] as f32 * brightness) as u8,
    255
  ];
}

pub fn pulse_channel_state(pulse: &PulseChannelState) -> ChannelState {
  let volume = pulse.envelope.current_volume();
  let playing = volume != 0 && pulse.length_counter.length > 0;
  let frequency = pulse_frequency(pulse.period_initial as f32);
  return ChannelState {
    playing: playing,
    frequency: frequency,
    volume: volume as f32,
  };
}

pub fn triangle_channel_state(triangle: &TriangleChannelState) -> ChannelState {
  // Note: The triangle channel doesn't have volume control in hardware, it's either
  // on or off. We set 15 here as it's the maximum volume for a pulse channel, for consistency.
  let volume = 15.0;
  let playing = 
      triangle.length_counter.length > 0 && 
      triangle.linear_counter_current != 0 &&
      triangle.period_initial > 2;
  let frequency = triangle_frequency(triangle.period_initial as f32);
  return ChannelState {
    playing: playing,
    frequency: frequency,
    volume: volume
  };
}

pub fn draw_note(buffer: &mut SimpleBuffer, current: ChannelState, old: ChannelState, color: &[u8]) {
  let current_py = frequency_to_coordinate(current.frequency, 380);
  let old_py = frequency_to_coordinate(old.frequency, 380);
  let note_head = current.playing && !old.playing;
  let note_tail = old.playing && !current.playing;
  if current_py >= 5 && current_py < 374 {
    if note_head {
      // Draw the first bit of an outline *before* the note
      drawing::rect(buffer, 
        254, (379 - current_py) + 32 - 1, 1, 7,
        &[0, 0, 0, 255]);
    }
    if current.playing {
      // Outline
      drawing::rect(buffer, 
        255, (379 - current_py) + 32 - 1, 1, 7,
        &[0, 0, 0, 255]); 
      // Note color
      drawing::rect(buffer, 
        255, (379 - current_py) + 32, 1, 5,
        &apply_brightness(color, current.volume / 23.0 + 0.25));
    }
  }
  if old_py >= 5 && old_py < 374 {
    if note_tail {
      // Final Outline
      drawing::rect(buffer, 
        255, (379 - old_py) + 32 - 1, 1, 7,
        &[0, 0, 0, 255]);
    }
  }
}

impl PianoRollWindow {
  pub fn new() -> PianoRollWindow {
    let font = Font::new("assets/8x8_font.png", 8);

    return PianoRollWindow {
      buffer: SimpleBuffer::new(256, 412),
      font: font,
      shown: false,
      last_frame: 0,
      last_pulse_1: ChannelState {playing: false, frequency: 0.0, volume: 0.0},
      last_pulse_2: ChannelState {playing: false, frequency: 0.0, volume: 0.0},
      last_triangle: ChannelState {playing: false, frequency: 0.0, volume: 0.0},
    }
  }

  pub fn shift_playfield_left(&mut self, sx: u32, sy: u32, width: u32, height: u32) {
    for y in sy .. sy + height {
      for x in sx .. sx + width - 1 {
        let right_color = self.buffer.get_pixel(x + 1, y);
        self.buffer.put_pixel(x, y, &right_color);
      }
    }
  }

  pub fn draw_piano_keys(&mut self) {
    // Draw staff lines, roughly in the shape of piano keys.
    // Note, these are highest to lowest:
    let octave_key_colors = [
      [112, 112, 128, 255],
      [112, 112, 128, 255],
      [ 56,  56,  64, 255],
      [112, 112, 128, 255],
      [ 56,  56,  64, 255],
      [112, 112, 128, 255],
      [ 56,  56,  64, 255],
      [112, 112, 128, 255],
      [112, 112, 128, 255],
      [ 56,  56,  64, 255],
      [112, 112, 128, 255],
      [ 56,  56,  64, 255]];

    for key in 0 .. 76 {
      let key_color = octave_key_colors[key % 12];
      let octave = ((key - 1) / 12) % 2;
      let octave_brightness = (octave as f32) * 0.075;
      self.buffer.put_pixel(255, 32 + (key as u32) * 5 + 0, &apply_brightness(&key_color, 0.28 + octave_brightness));
      self.buffer.put_pixel(255, 32 + (key as u32) * 5 + 1, &apply_brightness(&key_color, 0.26 + octave_brightness));
      self.buffer.put_pixel(255, 32 + (key as u32) * 5 + 2, &apply_brightness(&key_color, 0.24 + octave_brightness));
      self.buffer.put_pixel(255, 32 + (key as u32) * 5 + 3, &apply_brightness(&key_color, 0.22 + octave_brightness));
      self.buffer.put_pixel(255, 32 + (key as u32) * 5 + 4, &apply_brightness(&key_color, 0.20 + octave_brightness));
    }
  }

  pub fn draw_channels(&mut self, apu: &ApuState) {
    // Pulse 1
    let current_pulse_1 = pulse_channel_state(&apu.pulse_1);
    draw_note(&mut self.buffer, current_pulse_1, self.last_pulse_1, &[255, 128, 128, 255]);
    self.last_pulse_1 = current_pulse_1;

    // Pulse 2
    let current_pulse_2 = pulse_channel_state(&apu.pulse_2);
    draw_note(&mut self.buffer, current_pulse_2, self.last_pulse_2, &[255, 192, 129, 255]);
    self.last_pulse_2 = current_pulse_2;

    // Triangle
    let current_triangle = triangle_channel_state(&apu.triangle);
    draw_note(&mut self.buffer, current_triangle, self.last_triangle, &[128, 255, 128, 255]);
    self.last_triangle = current_triangle;
  }

  pub fn draw_headers(&mut self) {
    drawing::text(&mut self.buffer, &self.font, 0, 0,  "PULSE 1", &[192,  32,  32, 255]);
    drawing::text(&mut self.buffer, &self.font, 0, 16, &format!("{:.2}", self.last_pulse_1.frequency), &[192,  32,  32, 255]);

    drawing::text(&mut self.buffer, &self.font, 84, 0,  "PULSE 2", &[192,  128,  32, 255]);
    drawing::text(&mut self.buffer, &self.font, 84, 16, &format!("{:.2}", self.last_pulse_2.frequency), &[192,  128,  32, 255]);

    drawing::text(&mut self.buffer, &self.font, 168, 0,  "TRIANGLE", &[32,  192,  32, 255]);
    drawing::text(&mut self.buffer, &self.font, 168, 16, &format!("{:.2}", self.last_triangle.frequency), &[32,  192,  32, 255]);
  }

  pub fn update(&mut self, nes: &mut NesState) {
    if nes.ppu.current_frame == self.last_frame {
      // We're paused! Bail on all drawing.
      return;
    }
    self.last_frame = nes.ppu.current_frame;

    self.shift_playfield_left(0, 32, 256, 380);
    // Clear the header area
    let width = self.buffer.width;
    drawing::rect(&mut self.buffer,   0, 0, width,  32, &[0,0,0,255]);

    self.draw_piano_keys();
    self.draw_channels(&nes.apu);
    self.draw_headers();
  }
}

