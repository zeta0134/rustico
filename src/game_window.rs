extern crate nfd;
extern crate sdl2;

use rusticnes_core::memory;
use rusticnes_core::mmc::mapper::Mirroring;
use rusticnes_core::nes;
use rusticnes_core::nes::NesState;
use rusticnes_core::palettes::NTSC_PAL;

use nfd::Response;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

use std::path::PathBuf;

pub struct GameWindow {
  pub canvas: sdl2::render::WindowCanvas,
  pub screen_buffer: [u8; 256 * 240 * 4],
  pub running: bool,
  pub file_loaded: bool,
  pub shown: bool,
  pub game_path: PathBuf,
  pub save_path: PathBuf,
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
      file_loaded: false,
      shown: true,
      game_path: PathBuf::from(""),
      save_path: PathBuf::from(""),
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
    let maybe_nes = nes::open_file(file_path);
    match maybe_nes {
      Ok(nes_state) => {
        *nes = nes_state;
        self.running = true;
        self.file_loaded = true;
        self.game_path = PathBuf::from(file_path);
        self.save_path = self.game_path.with_extension("sav");
        if nes.mapper.has_sram() {
          nes.read_sram(self.save_path.to_str().unwrap());
        }
      },
      Err(why) => {
        println!("{}", why);
      }
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

  pub fn draw(&mut self) {
    let game_screen_texture_creator = self.canvas.texture_creator();
    let mut game_screen_texture = game_screen_texture_creator.create_texture(PixelFormatEnum::RGBA8888, TextureAccess::Streaming, 256, 240).unwrap();
    
    self.canvas.set_draw_color(Color::RGB(255, 255, 255));
    let _ = game_screen_texture.update(None, &self.screen_buffer, 256 * 4);
    let _ = self.canvas.copy(&game_screen_texture, None, None);
    self.canvas.present();
  }

  pub fn print_program_state(nes: &mut NesState) {
    let registers = nes.registers;
    println!("=== NES State ===");
    println!("A: 0x{:02X} X: 0x{:02X} Y: 0x{:02X}", registers.a, registers.x, registers.y);
    println!("PC: 0x{:02X} S: 0x{:02X}", registers.pc, registers.s);
    println!("Flags: nv  dzic");
    println!("       {:b}{:b}  {:b}{:b}{:b}{:b}",
      registers.flags.negative as u8,
      registers.flags.overflow as u8,
      registers.flags.decimal as u8,
      registers.flags.zero as u8,
      registers.flags.interrupts_disabled as u8,
      registers.flags.carry as u8,
    );
    println!("\nMemory @ Program Counter");
    // print out the next 8 bytes or so from the program counter
    let mut pc = registers.pc;
    for _ in 1 .. 8 {
      println!("0x{:04X}: 0x{:02X}", pc, memory::passively_read_byte(nes, pc));
      pc = pc.wrapping_add(1);
    }
 
    let mirror_mode = match nes.mapper.mirroring() {
      Mirroring::Horizontal => "Horizontal",
      Mirroring::Vertical => "Vertical",
      Mirroring::OneScreenLower => "OneScreen - Lower",
      Mirroring::OneScreenUpper => "OneScreen - Upper",
      Mirroring::FourScreen => "FourScreen",
    };
 
    println!("\nPPU: Control: {:02X} Mask: {:02X} Status: {:02X} Fine Y: {:02X}",
      nes.ppu.control, nes.ppu.mask, nes.ppu.status, nes.ppu.fine_y());
    println!("VRAM: Current: {:016b} Temp:Address: {:016b}",
      nes.ppu.current_vram_address, nes.ppu.temporary_vram_address);
    println!("OAM Address: {:04X} PPU Address: {:04X}",
      nes.ppu.oam_addr, nes.ppu.current_vram_address);
    println!("Frame: {}, Scanline: {}, M. Clock: {}, CPU. Cycle: {}, Scanline Cycle: {}, Mirroring: {}\n",
      nes.ppu.current_frame, nes.ppu.current_scanline, nes.master_clock, (nes.cpu.tick + 1), nes.ppu.current_scanline_cycle, mirror_mode);
    nes.mapper.print_debug_status();
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

    let self_id = self.canvas.window().id();
    match *event {
      Event::Window { window_id: id, win_event: WindowEvent::Close, .. } if id == self_id => {
        self.shown = false;
        self.canvas.window_mut().hide();
        // We're closing the program, so write out the SRAM one last time
        nes.write_sram(self.save_path.to_str().unwrap());
        println!("SRAM Saved! (Closing Main Window)");
      },
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
          Keycode::Space => {
            nes.step();
            GameWindow::print_program_state(nes);
          },
          Keycode::C => {
            nes.cycle();
            GameWindow::print_program_state(nes);
          },
          Keycode::H => {
            nes.run_until_hblank();
            GameWindow::print_program_state(nes);
          },
          Keycode::V => {
            nes.run_until_vblank();
            GameWindow::print_program_state(nes);
          },
          Keycode::S => {
            // Manual SRAM write
            nes.write_sram(self.save_path.to_str().unwrap());
            println!("SRAM Saved!");
          },
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

