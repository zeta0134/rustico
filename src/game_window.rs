extern crate nfd;
extern crate sdl2;

use rusticnes_core::cartridge;
use rusticnes_core::memory;
use rusticnes_core::nes;
use rusticnes_core::nes::NesState;
use rusticnes_core::palettes::NTSC_PAL;

use nfd::Response;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

use std::error::Error;
use std::fs::File;
use std::io::Read;

pub struct GameWindow {
  pub canvas: sdl2::render::WindowCanvas,
  pub screen_buffer: [u8; 256 * 240 * 4],
  pub running: bool,
  pub file_loaded: bool,
}

impl GameWindow {
  pub fn new(sdl_context: &sdl2::Sdl) -> GameWindow {
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("RusticNES", 512, 480)
        .position(50, 50)
        .opengl()
        .build()
        .unwrap();

    let mut game_canvas = window.into_canvas().present_vsync().build().unwrap();
    game_canvas.set_draw_color(Color::RGB(0, 0, 0));
    game_canvas.clear();
    game_canvas.present();

    let game_screen_buffer = [0u8; 256 * 240 * 4];

    return GameWindow {
      canvas: game_canvas,
      screen_buffer: game_screen_buffer,
      running: false,
      file_loaded: false
    }
  }

  pub fn open_file_dialog(&mut self, nes: &mut NesState) {
    let result = nfd::dialog().filter("nes").open().unwrap_or_else(|e| { panic!(e); });

    match result {
      Response::Okay(file_path) => {
        println!("Opened: {:?}", file_path);
        println!("Attempting to load {}...", file_path);

        self.open_file(nes, &file_path);
      },
      Response::OkayMultiple(files) => println!("Opened: {:?}", files),
      Response::Cancel => println!("No file opened!"),
    }
  }

  pub fn open_file(&mut self, nes: &mut NesState, file_path: &str) {
    let mut file = match File::open(file_path) {
      Err(why) => panic!("Couldn't open {}: {}", file_path, why.description()),
      Ok(file) => file,
    };
    // Read the whole thing
    let mut cartridge = Vec::new();
    match file.read_to_end(&mut cartridge) {
      Err(why) => panic!("Couldn't read data: {}", why.description()),
      Ok(bytes_read) => {
        println!("Data read successfully: {}", bytes_read);

        let nes_header = cartridge::extract_header(&cartridge);
        cartridge::print_header_info(nes_header);
        let mapper = cartridge::load_from_cartridge(nes_header, &cartridge);
        *nes = NesState::new(mapper);
        self.running = true;
        self.file_loaded = true;
        nes.apu.buffer_full = false;

        // Initialize CPU register state for power-up sequence
        nes.registers.a = 0;
        nes.registers.y = 0;
        nes.registers.x = 0;
        nes.registers.s = 0xFD;

        let pc_low = memory::read_byte(nes, 0xFFFC);
        let pc_high = memory::read_byte(nes, 0xFFFD);
        nes.registers.pc = pc_low as u16 + ((pc_high as u16) << 8);
      },
    };
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
      
      self.canvas.set_draw_color(Color::RGB(255, 255, 255));
      let _ = game_screen_texture.update(None, &self.screen_buffer, 256 * 4);
      let _ = self.canvas.copy(&game_screen_texture, None, None);
    }
    self.canvas.present();
  }

  pub fn handle_event(&mut self, nes: &mut NesState, event: &sdl2::event::Event) {
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

    match *event {
      Event::KeyDown { keycode: Some(key), .. } => {
        for i in 0 .. 8 {
          if key == key_mappings[i] {
            // Set the corresponding bit
            nes.p1_input |= 0x1 << i;
          }
        }
        match key {
          Keycode::R => {
            if self.file_loaded {
              self.running = !self.running;
            }
          },
          Keycode::O => {self.open_file_dialog(nes);},
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

