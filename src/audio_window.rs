extern crate sdl2;

use rusticnes_core::apu::ApuState;
use rusticnes_core::nes::NesState;

use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

pub struct AudioWindow {
  pub canvas: sdl2::render::WindowCanvas,
  pub screen_buffer: [u8; 256 * 192 * 4],
}

impl AudioWindow {
  pub fn new(sdl_context: &sdl2::Sdl) -> AudioWindow {
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Audio Debugger", 512, 384)
        .position(50, 565)
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let screen_buffer = [0u8; 256 * 192 * 4];

    return AudioWindow {
      canvas: canvas,
      screen_buffer: screen_buffer,
    }
  }

  pub fn put_pixel(&mut self, x: u32, y: u32, color: &[u8]) {
    let index = ((y * 256 + x) * 4) as usize;
    self.screen_buffer[index .. (index + 4)].clone_from_slice(color);
  }

  pub fn draw_waveform(&mut self, audiobuffer: &[u16], start_index: usize, color: &[u8], x: u32, y: u32, width: u32, height: u32, scale: u32) {
    let mut last_y = 0;
    for dx in x .. (x + width) {
        let sample_index = (start_index + dx as usize) % audiobuffer.len();
        let sample = audiobuffer[sample_index];
        let current_x = dx as u32;
        let mut current_y = ((sample as u32 * height) / scale) as u32;
        if current_y >= height {
            current_y = height - 1;
        }
        for dy in current_y .. last_y {
            self.put_pixel(current_x, y + dy, color);
        }
        for dy in last_y .. current_y {
            self.put_pixel(current_x, y + dy, color);
        }
        last_y = current_y;
        self.put_pixel(dx, y + current_y, color);
    }
  }

  pub fn draw_audio_samples(&mut self, apu: &ApuState) {
    // Draw audio samples! What could possibly go wrong?
    // Do we need to clear this manually?
    //*

    // Background
    for x in 0 .. 256 {
        for y in   0 ..  192 { self.put_pixel(x, y, &[8,  8,  8, 255]); }
        if !(apu.pulse_1.debug_disable) {
            for y in   0 ..  32 { self.put_pixel(x, y, &[32,  8,  8, 255]); }
        }
        if !(apu.pulse_2.debug_disable) {
            for y in  32 ..  64 { self.put_pixel(x, y, &[32, 16,  8, 255]); }
        }
        if !(apu.triangle.debug_disable) {
            for y in  64 ..  96 { self.put_pixel(x, y, &[ 8, 32,  8, 255]); }
        }
        if !(apu.noise.debug_disable) {
            for y in  96 .. 128 { self.put_pixel(x, y, &[ 8, 16, 32, 255]); }
        }
        if !(apu.dmc.debug_disable) {
            for y in  128 .. 160 { self.put_pixel(x, y, &[ 16, 8, 32, 255]); }
        }
        for y in 160 .. 192 { self.put_pixel(x, y, &[16, 16, 16, 255]); }
    }

    if !(apu.pulse_1.debug_disable) {
        self.draw_waveform(&apu.pulse_1.debug_buffer,
            apu.buffer_index, &[192,  32,  32, 255], 0,   0, 256,  32, 16);
    }
    if !(apu.pulse_2.debug_disable) {
        self.draw_waveform(&apu.pulse_2.debug_buffer,
            apu.buffer_index, &[192,  96,  32, 255], 0,  32, 256,  32, 16);
    }
    if !(apu.triangle.debug_disable) {
        self.draw_waveform(&apu.triangle.debug_buffer,
            apu.buffer_index, &[32, 192,  32, 255], 0,  64, 256,  32, 16);
    }
    if !(apu.noise.debug_disable) {
        self.draw_waveform(&apu.noise.debug_buffer,
            apu.buffer_index, &[32,  96, 192, 255], 0,  96, 256,  32, 16);
    }
    if !(apu.dmc.debug_disable) {
        self.draw_waveform(&apu.dmc.debug_buffer,
            apu.buffer_index, &[96,  32, 192, 255], 0, 128, 256,  32, 128);
    }
    self.draw_waveform(&apu.sample_buffer,
        apu.buffer_index, &[192, 192, 192, 255], 0, 160, 256,  32, 16384);
  }

  pub fn update(&mut self, nes: &mut NesState) {
    self.draw_audio_samples(&nes.apu);
  }

  pub fn draw(&mut self) {
    let texture_creator = self.canvas.texture_creator();
    let mut texture = texture_creator.create_texture(PixelFormatEnum::ABGR8888, TextureAccess::Streaming, 256, 192).unwrap();
      
    self.canvas.set_draw_color(Color::RGB(255, 255, 255));
    let _ = texture.update(None, &self.screen_buffer, 256 * 4);
    let _ = self.canvas.copy(&texture, None, None);

    self.canvas.present();
  }

  pub fn handle_event(&mut self, _: &mut NesState, _: &sdl2::event::Event) {
    
  }
}

