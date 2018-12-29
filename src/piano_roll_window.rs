extern crate sdl2;

use rusticnes_core::apu::ApuState;
use rusticnes_core::nes::NesState;

use sdl2::keyboard::Keycode;

use drawing;
use drawing::Font;
use drawing::SimpleBuffer;

pub struct PianoRollWindow {
  pub buffer: SimpleBuffer,
  pub shown: bool,
  pub font: Font,
}

// Given a note frequency, returns the y-coordinate within the specified height on a piano
// roll. Assumes the range of a standard 88-key piano.
pub fn frequency_to_coordinate(frequency: f32, height: u32) -> u32 {
  let a1 = (55.0 as f32).ln();
  let cS8 = (4434.922 as f32).ln();
  let range = cS8 - a1;
  return ((frequency.ln() - a1) * (height as f32) / range).ceil() as u32;
}

pub fn apply_brightness(color: &[u8], brightness: f32) -> [u8; 4] {
  return [
    (color[0] as f32 * brightness) as u8,
    (color[1] as f32 * brightness) as u8,
    (color[2] as f32 * brightness) as u8,
    255
  ];
}

impl PianoRollWindow {
  pub fn new() -> PianoRollWindow {
    let font = Font::new("assets/8x8_font.png", 8);

    return PianoRollWindow {
      buffer: SimpleBuffer::new(256, 412),
      font: font,
      shown: false,
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

  pub fn update(&mut self, nes: &mut NesState) {
    self.shift_playfield_left(0, 32, 256, 380);
    // Clear!
    let width = self.buffer.width;
    let height = self.buffer.height;
    drawing::rect(&mut self.buffer,   0, 0, width,  32, &[0,0,0,255]);
    drawing::rect(&mut self.buffer, 255, 0,     1, 412, &[0,0,0,255]);

    let cpu_frequency = 1.789773 * 1024.0 * 1024.0;

    let pulse_1_period = nes.apu.pulse_1.period_initial;
    let pulse_1_frequency = cpu_frequency / (16.0 * ((pulse_1_period as f32) + 1.0));
    let pulse_1_y = frequency_to_coordinate(pulse_1_frequency, 380);
    let pulse_1_volume = nes.apu.pulse_1.envelope.current_volume();
    let pulse_1_playing = pulse_1_volume != 0 && nes.apu.pulse_1.length_counter.length > 0;

    let pulse_2_period = nes.apu.pulse_2.period_initial;
    let pulse_2_frequency = cpu_frequency / (16.0 * ((pulse_2_period as f32) + 1.0));
    let pulse_2_y = frequency_to_coordinate(pulse_2_frequency, 380);
    let pulse_2_volume = nes.apu.pulse_2.envelope.current_volume();
    let pulse_2_playing = pulse_2_volume != 0 && nes.apu.pulse_2.length_counter.length > 0;

    let triangle_period = nes.apu.triangle.period_initial;
    let triangle_frequency = cpu_frequency / (32.0 * ((triangle_period as f32) + 1.0));
    let triangle_y = frequency_to_coordinate(triangle_frequency, 380);
    let triangle_playing = 
      nes.apu.triangle.length_counter.length > 0 && 
      nes.apu.triangle.linear_counter_current != 0 &&
      nes.apu.triangle.period_initial > 2;

    drawing::text(&mut self.buffer, &self.font, 0, 0,  "PULSE 1", &[192,  32,  32, 255]);
    drawing::text(&mut self.buffer, &self.font, 0, 8,  &format!("{}", pulse_1_period), &[192,  32,  32, 255]);
    drawing::text(&mut self.buffer, &self.font, 0, 16, &format!("{:.2}", pulse_1_frequency), &[192,  32,  32, 255]);
    drawing::text(&mut self.buffer, &self.font, 0, 24, &format!("{}", pulse_1_playing), &[192,  32,  32, 255]);

    drawing::text(&mut self.buffer, &self.font, 64, 0,  "PULSE 2", &[192,  128,  32, 255]);
    drawing::text(&mut self.buffer, &self.font, 64, 8,  &format!("{}", pulse_2_period), &[192,  128,  32, 255]);
    drawing::text(&mut self.buffer, &self.font, 64, 16, &format!("{:.2}", pulse_2_frequency), &[192,  128,  32, 255]);
    drawing::text(&mut self.buffer, &self.font, 64, 24, &format!("{}", pulse_2_playing), &[192,  128,  32, 255]);

    drawing::text(&mut self.buffer, &self.font, 128, 0,  "TRIANGLE", &[32,  192,  32, 255]);
    drawing::text(&mut self.buffer, &self.font, 128, 8,  &format!("{}", triangle_period), &[32,  192,  32, 255]);
    drawing::text(&mut self.buffer, &self.font, 128, 16, &format!("{:.2}", triangle_frequency), &[32,  192,  32, 255]);
    drawing::text(&mut self.buffer, &self.font, 128, 24, &format!("{}", nes.apu.triangle.length_counter.length), &[32,  192,  32, 255]);

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
      let octave_brightness = ((octave as f32) * 0.075);
      self.buffer.put_pixel(255, 32 + (key as u32) * 5 + 0, &apply_brightness(&key_color, 0.28 + octave_brightness));
      self.buffer.put_pixel(255, 32 + (key as u32) * 5 + 1, &apply_brightness(&key_color, 0.26 + octave_brightness));
      self.buffer.put_pixel(255, 32 + (key as u32) * 5 + 2, &apply_brightness(&key_color, 0.24 + octave_brightness));
      self.buffer.put_pixel(255, 32 + (key as u32) * 5 + 3, &apply_brightness(&key_color, 0.22 + octave_brightness));
      self.buffer.put_pixel(255, 32 + (key as u32) * 5 + 4, &apply_brightness(&key_color, 0.20 + octave_brightness));
    }


    if pulse_1_playing && pulse_1_y >= 4 && pulse_1_y < 375 {
      drawing::rect(&mut self.buffer, 
        255, (379 - pulse_1_y) + 32, 1, 5,
        &apply_brightness(&[255, 32, 32, 255], pulse_2_volume as f32 / 23.0 + 0.25));
    }

    if pulse_2_playing && pulse_2_y >= 4 && pulse_2_y < 375 {
      drawing::rect(&mut self.buffer, 
        255, (379 - pulse_2_y) + 32, 1, 5,
        &apply_brightness(&[255, 192, 32, 255], pulse_2_volume as f32 / 23.0 + 0.25));
    }

    if triangle_playing && triangle_y >= 4 && triangle_y < 375 {
      drawing::rect(&mut self.buffer, 
        255, (379 - triangle_y) + 32, 1, 5,
        &[32, 192, 32, 255]);
    }
  }
}

