extern crate sdl2;

use rusticnes_core::apu::ApuState;
use rusticnes_core::apu::PulseChannelState;
use rusticnes_core::apu::TriangleChannelState;
use rusticnes_core::apu::NoiseChannelState;
use rusticnes_core::apu::DmcState;
use rusticnes_core::nes::NesState;

use drawing;
use drawing::Font;
use drawing::SimpleBuffer;

const NTSC_CPU_FREQUENCY: f32 = 1.789773 * 1024.0 * 1024.0;
const HEADER_HEIGHT: u32 = 32;
const NOTE_FIELD_X: u32 = 0;
const NOTE_FIELD_Y: u32 = HEADER_HEIGHT;
const NOTE_FIELD_SPACING: u32 = 5;
const KEY_HEIGHT: u32 = 5;
const NOTE_COUNT: u32 = 76;
const PERCUSSION_COUNT: u32 = 16;
const NOTE_FIELD_WIDTH: u32 = 512;
const NOTE_FIELD_HEIGHT: u32 = KEY_HEIGHT * NOTE_COUNT;
const PERCUSSION_FIELD_HEIGHT: u32 = KEY_HEIGHT * PERCUSSION_COUNT;
const LOWEST_NOTE_FREQ: f32 = 55.0; // A0
const HIGHEST_NOTE_FREQ: f32 = 4434.922; // C#8

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
  pub last_noise: ChannelState,
  pub last_dmc: ChannelState,
}

// Given a note frequency, returns the y-coordinate within the specified height on a piano roll.
pub fn frequency_to_coordinate(frequency: f32, height: u32) -> u32 {
  let range = HIGHEST_NOTE_FREQ.ln() - LOWEST_NOTE_FREQ.ln();
  return ((frequency.ln() - LOWEST_NOTE_FREQ.ln()) * (height as f32) / range).ceil() as u32;
}

pub fn pulse_frequency(pulse_period: f32) -> f32 {
  return NTSC_CPU_FREQUENCY / (16.0 * (pulse_period + 1.0));
}

pub fn triangle_frequency(triangle_period: f32) -> f32 {
  return NTSC_CPU_FREQUENCY / (32.0 * (triangle_period + 1.0));
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

pub fn noise_channel_state(noise: &NoiseChannelState) -> ChannelState {
  let volume = noise.envelope.current_volume();
  let playing = volume != 0 && noise.length_counter.length > 0;
  // Noise "frequency" is a little funky. For visualization purposes, we're just
  // going to take the value set in hardware and use it directly:
  let frequency = match noise.period_initial {
    4 => 0,
    8 => 1, 
    16 => 2, 
    32 => 3,
    64 => 4,
    96 => 5,
    128 => 6,
    160 => 7,
    202 => 8,
    254 => 9,
    380 => 10,
    508 => 11,
    762 => 12,
    1016 => 13,
    2034 => 14,
    4068 => 15,
    _ => 0
  };

  return ChannelState {
    playing: playing,
    frequency: frequency as f32,
    volume: volume as f32
  };
}

pub fn dmc_channel_state(dmc: &DmcState) -> ChannelState {
  let volume = 15;
  let playing = !dmc.silence_flag;
  // DMC "frequency" is equally funky, since the resulting frequency depends on the sample
  // being played. Again for visualization purposes, we're just going to take the value 
  // set in hardware and use it directly:
  let frequency = match dmc.period_initial {
    54 => 0,
    72 => 1, 
    84 => 2, 
    106 => 3,

    128 => 4,
    142 => 5,
    160 => 6,
    190 => 7,

    214 => 8,
    226 => 9,
    254 => 10,
    286 => 11,

    320 => 12,
    340 => 13,
    380 => 14,
    428 => 15,
    _ => 0
  };

  return ChannelState {
    playing: playing,
    frequency: frequency as f32,
    volume: volume as f32
  };
}

pub fn draw_note(buffer: &mut SimpleBuffer, current: ChannelState, old: ChannelState, color: &[u8]) {
  let current_py = frequency_to_coordinate(current.frequency, NOTE_FIELD_HEIGHT);
  let old_py = frequency_to_coordinate(old.frequency, NOTE_FIELD_HEIGHT);
  let note_head = current.playing && !old.playing;
  let note_tail = old.playing && !current.playing;
  if current_py >= KEY_HEIGHT && current_py < (NOTE_FIELD_HEIGHT - KEY_HEIGHT) {
    if note_head {
      // Draw the first bit of an outline *before* the note
      drawing::rect(buffer, 
        NOTE_FIELD_X + NOTE_FIELD_WIDTH - 2, 
        NOTE_FIELD_HEIGHT - current_py - 1 + NOTE_FIELD_Y - 1, 
        1, 
        KEY_HEIGHT + 2,
        &[0, 0, 0, 255]);
    }
    if current.playing {
      // Outline
      drawing::rect(buffer, 
        NOTE_FIELD_X + NOTE_FIELD_WIDTH - 1, 
        NOTE_FIELD_HEIGHT - current_py - 1 + NOTE_FIELD_Y - 1, 
        1, 
        KEY_HEIGHT + 2,
        &[0, 0, 0, 255]); 
      // Note color
      drawing::rect(buffer, 
        NOTE_FIELD_X + NOTE_FIELD_WIDTH - 1, 
        NOTE_FIELD_HEIGHT - current_py - 1 + NOTE_FIELD_Y, 
        1, 
        KEY_HEIGHT,
        &apply_brightness(color, current.volume / 23.0 + 0.25));
    }
  }
  if old_py >= KEY_HEIGHT && old_py < (NOTE_FIELD_HEIGHT - KEY_HEIGHT) {
    if note_tail {
      // Final Outline
      drawing::rect(buffer, 
        NOTE_FIELD_X + NOTE_FIELD_WIDTH - 1, 
        NOTE_FIELD_HEIGHT - old_py - 1 + NOTE_FIELD_Y - 1, 
        1, 
        KEY_HEIGHT + 2,
        &[0, 0, 0, 255]);
    }
  }
}

pub fn draw_percussion(buffer: &mut SimpleBuffer, current: ChannelState, old: ChannelState, color: &[u8]) {
  let current_py = (current.frequency * (KEY_HEIGHT as f32)) as u32;
  let old_py = (old.frequency * (KEY_HEIGHT as f32)) as u32;
  let note_head = current.playing && !old.playing;
  let note_tail = old.playing && !current.playing;
  if current_py <= (PERCUSSION_FIELD_HEIGHT - KEY_HEIGHT) {
    if note_head {
      // Draw the first bit of an outline *before* the note
      drawing::rect(buffer, 
        NOTE_FIELD_X + NOTE_FIELD_WIDTH - 2, 
        current_py + NOTE_FIELD_Y + NOTE_FIELD_HEIGHT + NOTE_FIELD_SPACING - 1, 
        1, 
        KEY_HEIGHT + 2,
        &[0, 0, 0, 255]);
    }
    if current.playing {
      // Outline
      drawing::rect(buffer, 
        NOTE_FIELD_X + NOTE_FIELD_WIDTH - 1, 
        current_py + NOTE_FIELD_Y + 380 + NOTE_FIELD_SPACING - 1, 
        1, 
        KEY_HEIGHT + 2,
        &[0, 0, 0, 255]); 
      // Note color
      drawing::rect(buffer, 
        NOTE_FIELD_X + NOTE_FIELD_WIDTH - 1, 
        current_py + NOTE_FIELD_Y + NOTE_FIELD_HEIGHT + NOTE_FIELD_SPACING, 
        1, 
        KEY_HEIGHT,
        &apply_brightness(color, current.volume / 23.0 + 0.25));
    }
  }
  if old_py <= 75 {
    if note_tail {
      // Final Outline
      drawing::rect(buffer, 
        NOTE_FIELD_X + NOTE_FIELD_WIDTH - 1, 
        old_py + NOTE_FIELD_Y + NOTE_FIELD_HEIGHT + NOTE_FIELD_SPACING - 1, 
        1, 
        KEY_HEIGHT + 2,
        &[0, 0, 0, 255]);
    }
  }
}

impl PianoRollWindow {
  pub fn new() -> PianoRollWindow {
    let font = Font::from_raw(include_bytes!("assets/8x8_font.png"), 8);

    return PianoRollWindow {
      buffer: SimpleBuffer::new(
        NOTE_FIELD_WIDTH, 
        HEADER_HEIGHT + NOTE_FIELD_HEIGHT + NOTE_FIELD_SPACING + PERCUSSION_FIELD_HEIGHT + NOTE_FIELD_SPACING),
      font: font,
      shown: false,
      last_frame: 0,
      last_pulse_1: ChannelState {playing: false, frequency: 0.0, volume: 0.0},
      last_pulse_2: ChannelState {playing: false, frequency: 0.0, volume: 0.0},
      last_triangle: ChannelState {playing: false, frequency: 0.0, volume: 0.0},
      last_noise: ChannelState {playing: false, frequency: 0.0, volume: 0.0},
      last_dmc: ChannelState {playing: false, frequency: 0.0, volume: 0.0},
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

    for key in 0 .. NOTE_COUNT {
      let key_color = octave_key_colors[(key % 12) as usize];
      let octave = ((key - 1) / 12) % 2;
      let octave_brightness = (octave as f32) * 0.075;
      for i in 0 .. KEY_HEIGHT {
        self.buffer.put_pixel(
          NOTE_FIELD_X + NOTE_FIELD_WIDTH - 1,
          HEADER_HEIGHT + (key as u32) * KEY_HEIGHT + i,
          &apply_brightness(&key_color, 0.30 - ((i as f32) * 0.02) + octave_brightness));
      }
    }
  }

  pub fn draw_percussion_keys(&mut self) {
    let percussion_key_colors = [
      [112, 128, 128, 255],
      [ 56,  64,  64, 255]];

    for key in 0 .. PERCUSSION_COUNT {
      let key_color = percussion_key_colors[(key % 2) as usize];
      for i in 0 .. KEY_HEIGHT {
        self.buffer.put_pixel(
          NOTE_FIELD_X + NOTE_FIELD_WIDTH - 1, 
          HEADER_HEIGHT + NOTE_FIELD_HEIGHT + NOTE_FIELD_SPACING + (key as u32) * KEY_HEIGHT + i, 
          &apply_brightness(&key_color, 0.30 - ((i as f32) * 0.02)));
      }
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

    // DMC ("underneath" noise, so we draw it first)
    let current_dmc = dmc_channel_state(&apu.dmc);
    draw_percussion(&mut self.buffer, current_dmc, self.last_dmc, &[128, 64, 255, 255]);
    self.last_dmc = current_dmc;

    // Noise
    let current_noise = noise_channel_state(&apu.noise);
    if apu.noise.mode == 0 {
      draw_percussion(&mut self.buffer, current_noise, self.last_noise, &[128, 128, 255, 255]);
    } else {
      draw_percussion(&mut self.buffer, current_noise, self.last_noise, &[128, 255, 255, 255]);
    }
    self.last_noise = current_noise;
  
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

    self.shift_playfield_left(NOTE_FIELD_X, NOTE_FIELD_Y, NOTE_FIELD_WIDTH, NOTE_FIELD_HEIGHT + NOTE_FIELD_SPACING + PERCUSSION_FIELD_HEIGHT);
    // Clear the header area
    let width = self.buffer.width;
    drawing::rect(&mut self.buffer,   0, 0, width,  HEADER_HEIGHT, &[0,0,0,255]);

    self.draw_piano_keys();
    self.draw_percussion_keys();
    self.draw_channels(&nes.apu);
    self.draw_headers();
  }
}

