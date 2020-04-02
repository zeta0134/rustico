extern crate nfd;

use rusticnes_core::nes::NesState;
use rusticnes_core::palettes::NTSC_PAL;

use nfd::Response;

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use sdl2::keyboard::Keycode;

pub struct GameWindow {
  pub screen_buffer: [u8; 256 * 240 * 4],
  pub running: bool,
  pub file_loaded: bool,
  pub shown: bool,
  pub game_path: PathBuf,
  pub save_path: PathBuf,
  pub scale: u32,
  pub display_overscan: bool,
}

impl GameWindow {
  pub fn new() -> GameWindow {
    let game_screen_buffer = [0u8; 256 * 240 * 4];
    return GameWindow {
      screen_buffer: game_screen_buffer,
      running: false,
      file_loaded: false,
      shown: true,
      game_path: PathBuf::from(""),
      save_path: PathBuf::from(""),
      scale: 2,
      display_overscan: false,
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
    let file = File::open(file_path);
    match file {
        Err(why) => {
            println!("Couldn't open {}: {}", file_path, why.description());
            return;
        },
        Ok(_) => (),
    };
    // Read the whole thing
    let mut cartridge = Vec::new();
    match file.unwrap().read_to_end(&mut cartridge) {
        Err(why) => {
            println!("Couldn't read from {}: {}", file_path, why.description());
        },
        Ok(bytes_read) => {
            println!("Data read successfully: {}", bytes_read);
            let maybe_nes = NesState::from_rom(&cartridge);
            match maybe_nes {
            Ok(nes_state) => {
              *nes = nes_state;
              self.running = true;
              self.file_loaded = true;
              self.game_path = PathBuf::from(file_path);
              self.save_path = self.game_path.with_extension("sav");
              if nes.mapper.has_sram() {
                read_sram(nes, self.save_path.to_str().unwrap());
              }
            },
            Err(why) => {
              println!("{}", why);
            }
          }
        },
    };    
  }

  pub fn update(&mut self, nes: &mut NesState) {
    if self.running {
      nes.run_until_vblank();
    }

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

  pub fn handle_key_down(&mut self, nes: &mut NesState, key: Keycode) {
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

    for i in 0 .. 8 {
      if key == key_mappings[i] {
        // Set the corresponding bit
        nes.p1_input |= 0x1 << i;
      }
      // Prevent impossible combinations on a real D-Pad
      // TODO Later: make this an option?
      if key == Keycode::Up {
        nes.p1_input &= 0b1101_1111;
      }
      if key == Keycode::Down {
        nes.p1_input &= 0b1110_1111;
      }
      if key == Keycode::Left {
        nes.p1_input &= 0b0111_1111;
      }
      if key == Keycode::Right {
        nes.p1_input &= 0b1011_1111;
      }
    }
    match key {
      Keycode::P => {
        if self.file_loaded {
          self.running = !self.running;
        }
      },
      Keycode::Escape => {
        self.shown = false;
        // We're closing the program, so write out the SRAM one last time
        write_sram(nes, self.save_path.to_str().unwrap());
        println!("SRAM Saved! (Escape closes Main Window)");
      },
      Keycode::Space => {
        nes.step();
      },
      Keycode::C => {
        nes.cycle();
      },
      Keycode::H => {
        nes.run_until_hblank();
      },
      Keycode::V => {
        nes.run_until_vblank();
      },
      Keycode::S => {
        // Manual SRAM write
        write_sram(nes, self.save_path.to_str().unwrap());
        println!("SRAM Saved!");
      },
      _ => ()
    }
  }

  pub fn handle_key_up(&mut self, nes: &mut NesState, key: Keycode) {
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

    for i in 0 .. 8 {
      if key == key_mappings[i] {
        // Clear the corresponding bit
        nes.p1_input &= (0x1 << i) ^ 0xFF;
      }
    }
    match key {
      Keycode::R => {
        println!("Resetting NES");
        nes.reset();
      },
      _ => ()
    }
  }
}

fn read_sram(nes: &mut NesState, file_path: &str) {
    let file = File::open(file_path);
    match file {
        Err(why) => {
            println!("Couldn't open {}: {}", file_path, why.description());
            return;
        },
        Ok(_) => (),
    };
    // Read the whole thing
    let mut sram_data = Vec::new();
    match file.unwrap().read_to_end(&mut sram_data) {
        Err(why) => {
            println!("Couldn't read data: {}", why.description());
            return;
        },
        Ok(_) => {
            nes.set_sram(sram_data);
        }
    }
}

fn write_sram(nes: &mut NesState, file_path: &str) {
    if nes.mapper.has_sram() {
        let file = File::create(file_path);
        match file {
            Err(why) => {
                println!("Couldn't open {}: {}", file_path, why.description());
            },
            Ok(mut file) => {
                let _ = file.write_all(&nes.sram());
            },
        };
    }
}