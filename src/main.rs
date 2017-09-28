extern crate nfd;
extern crate sdl2;

extern crate rusticnes_core;

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
    let mut running = false;

    let result = nfd::open_file_dialog(None, None).unwrap_or_else(|e| {
        panic!(e);
    });

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
                    running = true;

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



    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let keyboard = sdl_context.keyboard();

    let game_window = video_subsystem.window("Game Window", 512, 480)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut game_canvas = game_window.into_canvas().present_vsync().build().unwrap();
    game_canvas.set_draw_color(Color::RGB(192, 96, 96));
    game_canvas.clear();
    game_canvas.present();

    let mut game_screen_texture_creator = game_canvas.texture_creator();
    let mut game_screen_texture = game_screen_texture_creator.create_texture(PixelFormatEnum::RGBA8888, TextureAccess::Streaming, 256, 240).unwrap();

    let mut game_screen_buffer = [0u8; 256 * 240 * 4];
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut frame_counter = 0;

    // Audio!
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(1024)
    };

    let device = audio_subsystem.open_queue::<u16, _>(None, &desired_spec).unwrap();
    device.resume();

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

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(key), .. } => {
                    if sdl_context.keyboard().focused_window_id().is_some() {
                        let focused_window_id = sdl_context.keyboard().focused_window_id().unwrap();
                        if game_canvas.window().id() == focused_window_id {
                            for i in 0 .. 8 {
                                if key == key_mappings[i] {
                                    // Set the corresponding bit
                                    nes.p1_input |= 0x1 << i;
                                }
                            }
                            match key {
                                Keycode::R => {running = !running;},
                                _ => ()
                            }
                        }
                    }
                },
                Event::KeyUp { keycode: Some(key), .. } => {
                    if sdl_context.keyboard().focused_window_id().is_some() {
                        let focused_window_id = sdl_context.keyboard().focused_window_id().unwrap();
                        if game_canvas.window().id() == focused_window_id {
                            for i in 0 .. 8 {
                                if key == key_mappings[i] {
                                    // Clear the corresponding bit
                                    nes.p1_input &= (0x1 << i) ^ 0xFF;
                                }
                            }
                        }
                    }
                },
                _ => {}
            }
        }

        // Run the NES game loop if a cartridge is loaded
        if running {
            nes::run_until_vblank(&mut nes);

            // Update the game screen
            for x in 0 .. 256 {
                for y in 0 .. 240 {
                    let palette_index = ((nes.ppu.screen[y * 256 + x]) as usize) * 3;
                    game_screen_buffer[((y * 256 + x) * 4) + 3] = NTSC_PAL[palette_index + 0];
                    game_screen_buffer[((y * 256 + x) * 4) + 2] = NTSC_PAL[palette_index + 1];
                    game_screen_buffer[((y * 256 + x) * 4) + 1] = NTSC_PAL[palette_index + 2];
                    game_screen_buffer[((y * 256 + x) * 4) + 0] = 255;
                }
            }
        }

        // Delay for 1 / 60th of a frame, turned off for now. I think this
        // causes SDL to either vsync, or run unchecked. Need to investigate
        // this later, and figure out the best way to target 60 FPS in a
        // cross-platform friendly manner.
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));

        if nes.apu.buffer_full {
            device.queue(&nes.apu.output_buffer);
            nes.apu.buffer_full = false;
        }

        game_screen_texture.update(None, &game_screen_buffer, 256 * 4);

        game_canvas.set_draw_color(Color::RGB(0, 0, 0));
        game_canvas.clear();
        game_canvas.set_draw_color(Color::RGB(255, 255, 255));
        game_canvas.copy(&game_screen_texture, None, None);
        game_canvas.present();

        frame_counter += 1;
    }
}