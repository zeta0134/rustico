extern crate sdl2;

use rusticnes_core::mmc::mapper::Mapper;
use rusticnes_core::nes::NesState;
use rusticnes_core::ppu;
use rusticnes_core::palettes::NTSC_PAL;

use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::TextureAccess;

use drawing::SimpleBuffer;

pub fn draw_tile(mapper: &mut Mapper, pattern_address: u16, tile_index: u16, buffer: &mut SimpleBuffer, dx: u32, dy: u32, palette: &[u8]) {
  for py in 0 .. 8 {
    let tile_address = pattern_address + tile_index * 16 + py;
    let mut tile_low  = mapper.debug_read_byte(tile_address);
    let mut tile_high = mapper.debug_read_byte(tile_address + 8);
    for px in 0 .. 8 {
      let palette_index = (tile_low & 0x1) + ((tile_high & 0x1) << 1);
      tile_low = tile_low >> 1;
      tile_high = tile_high >> 1;
      buffer.put_pixel(
        dx + (7 - px as u32), 
        dy + (py as u32), &[
          palette[(palette_index * 4 + 0) as usize],
          palette[(palette_index * 4 + 1) as usize],
          palette[(palette_index * 4 + 2) as usize],
          255]);
    }
  }
}

pub fn generate_chr_pattern(mapper: &mut Mapper, pattern_address: u16, buffer: &mut SimpleBuffer, dx: u32, dy: u32) {
  let debug_palette: [u8; 4*4] = [
    255, 255, 255, 255,
    192, 192, 192, 255,
    128, 128, 128, 255,
      0,   0,   0, 255];
  for x in 0 .. 16 {
    for y in 0 .. 16 {
      let tile_index = y * 16 + x;
      draw_tile(mapper, pattern_address, tile_index as u16, buffer, 
        dx + x * 8, dy + y * 8, &debug_palette);
    }
  }
}

pub struct VramWindow {
  pub canvas: sdl2::render::WindowCanvas,
  pub buffer: SimpleBuffer,
  pub shown: bool,
  pub palette_cache: [[u8; 4*4]; 4*2],
}

impl VramWindow {
  pub fn new(sdl_context: &sdl2::Sdl) -> VramWindow {
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("VRAM Debugger", 768, 512)
        .position(570, 50)
        .hidden()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    return VramWindow {
      canvas: canvas,
      buffer: SimpleBuffer::new(768, 512),
      shown: false,
      palette_cache: [[0u8; 4*4]; 4*2]
    }
  }

  pub fn update_palette_cache(&mut self, nes: &mut NesState) {
    // Initialize all palette colors with a straight copy
    for p in 0 .. 8 {
      for i in 0 .. 4 {
        let palette_color = nes.ppu.passively_read_byte(&mut *nes.mapper, 0x3F00 + p * 4 + i) as usize * 3;
        self.palette_cache[p as usize][i as usize * 4 + 0] = NTSC_PAL[palette_color + 0];
        self.palette_cache[p as usize][i as usize * 4 + 1] = NTSC_PAL[palette_color + 1];
        self.palette_cache[p as usize][i as usize * 4 + 2] = NTSC_PAL[palette_color + 2];
        self.palette_cache[p as usize][i as usize * 4 + 3] = 255;
      }
    }

    // Override the background colors with the universal background color:
    for p in 1 .. 8 {
      self.palette_cache[p][0] = self.palette_cache[0][0];
      self.palette_cache[p][1] = self.palette_cache[0][1];
      self.palette_cache[p][2] = self.palette_cache[0][2];
      self.palette_cache[p][3] = 255;
    }
  }

  pub fn generate_nametables(&mut self, mapper: &mut Mapper, ppu: &mut ppu::PpuState, dx: u32, dy: u32) {
    let mut pattern_address = 0x0000;
    if (ppu.control & 0x10) != 0 {
      pattern_address = 0x1000;
    }
    
    for tx in 0 .. 64 {
      for ty in 0 .. 60 {
        let tile_index = ppu.get_bg_tile(mapper, tx, ty);
        let palette_index = ppu.get_bg_palette(mapper, tx, ty);
        draw_tile(mapper, pattern_address, tile_index as u16, &mut self.buffer, 
          dx + tx as u32 * 8, dy + ty as u32 * 8, &self.palette_cache[palette_index as usize]);
      }
    }
  }

  pub fn update(&mut self, nes: &mut NesState) {
    self.update_palette_cache(nes);
    generate_chr_pattern(&mut *nes.mapper, 0x0000, &mut self.buffer,   0, 0);
    generate_chr_pattern(&mut *nes.mapper, 0x1000, &mut self.buffer, 128, 0);
    self.generate_nametables(&mut *nes.mapper, &mut nes.ppu, 256, 0);
  }
  

  pub fn draw(&mut self) {
    let texture_creator = self.canvas.texture_creator();
    let mut texture = texture_creator.create_texture(PixelFormatEnum::ABGR8888, TextureAccess::Streaming, self.buffer.width, self.buffer.height).unwrap();
      
    self.canvas.set_draw_color(Color::RGB(255, 255, 255));
    let _ = texture.update(None, &self.buffer.buffer, (self.buffer.width * 4) as usize);
    let _ = self.canvas.copy(&texture, None, None);

    self.canvas.present();
  }

  pub fn handle_event(&mut self, _: &mut NesState, event: &sdl2::event::Event) {
    let self_id = self.canvas.window().id();
    match *event {
      Event::Window { window_id: id, win_event: WindowEvent::Close, .. } if id == self_id => {
        self.shown = false;
        self.canvas.window_mut().hide();
      },
      _ => ()
    }
  }
}

