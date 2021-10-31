extern crate image;
extern crate rusticnes_core;

use rusticnes_core::nes::NesState;
use rusticnes_core::mmc::none::NoneMapper;
use rusticnes_core::palettes::NTSC_PAL;

use std::env;
use std::fs::File;
use std::str;

use std::io::Read;
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;

fn load_cartridge(nes: &mut NesState, cartridge_path: &str) {
  // Read in the ROM file and attempt to create a new NesState:
  let file = File::open(cartridge_path);
  match file {
    Err(why) => {
      panic!("Couldn't open {}: {}", cartridge_path, why);
    },
    Ok(_) => (),
  };

  // Read the whole thing
  let mut cartridge = Vec::new();
  match file.unwrap().read_to_end(&mut cartridge) {
    Err(why) => {
      panic!("Couldn't read from {}: {}", cartridge_path, why);
    },
    Ok(_) => {
      println!("Loading {}...", cartridge_path);
      let maybe_nes = NesState::from_rom(&cartridge);
      match maybe_nes {
        Ok(nes_state) => {
          *nes = nes_state;
        },
        Err(why) => {
          panic!("{}", why);
        }
      }
    },
  };
}

fn run(nes: &mut NesState, frames: u64) {
  for _ in 0 .. frames {
    nes.run_until_vblank();
  }
}

fn reset(nes: &mut NesState) {
  nes.reset();
}

fn tap(nes: &mut NesState, button: &str, frames: u64) {
  let button_index: u8 = match button {
    "a" => 0,
    "b" => 1,
    "select" => 2,
    "start" => 3,
    "up" => 4,
    "down" => 5,
    "left" => 6,
    "right" => 7,
    _ => panic!("Invalid button to tap: {}", button)
  };
  nes.p1_input |= 0x1 << button_index;
  run(nes, frames);
  nes.p1_input ^= 0x1 << button_index;
}

fn save_screenshot(nes: &NesState, output_path: &str) {
  let mut img = image::ImageBuffer::new(256, 240);
  for x in 0 .. 256 {
    for y in 0 .. 240 {
      let palette_index = ((nes.ppu.screen[y * 256 + x]) as usize) * 3;
      img.put_pixel(x as u32, y as u32, image::Rgba([
        NTSC_PAL[palette_index + 0],
        NTSC_PAL[palette_index + 1],
        NTSC_PAL[palette_index + 2],
        255 as u8]));
    }
  }

  image::ImageRgba8(img).save(output_path).unwrap();

  println!("Saved screenshot to {}", output_path);
}

fn save_blargg(nes: &mut NesState, output_filename: &str) {
  if nes.mapper.has_sram() {
    let sram = nes.mapper.get_sram();

    let test_status = sram[0];
    let magic_0 = sram[1];
    let magic_1 = sram[2];
    let magic_2 = sram[3];

    if magic_0 == 0xDE && magic_1 == 0xB0 && magic_2 == 0x61 {
      let test_status_string = match test_status {
        0x80 => format!("Running"),
        0x81 => format!("Needs RESET"),
        _ => format!("0x{:02X}", test_status),
      };


      // Starting at 0x6004, read all of NES memory up to the next null terminator (or 0x8000) as ASCII
      let begin = 4;
      // Locate the next null terminator
      let mut end = 4;
      while sram[end] != 0  && end < sram.len() {
        end+=1;
      }

      let test_text = str::from_utf8(&sram[begin .. end]).unwrap();
      let output = format!("Test Status: {}\n\n{}", test_status_string, test_text);

      // Output!
      let ref mut file = File::create(output_filename).unwrap();
      let _ = file.write_all(output.as_ref());
      println!("Saved blargg data to {}", output_filename);
    } else {
      let ref mut file = File::create(output_filename).unwrap();
      let _ = file.write_all(format!("Invalid blargg magic header, found 0x{:02X} 0x{:02X} 0x{:02X} instead.", magic_0, magic_1, magic_2).as_ref());
    }    
  } else {
    panic!("Cannot output blargg data, ROM has no SRAM!");
  }
}

fn command_file(nes: &mut NesState, command_path: &str) {
  let file = File::open(command_path);
  match file {
    Err(why) => {
      panic!("Couldn't open {}: {}", command_path, why);
    },
    Ok(_) => (),
  };

  let unwrapped_file = file.unwrap();
  let file_reader = BufReader::new(&unwrapped_file);
  for l in file_reader.lines() {
    let line = l.unwrap();
    let command_list = line.split(" ").map(|s| s.to_string()).collect();
    process_command_list(nes, command_list);
  }
}

fn process_command_list(nes: &mut NesState, mut command_list: Vec<String>) {
  while command_list.len() > 0 {
    let command = command_list.remove(0);
    match command.as_ref() {
      "cart" | "cartridge" | "rom" => {
        let cartridge_path = command_list.remove(0);
        load_cartridge(nes, cartridge_path.as_ref());
      },
      "run" | "frames" => {
        let frames: u64 = command_list.remove(0).parse().unwrap();
        run(nes, frames);
      },
      "reset" => {
        reset(nes);
      }
      "tap" => {
        let button = command_list.remove(0);
        let frames: u64 = command_list.remove(0).parse().unwrap();
        tap(nes, button.as_ref(), frames);
      }
      "screenshot" => {
        let cartridge_path = command_list.remove(0);
        save_screenshot(nes, cartridge_path.as_ref());
      },
      "blargg" => {
        let output_path = command_list.remove(0);
        save_blargg(nes, output_path.as_ref());
      },
      "fromfile" => {
        let command_file_path = command_list.remove(0);
        command_file(nes, command_file_path.as_ref());
      },
      "#" => {
        // A comment! Everything on this line is discarded
        return;
      }
      "" => {
        // Do nothing. This allows blank lines to exist.
      }
      _ => {
        panic!("Unrecognized command: {}\n\nChaos reigns within\nReflect, repent, and retry\nOrder shall return\n", command);
      }
    }
  }    
}

fn main() {
	let mut args: Vec<_> = env::args().collect();
  if args.len() < 2 {
    panic!("Usage: rusticnes-cli <commands>");
  }

  let mut nes = NesState::new(Box::new(NoneMapper::new()));

  // Pop off the name of the program
  let _ = args.remove(0);

  process_command_list(&mut nes, args);
}
