extern crate sdl2;

use rusticnes_core::nes::NesState;
use rusticnes_core::memory;

use sdl2::keyboard::Keycode;

use rusticnes_ui_common::drawing;
use rusticnes_ui_common::drawing::Font;
use rusticnes_ui_common::drawing::SimpleBuffer;

pub struct MemoryWindow {  
  pub buffer: SimpleBuffer,
  pub shown: bool,
  pub font: Font,
  pub view_ppu: bool,
  pub memory_page: u16,
}

impl MemoryWindow {
  pub fn new() -> MemoryWindow {
    let font = Font::from_raw(include_bytes!("assets/8x8_font.png"), 8);

    return MemoryWindow {
      buffer: SimpleBuffer::new(360, 220),
      font: font,
      shown: false,
      view_ppu: false,
      memory_page: 0x0000,
    }
  }

  pub fn draw_memory_page(&mut self, nes: &mut NesState, sx: u32, sy: u32) {
  for y in 0 .. 16 {
    for x in 0 .. 16 {
      let address = self.memory_page + (x as u16) + (y as u16 * 16);
      let byte: u8;
      let mut bg_color = [32, 32, 32, 255];
      if (x + y) % 2 == 0 {
        bg_color = [48, 48, 48, 255];
      }
      if self.view_ppu {
        let masked_address = address & 0x3FFF;
        byte = nes.ppu.debug_read_byte(& *nes.mapper, masked_address);
        if masked_address == (nes.ppu.current_vram_address & 0x3FFF) {
          bg_color = [128, 32, 32, 255];
        } else if nes.ppu.recent_reads.contains(&masked_address) {
          for i in 0 .. nes.ppu.recent_reads.len() {
            if nes.ppu.recent_reads[i] == masked_address {
              let brightness = 192 - (5 * i as u8);
              bg_color = [64, brightness, 64, 255];
              break;
            }
          }
        } else if nes.ppu.recent_writes.contains(&masked_address) {
          for i in 0 .. nes.ppu.recent_writes.len() {
            if nes.ppu.recent_writes[i] == masked_address {
              let brightness = 192 - (5 * i as u8);
              bg_color = [brightness, brightness, 32, 255];
              break;
            }
          }
        }
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
    let height = self.buffer.height;
    
    drawing::rect(&mut self.buffer, 0, 0, width, 33, &[0,0,0,255]);
    drawing::rect(&mut self.buffer, 0, 0, 56, height, &[0,0,0,255]);
    drawing::text(&mut self.buffer, &self.font, 0, 0, &format!("{} Page: 0x{:04X}",
      if self.view_ppu {"PPU"} else {"CPU"}, self.memory_page), 
      &[255, 255, 255, 255]);

    // Draw memory region selector
    for i in 0x0 .. 0x10 {
      // Highest Nybble
      let cell_x = 56  + (i as u32 * 19);
      let mut cell_y = 11;
      let mut text_color = [255, 255, 255, 64];
      if ((self.memory_page & 0xF000) >> 12) == i {
        drawing::rect(&mut self.buffer, cell_x, cell_y, 19, 11, &[64, 64, 64,255]);
        text_color = [255, 255, 255, 192];
      }
      drawing::hex(&mut self.buffer, &self.font, cell_x + 2, cell_y + 2, i as u32, 1, &text_color);
      drawing::char(&mut self.buffer, &self.font, cell_x + 2 + 8, cell_y + 2, 'X', &text_color);

      // Second-highest Nybble
      text_color = [255, 255, 255, 64];
      cell_y = 22;
      if ((self.memory_page & 0x0F00) >> 8) == i {
        drawing::rect(&mut self.buffer, cell_x, cell_y, 19, 11, &[64, 64, 64,255]);
        text_color = [255, 255, 255, 192];
      }
      drawing::char(&mut self.buffer, &self.font, cell_x + 2, cell_y + 2, 'X', &text_color);
      drawing::hex(&mut self.buffer, &self.font, cell_x + 2 + 8, cell_y + 2, i as u32, 1, &text_color);
    }

    // Draw row labels
    for i in 0 .. 0x10 {
      drawing::text(&mut self.buffer, &self.font, 0, 44 + 2 + (i as u32 * 11), &format!("0x{:04X}",
        self.memory_page + (i as u16 * 0x10)), 
        &[255, 255, 255, 64]);
    }
    self.draw_memory_page(nes, 56, 44);
  }

  pub fn handle_key_up(&mut self, _nes: &mut NesState, key: Keycode) {
    match key {
      Keycode::Period => {
        self.memory_page = self.memory_page.wrapping_add(0x100);
      },
      Keycode::Comma => {
        self.memory_page = self.memory_page.wrapping_sub(0x100);
      },
      Keycode::Slash => {
        self.view_ppu = !self.view_ppu;
      },
      _ => ()
    }
  }

  pub fn handle_click(&mut self, _nes: &mut NesState, mx: i32, my: i32) {
    if my < 11 && mx < 32 {
      self.view_ppu = !self.view_ppu;
    }
    if my >= 11 && my < 22 && mx > 56 && mx < 360 {
      let high_nybble = ((mx - 56) / 19) as u16;
      self.memory_page = (self.memory_page & 0x0FFF) | (high_nybble << 12);
    }
    if my >= 22 && my < 33 && mx > 56 && mx < 360 {
      let low_nybble = ((mx - 56) / 19) as u16;
      self.memory_page = (self.memory_page & 0xF0FF) | (low_nybble << 8);
    }
  }

  /*
  pub fn handle_event(&mut self, _: &mut NesState, event: &sdl2::event::Event) {
    let self_id = self.canvas.window().id();
    match *event {
      Event::Window { window_id: id, win_event: WindowEvent::Close, .. } if id == self_id => {
        self.shown = false;
        self.canvas.window_mut().hide();
      },
      Event::MouseButtonDown{ window_id: id, mouse_btn: MouseButton::Left, x: omx, y: omy, .. } if id == self_id => {
        let mx = omx / 2;
        let my = omy / 2;
        if my < 11 && mx < 32 {
          self.view_ppu = !self.view_ppu;
        }
        if my >= 11 && my < 22 && mx > 56 && mx < 360 {
          let high_nybble = ((mx - 56) / 19) as u16;
          self.memory_page = (self.memory_page & 0x0FFF) | (high_nybble << 12);
        }
        if my >= 22 && my < 33 && mx > 56 && mx < 360 {
          let low_nybble = ((mx - 56) / 19) as u16;
          self.memory_page = (self.memory_page & 0xF0FF) | (low_nybble << 8);
        }
      },
      Event::KeyDown { keycode: Some(key), .. } => {
        match key {
          Keycode::Period => {
            self.memory_page = self.memory_page.wrapping_add(0x100);
          },
          Keycode::Comma => {
            self.memory_page = self.memory_page.wrapping_sub(0x100);
          },
          Keycode::Slash => {
            self.view_ppu = !self.view_ppu;
          },
          _ => ()
        }
      },
      _ => ()
    }
  }*/
}

