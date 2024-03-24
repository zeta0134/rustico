// Don't pop up a console automatically on Windows builds
#![windows_subsystem = "windows"]

extern crate dirs;
extern crate image;
extern crate nfd2;
extern crate sdl2;

extern crate rustico_core;
extern crate rustico_ui_common;

mod cartridge_manager;
mod platform_window;

use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::render::TextureAccess;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use sdl2::video::WindowPos;

use std::env;
use std::fs;
use std::fs::remove_file;
use std::thread;
use std::time;
use std::ffi::OsString;

use rustico_ui_common::application::RuntimeState as RusticoRuntimeState;
use rustico_ui_common::events;
use rustico_ui_common::events::StandardControllerButton;
use rustico_ui_common::apu_window::ApuWindow;
use rustico_ui_common::cpu_window::CpuWindow;
use rustico_ui_common::game_window::GameWindow;
use rustico_ui_common::event_window::EventWindow;
use rustico_ui_common::memory_window::MemoryWindow;
use rustico_ui_common::piano_roll_window::PianoRollWindow;
use rustico_ui_common::ppu_window::PpuWindow;

use cartridge_manager::CartridgeManager;
use platform_window::PlatformWindow;

pub fn dispatch_event(windows: &mut Vec<PlatformWindow>, runtime_state: &mut RusticoRuntimeState, cartridge_state: &mut CartridgeManager, event: events::Event) -> Vec<events::Event> {
  let mut responses: Vec<events::Event> = Vec::new();
  for i in 0 .. windows.len() {
    // Note: Windows get an immutable reference to everything other than themselves
    responses.extend(windows[i].panel.handle_event(&runtime_state, event.clone()));
  }
  // ... but RuntimeState needs a mutable reference to itself
  responses.extend(runtime_state.handle_event(event.clone()));
  // Platform specific state, this is not passed to applications on purpose
  responses.extend(cartridge_state.handle_event(event.clone()));
  return responses;
}

pub fn main() {
  let version = env!("CARGO_PKG_VERSION");
  println!("Welcome to Rustico {}", version);

  let config_path: OsString = match dirs::config_dir() {
    Some(mut path) => {
      path.push("rustico");
      match fs::create_dir_all(&path) {
        Ok(_) => {},
        Err(e) => {println!("ERROR: {}\nFailed to create settings dir {}, settings will likely fail to save!", e, path.display())}
      };
      path.push("settings.toml");
      path.into_os_string()
    },
    None => {"rustico_settings.toml".into()}
  };

  let mut runtime_state = RusticoRuntimeState::new();
  let mut cartridge_state = CartridgeManager::new();
  runtime_state.settings.load(&config_path);

  let sdl_context = sdl2::init().unwrap();
  let audio_subsystem = sdl_context.audio().unwrap();
  let video_subsystem = sdl_context.video().unwrap();

  let mut windows: Vec<PlatformWindow> = Vec::new();

  // For now, we use index 0 as the "main" window; when this window closes, the application exits.
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(GameWindow::new())));
  windows[0].canvas.window_mut().set_position(WindowPos::Positioned(5), WindowPos::Positioned(40));
  
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(ApuWindow::new())));
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(CpuWindow::new())));
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(EventWindow::new())));
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(MemoryWindow::new())));
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(PianoRollWindow::new())));
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(PpuWindow::new())));

  let mut texture_creators: Vec<TextureCreator<WindowContext>> = Vec::new();
  for i in 0 .. windows.len() {
    texture_creators.push(windows[i].canvas.texture_creator());
  }

  let mut textures: Vec<Texture> = Vec::new();
  for i in 0 .. windows.len() {
    let width = windows[i].panel.active_canvas().width;
    let height = windows[i].panel.active_canvas().height;
    textures.push(texture_creators[i].create_texture(PixelFormatEnum::ABGR8888, TextureAccess::Streaming, width, height).unwrap());  
  }

  let mut application_events: Vec<events::Event> = Vec::new();
  let mut event_pump = sdl_context.event_pump().unwrap();

  // Setup Audio output format and sample rate
  let desired_spec = AudioSpecDesired {
    freq: Some(44100),
    channels: Some(1),
    samples: Some(256)
  };

  // Grab the active audio device and begin playback immediately. Until we fill the buffer, this will "play" silence:
  let device = audio_subsystem.open_queue::<i16, _>(None, &desired_spec).unwrap();
  device.clear();
  device.resume();

  let mut ctrl_mod = false;
  let mut dump_audio = false;

  let args: Vec<_> = env::args().collect();
  if args.len() > 1 {
    application_events.push(cartridge_state.open_cartridge_with_sram(&args[1]));
  }

  // Apply settings (default or otherwise)
  application_events.extend(runtime_state.settings.apply_settings());

  'running: loop {
    if !windows[0].panel.shown() {
      break 'running
    }

    // Process all incoming SDL events
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit {..} => {
          break 'running
        },
        _ => {
          // Global events, we want to always handle these even if the application is not focused. Note that
          // unfocused mouse handling appears to be inconsistent between platforms, we're choosing to let SDL
          // dictate this behavior to hopefully match platform conventions.
          match event {
            Event::MouseButtonDown{ window_id: id, mouse_btn: MouseButton::Left, x: omx, y: omy, .. } => {
              for i in 0 .. windows.len() {
                if id == windows[i].canvas.window().id() {
                  let wx = omx / windows[i].panel.scale_factor() as i32;
                  let wy = omy / windows[i].panel.scale_factor() as i32;
                  application_events.extend(windows[i].panel.handle_event(&runtime_state, events::Event::MouseClick(wx, wy)));
                }
              }
            },
            Event::MouseMotion{ window_id: id, x: omx, y: omy, .. } => {
              for i in 0 .. windows.len() {
                if id == windows[i].canvas.window().id() {
                  let wx = omx / windows[i].panel.scale_factor() as i32;
                  let wy = omy / windows[i].panel.scale_factor() as i32;
                  application_events.extend(windows[i].panel.handle_event(&runtime_state, events::Event::MouseMove(wx, wy)));
                }
              }
            },
            Event::Window { window_id: id, win_event: WindowEvent::Close, .. } => {
              for i in 0 .. windows.len() {
                if id == windows[i].canvas.window().id() {
                  application_events.extend(windows[i].panel.handle_event(&runtime_state, events::Event::CloseWindow));
                }
              }
            },
            _ => {}
          }

          if sdl_context.keyboard().focused_window_id().is_some() {
            let focused_window_id = sdl_context.keyboard().focused_window_id().unwrap();
            let mut application_focused = false;
            for i in 0 .. windows.len() {
              if windows[i].canvas.window().id() == focused_window_id {
                application_focused = true;
              }
            }

            // Focus-filtered events, typically keybindings and such.
            if application_focused {
              match event {
                Event::KeyDown { keycode: Some(key), .. } => {
                  // Handle global keydown events
                  if key == Keycode::LCtrl || key == Keycode::RCtrl {
                    ctrl_mod = true;
                  }

                  match key {
                    Keycode::X =>      {application_events.push(events::Event::StandardControllerPress(0, StandardControllerButton::A))},
                    Keycode::Z =>      {application_events.push(events::Event::StandardControllerPress(0, StandardControllerButton::B))},
                    Keycode::RShift => {application_events.push(events::Event::StandardControllerPress(0, StandardControllerButton::Select))},
                    Keycode::Return => {application_events.push(events::Event::StandardControllerPress(0, StandardControllerButton::Start))},
                    Keycode::Up =>     {application_events.push(events::Event::StandardControllerPress(0, StandardControllerButton::DPadUp))},
                    Keycode::Down =>   {application_events.push(events::Event::StandardControllerPress(0, StandardControllerButton::DPadDown))},
                    Keycode::Left =>   {application_events.push(events::Event::StandardControllerPress(0, StandardControllerButton::DPadLeft))},
                    Keycode::Right =>  {application_events.push(events::Event::StandardControllerPress(0, StandardControllerButton::DPadRight))},
                    _ => {}
                  }
                },
                Event::KeyUp { keycode: Some(key), .. } => {
                  // Handle global keydown events
                  if key == Keycode::LCtrl || key == Keycode::RCtrl {
                    ctrl_mod = false;
                  }
                  if ctrl_mod {
                    match key {
                      Keycode::Q => { break 'running },
                      Keycode::O => { 
                        ctrl_mod = false; // the open file dialog suppresses Ctrl release events, so trigger one manually
                        application_events.push(events::Event::RequestCartridgeDialog);
                      },
                      Keycode::A => {
                        dump_audio = !dump_audio;
                        if dump_audio {
                          let _ = remove_file("audiodump.raw");
                          println!("Beginning audio dump...");
                        } else {
                          println!("Audio dump stopped.");
                        }
                      },
                      
                      Keycode::Kp1 => {application_events.push(events::Event::ChangeDisk(0, 0));},
                      Keycode::Kp2 => {application_events.push(events::Event::ChangeDisk(0, 1));},
                      Keycode::Kp3 => {application_events.push(events::Event::ChangeDisk(1, 0));},
                      Keycode::Kp4 => {application_events.push(events::Event::ChangeDisk(1, 1));},
                      Keycode::Kp5 => {application_events.push(events::Event::ChangeDisk(2, 0));},
                      Keycode::Kp6 => {application_events.push(events::Event::ChangeDisk(2, 1));},
                      Keycode::Kp7 => {application_events.push(events::Event::ChangeDisk(3, 0));},
                      Keycode::Kp8 => {application_events.push(events::Event::ChangeDisk(3, 1));},
                      _ => ()
                    }
                  } else {
                    match key {
                      Keycode::Escape => {
                        // Escape closes the active window
                        for i in 0 .. windows.len() {
                          if windows[i].canvas.window().id() == focused_window_id {
                            windows[i].panel.handle_event(&runtime_state, events::Event::CloseWindow);
                          }
                        }
                      },

                      Keycode::F1 => {application_events.push(events::Event::ShowPpuWindow);},
                      Keycode::F2 => {application_events.push(events::Event::ShowApuWindow);},
                      Keycode::F3 => {application_events.push(events::Event::ShowMemoryWindow);},
                      Keycode::F4 => {application_events.push(events::Event::ShowCpuWindow);},
                      Keycode::F5 => {application_events.push(events::Event::ShowPianoRollWindow);},
                      Keycode::F6 => {application_events.push(events::Event::ShowEventWindow);},

                      Keycode::F9 => {application_events.push(events::Event::NesNudgeAlignment);},

                      Keycode::Period => {application_events.push(events::Event::MemoryViewerNextPage);},
                      Keycode::Comma => {application_events.push(events::Event::MemoryViewerPreviousPage);},
                      Keycode::Slash => {application_events.push(events::Event::MemoryViewerNextBus);},

                      Keycode::N => {application_events.push(events::Event::ToggleBooleanSetting("video.ntsc_filter".to_string()));},
                      Keycode::F => {application_events.push(events::Event::ToggleBooleanSetting("video.display_fps".to_string()));},

                      Keycode::S => {application_events.push(events::Event::RequestSramSave(cartridge_state.sram_path.clone()));},

                      Keycode::P => {application_events.push(events::Event::NesToggleEmulation);}
                      Keycode::R => {application_events.push(events::Event::NesReset);}
                      Keycode::Space => {application_events.push(events::Event::NesRunOpcode);},
                      Keycode::C => {application_events.push(events::Event::NesRunCycle);},
                      Keycode::H => {application_events.push(events::Event::NesRunScanline);},
                      Keycode::V => {application_events.push(events::Event::NesRunFrame);},


                      Keycode::X =>      {application_events.push(events::Event::StandardControllerRelease(0, StandardControllerButton::A))},
                      Keycode::Z =>      {application_events.push(events::Event::StandardControllerRelease(0, StandardControllerButton::B))},
                      Keycode::RShift => {application_events.push(events::Event::StandardControllerRelease(0, StandardControllerButton::Select))},
                      Keycode::Return => {application_events.push(events::Event::StandardControllerRelease(0, StandardControllerButton::Start))},
                      Keycode::Up =>     {application_events.push(events::Event::StandardControllerRelease(0, StandardControllerButton::DPadUp))},
                      Keycode::Down =>   {application_events.push(events::Event::StandardControllerRelease(0, StandardControllerButton::DPadDown))},
                      Keycode::Left =>   {application_events.push(events::Event::StandardControllerRelease(0, StandardControllerButton::DPadLeft))},
                      Keycode::Right =>  {application_events.push(events::Event::StandardControllerRelease(0, StandardControllerButton::DPadRight))},

                      Keycode::Equals | Keycode::KpPlus | Keycode::Plus => {application_events.push(events::Event::GameIncreaseScale);},
                      Keycode::KpMinus | Keycode::Minus => {application_events.push(events::Event::GameDecreaseScale);},
                      Keycode::KpMultiply => {application_events.push(events::Event::ToggleBooleanSetting("video.simulate_overscan".to_string()));},
                      _ => ()
                    }
                  }
                },
                _ => ()
              }
            }
          }
        }
      }
    }

    // If we're currently running, emit NesRunFrame events
    // TODO: Move this into some sort of timing manager, deal with real time deltas,
    // and separate these events from the monitor refresh rate.
    let mut new_frames = 0;
    //println!("device queue: {}, emulator queue: {}", device.size(), runtime_state.nes.apu.samples_queued());
    while (device.size() as usize) + (runtime_state.nes.apu.samples_queued() * 2) < 4096 {
      new_frames += 1;
      if runtime_state.running {
        // Play Audio (leave this loop when this buffer fills)
        if runtime_state.nes.apu.buffer_full {
          let buffer_size = runtime_state.nes.apu.output_buffer.len();
          let mut buffer = vec!(0i16; buffer_size);
          for i in 0 .. buffer_size {
            buffer[i] = runtime_state.nes.apu.output_buffer[i] as i16;
          }
          _ = device.queue_audio(&buffer);
          runtime_state.nes.apu.buffer_full = false;
          if dump_audio {
            runtime_state.nes.apu.dump_sample_buffer();
          }
        }

        // Run one frame, by running 262 scanlines (so we can capture events inbetween)
        while runtime_state.nes.ppu.current_scanline == 242 {
          application_events.push(events::Event::NesRunScanline);
          let events_to_process = application_events.clone();
          application_events.clear();
          for event in events_to_process {
            application_events.extend(dispatch_event(&mut windows, &mut runtime_state, &mut cartridge_state, event));
          }
        }
        while runtime_state.nes.ppu.current_scanline != 242 {
          application_events.push(events::Event::NesRunScanline);
          let events_to_process = application_events.clone();
          application_events.clear();
          for event in events_to_process {
            application_events.extend(dispatch_event(&mut windows, &mut runtime_state, &mut cartridge_state, event));
          }
        }
      } else {
        // we have to queue up *something*, so let's target around 60 Hz ish of silence
        let buffer = vec!(0i16; 44100 / 60);
        _ = device.queue_audio(&buffer);
      }

      // Run an update, and also flush out (unconditionally) any other queued events
      application_events.extend(dispatch_event(&mut windows, &mut runtime_state, &mut cartridge_state, events::Event::Update));
      let events_to_process = application_events.clone();
      application_events.clear();
      for event in events_to_process {
        application_events.extend(dispatch_event(&mut windows, &mut runtime_state, &mut cartridge_state, event));
      }
    }

    // only present (and thus vsync) if there are new frames to draw
    if new_frames > 0 {
      // Update window sizes
      for i in 0 .. windows.len() {
        if windows[i].needs_resize() {
          let (wx, wy) = windows[i].window_size();
          let _ = windows[i].canvas.window_mut().set_size(wx, wy);

          let (tx, ty) = windows[i].canvas_size();
          textures[i] = texture_creators[i].create_texture(PixelFormatEnum::ABGR8888, TextureAccess::Streaming, tx, ty).unwrap();
          windows[i].texture_size_x = tx;
          windows[i].texture_size_y = ty;
        }
        // Don't mind these rust-isms
        let borrowed_window = &mut windows[i];
        let title = borrowed_window.panel.title();
        let _ = borrowed_window.canvas.window_mut().set_title(title);
      }

      // Draw all windows
      for i in 0 .. windows.len() {
        if windows[i].panel.shown() {
          application_events.extend(windows[i].panel.handle_event(&runtime_state, events::Event::RequestFrame));
          windows[i].canvas.set_draw_color(Color::RGB(255, 255, 255));
          let _ = textures[i].update(None, &windows[i].panel.active_canvas().buffer, (windows[i].panel.active_canvas().width * 4) as usize);
          let _ = windows[i].canvas.copy(&textures[i], None, None);
          windows[i].canvas.present();
          windows[i].canvas.window_mut().show();
        } else {
          windows[i].canvas.window_mut().hide();
        }
      }
    } else {
      // sleep for a vanishingly tiny amount of time, so that we aren't hammering one CPU core with a constant busy loop
      let sleepy_time = time::Duration::from_micros(100); // 0.1 milliseconds
      thread::sleep(sleepy_time);
    }
  }

  println!("Exiting application! Attempting SRAM save one last time.");
  application_events.push(events::Event::RequestSramSave(cartridge_state.sram_path.clone()));
  while application_events.len() > 0 {
    let events_to_process = application_events.clone();
    application_events.clear();
    for event in events_to_process{
      application_events.extend(dispatch_event(&mut windows, &mut runtime_state, &mut cartridge_state, event));
    }
  }

  runtime_state.settings.save(&config_path);
}

