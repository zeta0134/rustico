extern crate nfd;
extern crate sdl2;

extern crate rusticnes_core;

mod audio_window;
mod game_window;

use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use rusticnes_core::nes::NesState;
use rusticnes_core::mmc::none::NoneMapper;

pub fn main() {
    let mut nes = NesState::new(Box::new(NoneMapper::new()));

    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let mut audio_window = audio_window::AudioWindow::new(&sdl_context);
    let mut game_window = game_window::GameWindow::new(&sdl_context);

    let mut event_pump = sdl_context.event_pump().unwrap();

    // Audio!
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(2048)
    };

    let device = audio_subsystem.open_queue::<u16, _>(None, &desired_spec).unwrap();
    device.clear();
    device.resume();

    let mut ctrl_mod = false;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {
                    // Pass events to sub-windows
                    if sdl_context.keyboard().focused_window_id().is_some() {
                        let focused_window_id = sdl_context.keyboard().focused_window_id().unwrap();
                        if game_window.canvas.window().id() == focused_window_id {
                            game_window.handle_event(&mut nes, &event);
                        }
                        if audio_window.canvas.window().id() == focused_window_id {
                            audio_window.handle_event(&mut nes, &event);
                        }
                    }

                    // Handle global keypress events here
                    match event {
                        Event::KeyDown { keycode: Some(key), .. } => {
                            if key == Keycode::LCtrl || key == Keycode::RCtrl {
                                ctrl_mod = true;
                            }
                            if ctrl_mod {
                                match key {
                                    Keycode::Q => { break 'running },
                                    _ => ()
                                }
                            }
                        },
                        Event::KeyUp { keycode: Some(key), .. } => {
                            if key == Keycode::LCtrl || key == Keycode::RCtrl {
                                ctrl_mod = false;
                            }
                        },
                        _ => () 
                    }
                }
            }
        }

        // Update all windows
        game_window.update(&mut nes);
        audio_window.update(&mut nes);

        // Play Audio
        if nes.apu.buffer_full {
            device.queue(&nes.apu.output_buffer);
            nes.apu.buffer_full = false;
        }

        // Draw all windows
        game_window.draw();
        audio_window.draw();

    }
}