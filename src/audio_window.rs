extern crate sdl2;

use rusticnes_core::apu::ApuState;
use rusticnes_core::nes::NesState;

use sdl2::keyboard::Keycode;

use rusticnes_ui_common::drawing;
use rusticnes_ui_common::drawing::Font;
use rusticnes_ui_common::drawing::SimpleBuffer;

pub fn draw_channel_waveform(imagebuffer: &mut SimpleBuffer, audiobuffer: &[i16], start_index: usize, color: &[u8], x: u32, y: u32, width: u32, height: u32, scale: u32) {
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
      imagebuffer.put_pixel(current_x, y + dy, color);
    }
    for dy in last_y .. current_y {
      imagebuffer.put_pixel(current_x, y + dy, color);
    }
    last_y = current_y;
    imagebuffer.put_pixel(dx, y + current_y, color);
  }
}

pub fn draw_final_waveform(imagebuffer: &mut SimpleBuffer, audiobuffer: &[i16], start_index: usize, color: &[u8], x: u32, y: u32, width: u32, height: u32, scale: u32) {
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
      imagebuffer.put_pixel(current_x, y + dy, color);
    }
    for dy in last_y .. current_y {
      imagebuffer.put_pixel(current_x, y + dy, color);
    }
    last_y = current_y;
    imagebuffer.put_pixel(dx, y + current_y, color);
  }
}

pub fn draw_audio_samples(imagebuffer: &mut SimpleBuffer, font: &Font, apu: &ApuState) {
  // Background
  // TODO: Optimize this somewhat
  for x in 0 .. 256 {
      for y in   0 ..  192 { imagebuffer.put_pixel(x, y, &[8,  8,  8, 255]); }
      if !(apu.pulse_1.debug_disable) {
          for y in   0 ..  32 { imagebuffer.put_pixel(x, y, &[32,  8,  8, 255]); }
      }
      if !(apu.pulse_2.debug_disable) {
          for y in  32 ..  64 { imagebuffer.put_pixel(x, y, &[32, 16,  8, 255]); }
      }
      if !(apu.triangle.debug_disable) {
          for y in  64 ..  96 { imagebuffer.put_pixel(x, y, &[ 8, 32,  8, 255]); }
      }
      if !(apu.noise.debug_disable) {
          for y in  96 .. 128 { imagebuffer.put_pixel(x, y, &[ 8, 16, 32, 255]); }
      }
      if !(apu.dmc.debug_disable) {
          for y in  128 .. 160 { imagebuffer.put_pixel(x, y, &[ 16, 8, 32, 255]); }
      }
      for y in 160 .. 192 { imagebuffer.put_pixel(x, y, &[16, 16, 16, 255]); }
  }

  if !(apu.pulse_1.debug_disable) {
      draw_channel_waveform(imagebuffer, &apu.pulse_1.debug_buffer,
          apu.buffer_index, &[192,  32,  32, 255], 0,   0, 256,  32, 16);
  }
  if !(apu.pulse_2.debug_disable) {
      draw_channel_waveform(imagebuffer, &apu.pulse_2.debug_buffer,
          apu.buffer_index, &[192,  96,  32, 255], 0,  32, 256,  32, 16);
  }
  if !(apu.triangle.debug_disable) {
      draw_channel_waveform(imagebuffer, &apu.triangle.debug_buffer,
          apu.buffer_index, &[32, 192,  32, 255], 0,  64, 256,  32, 16);
  }
  if !(apu.noise.debug_disable) {
      draw_channel_waveform(imagebuffer, &apu.noise.debug_buffer,
          apu.buffer_index, &[32,  96, 192, 255], 0,  96, 256,  32, 16);
  }
  if !(apu.dmc.debug_disable) {
      draw_channel_waveform(imagebuffer, &apu.dmc.debug_buffer,
          apu.buffer_index, &[96,  32, 192, 255], 0, 128, 256,  32, 128);
  }
  draw_final_waveform(imagebuffer, &apu.sample_buffer,
      apu.buffer_index, &[192, 192, 192, 255], 0, 160, 256,  32, 65536);

  drawing::text(imagebuffer, font, 0, 32  - 8, 
    &format!("Pulse 1 - {}{:03X} {}{:02X} {}{:02X}  {:08b}",
    if apu.pulse_1.sweep_enabled {if apu.pulse_1.sweep_negate {"-"} else {"+"}} else {" "}, apu.pulse_1.period_initial,
    if apu.pulse_1.envelope.looping {"L"} else {" "}, apu.pulse_1.envelope.current_volume(),
    if apu.pulse_1.length_counter.length == 0 {"M"} else {" "}, apu.pulse_1.length_counter.length,
    apu.pulse_1.duty),
    &[192,  32,  32, 255]);

  drawing::text(imagebuffer, font, 0, 64  - 8, 
    &format!("Pulse 2 - {}{:03X} {}{:02X} {}{:02X}  {:08b}",
    if apu.pulse_2.sweep_enabled {if apu.pulse_2.sweep_negate {"-"} else {"+"}} else {" "}, apu.pulse_2.period_initial,
    if apu.pulse_2.envelope.looping {"L"} else {" "}, apu.pulse_2.envelope.current_volume(),
    if apu.pulse_2.length_counter.length == 0 {"M"} else {" "}, apu.pulse_2.length_counter.length,
    apu.pulse_2.duty),
    &[192,  96,  32, 255]);

  drawing::text(imagebuffer, font, 0, 96  - 8, 
    &format!("Triangle - {:03X}     {}{:02X}        {:02X}", 
    apu.triangle.period_initial,
    if apu.triangle.length_counter.length == 0 {"M"} else {" "}, apu.triangle.length_counter.length,
    apu.triangle.sequence_counter), 
    &[ 32, 192,  32, 255]);

  drawing::text(imagebuffer, font, 0, 128 - 8, 
    &format!("Noise -    {:03X} {}{:02X} {}{:02X}        {:02X}",
    apu.noise.period_initial,
    if apu.noise.envelope.looping {"L"} else {" "}, apu.noise.envelope.current_volume(),
    if apu.noise.length_counter.length == 0 {"M"} else {" "}, apu.noise.length_counter.length,
    apu.noise.mode),
    &[ 32,  96, 192, 255]);

  drawing::text(imagebuffer, font, 0, 160 - 8, 
    &format!("DMC -      {:03X}     {}{:02X}  {:04X}  {:02X}",
    apu.dmc.period_initial,
    if apu.triangle.length_counter.length == 0 {"M"} else {" "}, apu.triangle.length_counter.length,
    apu.dmc.starting_address, apu.dmc.output_level),
    &[ 96,  32, 192, 255]);
  
  drawing::text(imagebuffer, font, 0, 192 - 8, "Final",    &[192, 192, 192, 255]);
}

pub struct AudioWindow {
  pub buffer: SimpleBuffer,
  pub shown: bool,
  pub font: Font,
}

impl AudioWindow {
  pub fn new() -> AudioWindow {
    let font = Font::from_raw(include_bytes!("assets/8x8_font.png"), 8);

    return AudioWindow {
      buffer: SimpleBuffer::new(256, 192),
      font: font,
      shown: false,
    }
  }

  pub fn update(&mut self, nes: &mut NesState) {
    draw_audio_samples(&mut self.buffer, &self.font, &nes.apu);
  }

  pub fn handle_key_up(&mut self, nes: &mut NesState, key: Keycode) {
    match key {
      Keycode::Num5 => {
        nes.apu.pulse_1.debug_disable = !nes.apu.pulse_1.debug_disable;
      },
      Keycode::Num6 => {
        nes.apu.pulse_2.debug_disable = !nes.apu.pulse_2.debug_disable;
      },
      Keycode::Num7 => {
        nes.apu.triangle.debug_disable = !nes.apu.triangle.debug_disable;
      },
      Keycode::Num8 => {
        nes.apu.noise.debug_disable = !nes.apu.noise.debug_disable;
      },
      Keycode::Num9 => {
        nes.apu.dmc.debug_disable = !nes.apu.dmc.debug_disable;
      },
      _ => ()
    }
  }
}

