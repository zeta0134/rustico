extern crate nfd;
extern crate sdl2;

extern crate rusticnes_core;

mod audio_window;
mod game_window;
mod vram_window;

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
    let mut vram_window = vram_window::VramWindow::new(&sdl_context);

    let mut event_pump = sdl_context.event_pump().unwrap();

    // Audio!
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(2048)
    };

    let device = audio_subsystem.open_queue::<i16, _>(None, &desired_spec).unwrap();
    device.clear();
    device.resume();

    let mut ctrl_mod = false;

    'running: loop {
        if !game_window.shown {
            break 'running
        }
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
                        if vram_window.canvas.window().id() == focused_window_id {
                            vram_window.handle_event(&mut nes, &event);
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
                                    Keycode::O => { game_window.open_file_dialog(&mut nes); },
                                    _ => ()
                                }
                            } else {
                                match key {
                                    Keycode::F1 => {
                                        if vram_window.shown {
                                            vram_window.shown = false;
                                            vram_window.canvas.window_mut().hide();
                                        } else {
                                            vram_window.shown = true;
                                            vram_window.canvas.window_mut().show();
                                        }
                                    },
                                    Keycode::F2 => {
                                        if audio_window.shown {
                                            audio_window.shown = false;
                                            audio_window.canvas.window_mut().hide();
                                        } else {
                                            audio_window.shown = true;
                                            audio_window.canvas.window_mut().show();
                                        }
                                    },
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
        if game_window.shown {
            game_window.update(&mut nes);
        }
        if audio_window.shown {
            audio_window.update(&mut nes);
        }
        if vram_window.shown {
            vram_window.update(&mut nes);
        }

        // Play Audio
        if nes.apu.buffer_full {
            let mut buffer = [0i16; 4096];
            for i in 0 .. 4096 {
                buffer[i] = nes.apu.output_buffer[i] as i16;
            }
            device.queue(&buffer);
            nes.apu.buffer_full = false;
        }

        // Draw all windows
        if game_window.shown {
            game_window.draw();
        }
        if audio_window.shown {
            audio_window.draw();
        }
        if vram_window.shown {
            vram_window.draw();
        }

    }
}