extern crate nfd;
extern crate sdl2;

extern crate rusticnes_core;

mod game_window;

use nfd::Response;
use sdl2::audio::AudioSpecDesired;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

use rusticnes_core::memory;
use rusticnes_core::nes;
use rusticnes_core::nes::NesState;
use rusticnes_core::mmc::none::NoneMapper;
use rusticnes_core::cartridge;
use rusticnes_core::palettes::NTSC_PAL;

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::time::Duration;

pub fn main() {
    let mut nes = NesState::new(Box::new(NoneMapper::new()));

    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let keyboard = sdl_context.keyboard();

    let mut game_window = game_window::GameWindow::new(&sdl_context);

    let result = nfd::dialog().filter("nes").open().unwrap_or_else(|e| { panic!(e); });

    match result {
        Response::Okay(file_path) => {
            println!("Opened: {:?}", file_path);

            println!("Attempting to load {}...", file_path);

            let mut file = match File::open(file_path) {
                Err(why) => panic!("Couldn't open mario.nes: {}", why.description()),
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
                    nes = NesState::new(mapper);
                    game_window.running = true;

                    // Initialize CPU register state for power-up sequence
                    nes.registers.a = 0;
                    nes.registers.y = 0;
                    nes.registers.x = 0;
                    nes.registers.s = 0xFD;

                    let pc_low = memory::read_byte(&mut nes, 0xFFFC);
                    let pc_high = memory::read_byte(&mut nes, 0xFFFD);
                    nes.registers.pc = pc_low as u16 + ((pc_high as u16) << 8);
                },
            };


        },
        Response::OkayMultiple(files) => println!("Opened: {:?}", files),
        Response::Cancel => println!("No file opened!"),
    }

    let mut event_pump = sdl_context.event_pump().unwrap();

    // Audio!
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(1024)
    };

    let device = audio_subsystem.open_queue::<u16, _>(None, &desired_spec).unwrap();
    device.resume();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {
                    if sdl_context.keyboard().focused_window_id().is_some() {
                        let focused_window_id = sdl_context.keyboard().focused_window_id().unwrap();
                        if game_window.canvas.window().id() == focused_window_id {
                            game_window.handle_event(&mut nes, event);
                        }
                    }
                }
            }
        }

        // Update all windows
        game_window.update(&mut nes);

        // Delay for 1 / 60th of a frame, turned off for now. I think this
        // causes SDL to either vsync, or run unchecked. Need to investigate
        // this later, and figure out the best way to target 60 FPS in a
        // cross-platform friendly manner.
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));

        if nes.apu.buffer_full {
            device.queue(&nes.apu.output_buffer);
            nes.apu.buffer_full = false;
        }

        // Draw all windows
        game_window.draw();
    }
}