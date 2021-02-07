use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use rusticnes_ui_common;

pub struct CartridgeManager {
  pub game_path: String,
  pub sram_path: String,
}

impl CartridgeManager {
  pub fn new() -> CartridgeManager {
    return CartridgeManager {
      game_path: String::from(""),
      sram_path: String::from(""),
    }
  }

  pub fn open_cartridge_with_sram(&mut self, file_path: &str) -> rusticnes_ui_common::Event {
    match std::fs::read(file_path) {
      Ok(cartridge_data) => {
        let cartridge_path = PathBuf::from(file_path);
        let sram_path = cartridge_path.with_extension("sav");
        match std::fs::read(&sram_path.to_str().unwrap()) {
          Ok(sram_data) => {
            return rusticnes_ui_common::Event::LoadCartridge(file_path.to_string(), Rc::new(cartridge_data), Rc::new(sram_data));
          },
          Err(reason) => {
            println!("Failed to load SRAM: {}", reason);
            println!("Continuing anyway.");
            let bucket_of_nothing: Vec<u8> = Vec::new();
            return rusticnes_ui_common::Event::LoadCartridge(file_path.to_string(), Rc::new(cartridge_data), Rc::new(bucket_of_nothing));
          }
        }
      },
      Err(reason) => {
        println!("{}", reason);
        return rusticnes_ui_common::Event::LoadFailed(reason.to_string());
      }
    }
  }

  pub fn save_sram(&self, filename: String, sram_data: &[u8]) {
    let file = File::create(filename);
    match file {
        Err(why) => {
            println!("Couldn't open {}: {}", self.sram_path, why.to_string());
        },
        Ok(mut file) => {
            let _ = file.write_all(sram_data);
            println!("Wrote sram data to: {}", self.sram_path);
        },
    };
  }

  pub fn handle_event(&mut self, event: rusticnes_ui_common::Event) -> Vec<rusticnes_ui_common::Event> {
    let mut responses: Vec<rusticnes_ui_common::Event> = Vec::new();
    match event {
      rusticnes_ui_common::Event::RequestCartridgeDialog => {
        match open_file_dialog() {
          Ok(file_path) => {
            responses.push(rusticnes_ui_common::Event::RequestSramSave(self.sram_path.clone()));
            responses.push(self.open_cartridge_with_sram(&file_path));
          },
          Err(reason) => {
            println!("{}", reason);
            responses.push(rusticnes_ui_common::Event::LoadFailed(reason));
          }
        }
      },
      rusticnes_ui_common::Event::CartridgeLoaded(cart_id) => {
        self.game_path = cart_id.to_string();
        self.sram_path = PathBuf::from(cart_id).with_extension("sav").to_str().unwrap().to_string();
        println!("Cartridge loading success! Storing save path as: {}", self.sram_path);
      },
      rusticnes_ui_common::Event::LoadFailed(reason) => {
        println!("Loading failed: {}", reason);
      },
      rusticnes_ui_common::Event::CartridgeRejected(cart_id, reason) => {
        println!("Cartridge {} could not be played: {}", cart_id, reason);
      },
      rusticnes_ui_common::Event::SaveSram(sram_id, sram_data) => {
        self.save_sram(sram_id, &sram_data);
      },
      _ => {}
    }
    return responses;
  }
}

pub fn open_file_dialog() -> Result<String, String> {
  let result = nfd2::dialog().filter("nes").open().unwrap_or_else(|e| { panic!(e); });

  match result {
    nfd2::Response::Okay(file_path) => {
      return Ok(file_path.to_str().unwrap().to_string());
    },
    nfd2::Response::OkayMultiple(_files) => return Err(String::from("Unexpected multiple files.")),
    nfd2::Response::Cancel => return Err(String::from("No file opened.")),
  }
}