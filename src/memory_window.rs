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

pub fn draw_memory_page(nes: &mut NesState, starting_address: u16, imagebuffer: &mut SimpleBuffer, font: &Font, sx: u32, sy: u32) {
  for y in 0 .. 16 {
    for x in 0 .. 16 {
      let address = starting_address + (x as u16) + (y as u16 * 16);
      let byte = memory::passively_read_byte(nes, address);
      let mut bg_color = [32, 32, 32, 255];
      if (x + y) % 2 == 0 {
        bg_color = [48, 48, 48, 255];
      }
      let mut text_color = [255, 255, 255, 255];
      if byte == 0 {
        text_color = [255, 255, 255, 64];
      }
      let cell_x = sx + x * 19;
      let cell_y = sy + y * 11;
      drawing::rect(imagebuffer, cell_x, cell_y, 19, 11, &bg_color);
      drawing::hex(imagebuffer, font, 
        cell_x + 2, cell_y + 2,
        byte as u32, 2, 
        &text_color);
    }
  }
}

pub struct MemoryWindow {
  pub canvas: sdl2::render::WindowCanvas,
  pub buffer: SimpleBuffer,
  pub shown: bool,
  pub font: Font,
}

impl MemoryWindow {
  pub fn new(sdl_context: &sdl2::Sdl) -> MemoryWindow {
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Memory Viewer", 304, 352)
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
      buffer: SimpleBuffer::new(304, 352),
      font: font,
      shown: false,
    }
  }

  

  pub fn update(&mut self, nes: &mut NesState) {
    draw_memory_page(nes, 0x0000, &mut self.buffer, &self.font, 0, 0);
    draw_memory_page(nes, 0x0100, &mut self.buffer, &self.font, 0, 176);
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
          _ => ()
        }
      },
      _ => ()
    }
  }
}

