extern crate sdl2;

use rusticnes_core::mmc::mapper::Mapper;
use rusticnes_core::nes::NesState;
use rusticnes_core::ppu;
use rusticnes_core::palettes::NTSC_PAL;

use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
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

pub fn draw_color_box(buffer: &mut SimpleBuffer, dx: u32, dy: u32, color: &[u8]) {
  // First, draw a white outline
  for x in 0 .. 16 {
    for y in 0 .. 16 {
      buffer.put_pixel(dx + x, dy + y, 
        &[255, 255, 255, 255]);
    }
  }
  // Then draw the palette color itself in the center of the outline
  for x in 1 .. 15 {
    for y in 1 .. 15 {
      buffer.put_pixel(dx + x, dy + y, color);          
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

    // Draw a red border around the present scroll viewport
    let vram_address = ppu.current_vram_address;
    let coarse_x =  vram_address & 0b000_00_00000_11111;
    let coarse_y = (vram_address & 0b000_00_11111_00000) >> 5;
    let fine_x = ppu.fine_x;
    let fine_y =   (vram_address & 0b111_00_00000_00000) >> 12;
    let scroll_x = (coarse_x << 3 | fine_x as u16) as u32;
    let scroll_y = (coarse_y << 3 | fine_y as u16) as u32;

    for x in scroll_x .. scroll_x + 256 {
      let px = x % 512;
      let mut py = (scroll_y) % 480;
      self.buffer.put_pixel(dx + px, dy + py, &[255, 0, 0, 255]);
      py = (scroll_y + 239) % 480;
      self.buffer.put_pixel(dx + px, dy + py, &[255, 0, 0, 255]);
    }

    for y in scroll_y .. scroll_y + 240 {
      let py = y % 480;
      let mut px = scroll_x % 512;
      self.buffer.put_pixel(dx + px, dy + py, &[255, 0, 0, 255]);
      px = (scroll_x + 255) % 512;
      self.buffer.put_pixel(dx + px, dy + py, &[255, 0, 0, 255]);
    }
  }

  pub fn draw_palettes(&mut self, dx: u32, dy: u32) {
    // Global Background (just once)
    let color = &self.palette_cache[0][0 .. 4];
    draw_color_box(&mut self.buffer, dx, dy, color);

    // Backgrounds
    for p in 0 .. 4 {
      for i in 1 .. 4 {
        let x = dx + p * 64 + i * 15;
        let y = dy;
        let color = &self.palette_cache[p as usize][(i * 4) as usize .. (i * 4 + 4) as usize];
        draw_color_box(&mut self.buffer, x, y, color);
      }
    }

    // Sprites
    for p in 0 .. 4 {
      for i in 1 .. 4 {
        let x = dx + p * 64 + i * 15;
        let y = dy + 18;
        let color = &self.palette_cache[(p + 4) as usize][(i * 4) as usize .. (i * 4 + 4) as usize];
        draw_color_box(&mut self.buffer, x, y, color);
      }
    }
  }

  pub fn update(&mut self, nes: &mut NesState) {
    self.update_palette_cache(nes);
    // Left Pane: CHR memory, Palette Colors
    generate_chr_pattern(&mut *nes.mapper, 0x0000, &mut self.buffer,   0, 0);
    generate_chr_pattern(&mut *nes.mapper, 0x1000, &mut self.buffer, 128, 0);
    self.draw_palettes(0, 128);
    // Right Panel: Entire nametable
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
