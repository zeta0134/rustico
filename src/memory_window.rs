extern crate sdl2;

use rusticnes_core::nes::NesState;
use rusticnes_core::memory;

use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

use drawing;
use drawing::Font;
use drawing::SimpleBuffer;

pub struct MemoryWindow {
  pub canvas: sdl2::render::WindowCanvas,
  pub buffer: SimpleBuffer,
  pub shown: bool,
  pub font: Font,
  pub view_ppu: bool,
  pub memory_page: u16,
}

impl MemoryWindow {
  pub fn new(sdl_context: &sdl2::Sdl) -> MemoryWindow {
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Memory Viewer", 304, 360)
        .position(570, 50)
        .hidden()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let font = Font::new("assets/8x8_font.png", 8);

    return MemoryWindow {
      canvas: canvas,
      buffer: SimpleBuffer::new(304, 360),
      font: font,
      shown: false,
      view_ppu: false,
      memory_page: 0x0000,
    }
  }

  pub fn draw_memory_page(&mut self, nes: &mut NesState, sx: u32, sy: u32) {
  for y in 0 .. 32 {
    for x in 0 .. 16 {
      let address = self.memory_page + (x as u16) + (y as u16 * 16);
      let byte: u8;
      let mut bg_color = [32, 32, 32, 255];
      if (x + y) % 2 == 0 {
        bg_color = [48, 48, 48, 255];
      }
      if self.view_ppu {
        let masked_address = address & 0x3FFF;
        match masked_address {
          0x0000 ... 0x1FFF => byte = nes.mapper.debug_read_byte(masked_address),
          _ => byte = nes.ppu._read_byte(&mut *nes.mapper, masked_address)
        };
      } else {
        byte = memory::passively_read_byte(nes, address);
        if address == nes.registers.pc {
          bg_color = [128, 32, 32, 255];
        } else if address == (nes.registers.s as u16 + 0x100) {
          bg_color = [32, 32, 128, 255];
        } else if nes.memory.recent_reads.contains(&address) {
          for i in 0 .. nes.memory.recent_reads.len() {
            if nes.memory.recent_reads[i] == address {
              let brightness = 192 - (5 * i as u8);
              bg_color = [64, brightness, 64, 255];
              break;
            }
          }
        } else if nes.memory.recent_writes.contains(&address) {
          for i in 0 .. nes.memory.recent_writes.len() {
            if nes.memory.recent_writes[i] == address {
              let brightness = 192 - (5 * i as u8);
              bg_color = [brightness, brightness, 32, 255];
              break;
            }
          }
        }
      }
      let mut text_color = [255, 255, 255, 192];
      if byte == 0 {
        text_color = [255, 255, 255, 64];
      }
      let cell_x = sx + x * 19;
      let cell_y = sy + y * 11;
      drawing::rect(&mut self.buffer, cell_x, cell_y, 19, 11, &bg_color);
      drawing::hex(&mut self.buffer, &self.font, 
        cell_x + 2, cell_y + 2,
        byte as u32, 2, 
        &text_color);
    }
  }
}

  pub fn update(&mut self, nes: &mut NesState) {
    let width = self.buffer.width;
    drawing::rect(&mut self.buffer, 0, 0, width, 8, &[0,0,0,255]);
    drawing::text(&mut self.buffer, &self.font, 0, 0, &format!("{} - 0x{:04X}",
      if self.view_ppu {"PPU"} else {"CPU"}, self.memory_page), 
      &[255, 255, 255, 255]);
    self.draw_memory_page(nes, 0, 8);
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
      Event::KeyDown { keycode: Some(key), .. } => {
        match key {
          Keycode::Period => {
            self.memory_page = self.memory_page.wrapping_add(0x200);
          },
          Keycode::Comma => {
            self.memory_page = self.memory_page.wrapping_sub(0x200);
          },
          Keycode::Slash => {
            self.view_ppu = !self.view_ppu;
          },
          _ => ()
        }
      },
      _ => ()
    }
  }
}

