extern crate image;
extern crate rusticnes_core;

use rusticnes_core::nes::NesState;
use rusticnes_core::mmc::none::NoneMapper;
use rusticnes_core::palettes::NTSC_PAL;

use std::env;
use std::fs::File;
use std::str;

use std::error::Error;
use std::io::Read;
use std::io::Write;

fn load_cartridge(nes: &mut NesState, cartridge_path: &str) {
  // Read in the ROM file and attempt to create a new NesState:
  let file = File::open(cartridge_path);
  match file {
    Err(why) => {
      panic!("Couldn't open {}: {}", cartridge_path, why.description());
    },
    Ok(_) => (),
  };

  // Read the whole thing
  let mut cartridge = Vec::new();
  match file.unwrap().read_to_end(&mut cartridge) {
    Err(why) => {
      panic!("Couldn't read from {}: {}", cartridge_path, why.description());
    },
    Ok(_) => {
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

  let ref mut fout = File::create(output_path).unwrap();
  image::ImageRgba8(img).save(fout, image::PNG).unwrap();
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
    } else {
      let ref mut file = File::create(output_filename).unwrap();
      let _ = file.write_all(format!("Invalid blargg magic header, found 0x{:02X} 0x{:02X} 0x{:02X} instead.", magic_0, magic_1, magic_2).as_ref());
    }    
  } else {
    panic!("Cannot output blargg data, ROM has no SRAM!");
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

  while args.len() > 0 {
    let command = args.remove(0);
    match command.as_ref() {
      "cart" | "cartridge" | "rom" => {
        let cartridge_path = args.remove(0);
        load_cartridge(&mut nes, cartridge_path.as_ref());
      },
      "run" | "frames" => {
        let frames: u64 = args.remove(0).parse().unwrap();
        run(&mut nes, frames);
      },
      "screenshot" => {
        let cartridge_path = args.remove(0);
        save_screenshot(&nes, cartridge_path.as_ref());
      },
      "blargg" => {
        let output_path = args.remove(0);
        save_blargg(&mut nes, output_path.as_ref());
      },
      _ => {
        panic!("Unrecognized command: {}\n\nChaos reigns within\nReflect, repent, and retry\nOrder shall return\n", command);
      }
    }
  }    
}
