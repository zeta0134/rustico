extern crate nfd;

use rusticnes_core::nes::NesState;
use rusticnes_core::palettes::NTSC_PAL;

use sdl2::keyboard::Keycode;

pub struct GameWindow {
  pub screen_buffer: [u8; 256 * 240 * 4],
  pub running: bool,
  pub file_loaded: bool,
  pub shown: bool,
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
      scale: 2,
      display_overscan: false,
    }
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
      },
      Keycode::Space => {
        nes.step();
      },
      Keycode::D => {
        nes.mapper.print_debug_status();
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