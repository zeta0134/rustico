extern crate sdl2;

use rusticnes_core::nes;
use rusticnes_core::nes::NesState;
use rusticnes_core::palettes::NTSC_PAL;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;
use sdl2::video::WindowContext;

pub struct GameWindow {
  pub canvas: sdl2::render::WindowCanvas,
  pub screen_buffer: [u8; 256 * 240 * 4],

  pub running: bool,
}

impl GameWindow {
  pub fn new(sdl_context: &sdl2::Sdl) -> GameWindow {
    let audio_subsystem = sdl_context.audio().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let keyboard = sdl_context.keyboard();

    let window = video_subsystem.window("RusticNES", 512, 480)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut game_canvas = window.into_canvas().present_vsync().build().unwrap();
    game_canvas.set_draw_color(Color::RGB(0, 0, 0));
    game_canvas.clear();
    game_canvas.present();

    let mut game_screen_buffer = [0u8; 256 * 240 * 4];

    return GameWindow {
      canvas: game_canvas,
      screen_buffer: game_screen_buffer,
      running: false
    }
  }

  pub fn update(&mut self, nes: &mut NesState) {
    if self.running {
      nes::run_until_vblank(nes);

      // Update the game screen
      for x in 0 .. 256 {
        for y in 0 .. 240 {
          let palette_index = ((nes.ppu.screen[y * 256 + x]) as usize) * 3;
          self.screen_buffer[((y * 256 + x) * 4) + 3] = NTSC_PAL[palette_index + 0];
          self.screen_buffer[((y * 256 + x) * 4) + 2] = NTSC_PAL[palette_index + 1];
          self.screen_buffer[((y * 256 + x) * 4) + 1] = NTSC_PAL[palette_index + 2];
          self.screen_buffer[((y * 256 + x) * 4) + 0] = 255;
        }
      }
    }
  }

  pub fn draw(&mut self) {
    if self.running {
      let game_screen_texture_creator = self.canvas.texture_creator();
      let mut game_screen_texture = game_screen_texture_creator.create_texture(PixelFormatEnum::RGBA8888, TextureAccess::Streaming, 256, 240).unwrap();
      
      game_screen_texture.update(None, &self.screen_buffer, 256 * 4);
      self.canvas.set_draw_color(Color::RGB(255, 255, 255));
      self.canvas.copy(&game_screen_texture, None, None);
    }
    self.canvas.present();
  }

  pub fn handle_event(&mut self, nes: &mut NesState, event: sdl2::event::Event) {
    let key_mappings: [Keycode; 8] = [
      Keycode::X,
      Keycode::Z,
      Keycode::RShift,
      Keycode::Return,
      Keycode::Up,
      Keycode::Down,
      Keycode::Left,
      Keycode::Right,
    ];

    match event {
      Event::KeyDown { keycode: Some(key), .. } => {
        for i in 0 .. 8 {
          if key == key_mappings[i] {
            // Set the corresponding bit
            nes.p1_input |= 0x1 << i;
          }
        }
        match key {
          Keycode::R => {self.running = !self.running;},
          _ => ()
        }         
      },
      Event::KeyUp { keycode: Some(key), .. } => {
        for i in 0 .. 8 {
          if key == key_mappings[i] {
            // Clear the corresponding bit
            nes.p1_input &= (0x1 << i) ^ 0xFF;
          }
        }
      },
      _ => {}
    }
  }
}

