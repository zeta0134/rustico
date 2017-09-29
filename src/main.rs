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

pub fn main() {
    let mut nes = NesState::new(Box::new(NoneMapper::new()));

    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let keyboard = sdl_context.keyboard();

    let mut game_window = game_window::GameWindow::new(&sdl_context);
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Audio!
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(1024)
    };

    let device = audio_subsystem.open_queue::<u16, _>(None, &desired_spec).unwrap();
    device.clear();
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

        // Play Audio
        if nes.apu.buffer_full {
            device.queue(&nes.apu.output_buffer);
            nes.apu.buffer_full = false;
        }

        // Draw all windows
        game_window.draw();

    }
}