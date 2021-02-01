extern crate image;
extern crate nfd2;
extern crate sdl2;

extern crate rusticnes_core;
extern crate rusticnes_ui_common;

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
use std::fs::remove_file;

use rusticnes_ui_common::application::RuntimeState as RusticNesRuntimeState;
use rusticnes_ui_common::events;
use rusticnes_ui_common::events::StandardControllerButton;
use rusticnes_ui_common::apu_window::ApuWindow;
use rusticnes_ui_common::cpu_window::CpuWindow;
use rusticnes_ui_common::game_window::GameWindow;
use rusticnes_ui_common::memory_window::MemoryWindow;
use rusticnes_ui_common::ppu_window::PpuWindow;

use cartridge_manager::CartridgeManager;
use platform_window::PlatformWindow;

pub fn dispatch_event(windows: &mut Vec<PlatformWindow>, runtime_state: &mut RusticNesRuntimeState, cartridge_state: &mut CartridgeManager, event: events::Event) -> Vec<events::Event> {
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
  let mut runtime_state = RusticNesRuntimeState::new();
  let mut cartridge_state = CartridgeManager::new();

  let sdl_context = sdl2::init().unwrap();
  let audio_subsystem = sdl_context.audio().unwrap();
  let video_subsystem = sdl_context.video().unwrap();

  let mut windows: Vec<PlatformWindow> = Vec::new();

  // For now, we use index 0 as the "main" window; when this window closes, the application exits.
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(GameWindow::new())));
  windows[0].canvas.window_mut().set_position(WindowPos::Positioned(5), WindowPos::Positioned(40));
  
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(ApuWindow::new())));
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(CpuWindow::new())));
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(MemoryWindow::new())));
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
    samples: Some(2048)
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
          if sdl_context.keyboard().focused_window_id().is_some() {
            let focused_window_id = sdl_context.keyboard().focused_window_id().unwrap();
            let mut application_focused = false;
            for i in 0 .. windows.len() {
              if windows[i].canvas.window().id() == focused_window_id {
                application_focused = true;
              }
            }

            // Global events, we want to always handle these even if the application is not focused. Note that
            // unfocused mouse handling appears to be inconsistent between platforms, we're choosing to let SDL
            // dictate this behavior to hopefully match platform conventions.
            match event {
              Event::MouseButtonDown{ window_id: id, mouse_btn: MouseButton::Left, x: omx, y: omy, .. } => {
                for i in 0 .. windows.len() {
                  if id == windows[i].canvas.window().id() {
                    let wx = omx / windows[i].panel.scale_factor() as i32;
                    let wy = omy / windows[i].panel.scale_factor() as i32;
                    windows[i].panel.handle_event(&runtime_state, events::Event::MouseClick(wx, wy));
                  }
                }
              },
              Event::Window { window_id: id, win_event: WindowEvent::Close, .. } => {
                for i in 0 .. windows.len() {
                  if id == windows[i].canvas.window().id() {
                    windows[i].panel.handle_event(&runtime_state, events::Event::CloseWindow);
                  }
                }
              },
              _ => {}
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

                      Keycode::Num5 => {application_events.push(events::Event::ApuTogglePulse1);},
                      Keycode::Num6 => {application_events.push(events::Event::ApuTogglePulse2);},
                      Keycode::Num7 => {application_events.push(events::Event::ApuToggleTriangle);},
                      Keycode::Num8 => {application_events.push(events::Event::ApuToggleNoise);},
                      Keycode::Num9 => {application_events.push(events::Event::ApuToggleDmc);},

                      Keycode::F1 => {application_events.push(events::Event::ShowPpuWindow);},
                      Keycode::F2 => {application_events.push(events::Event::ShowApuWindow);},
                      Keycode::F3 => {application_events.push(events::Event::ShowMemoryWindow);},
                      Keycode::F4 => {application_events.push(events::Event::ShowCpuWindow);},

                      Keycode::Period => {application_events.push(events::Event::MemoryViewerNextPage);},
                      Keycode::Comma => {application_events.push(events::Event::MemoryViewerPreviousPage);},
                      Keycode::Slash => {application_events.push(events::Event::MemoryViewerNextBus);},

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
                      Keycode::KpMultiply=> {application_events.push(events::Event::GameToggleOverscan);},
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
    if runtime_state.running {
      application_events.push(events::Event::NesRunFrame);
    }

    // Process all the application-level events
    let events_to_process = application_events.clone();
    application_events.clear();
    for event in events_to_process{
      application_events.extend(dispatch_event(&mut windows, &mut runtime_state, &mut cartridge_state, event));
    }

    // Update window sizes
    for i in 0 .. windows.len() {
      if windows[i].needs_resize() {
        let (wx, wy) = windows[i].size();
        let _ = windows[i].canvas.window_mut().set_size(wx, wy);

        let tx = windows[i].panel.active_canvas().width;
        let ty = windows[i].panel.active_canvas().height;
        textures[i] = texture_creators[i].create_texture(PixelFormatEnum::ABGR8888, TextureAccess::Streaming, tx, ty).unwrap()
      }
    }

    application_events.extend(dispatch_event(&mut windows, &mut runtime_state, &mut cartridge_state, events::Event::Update));

    // Play Audio
    if runtime_state.nes.apu.buffer_full {
      let mut buffer = [0i16; 4096];
      for i in 0 .. 4096 {
        buffer[i] = runtime_state.nes.apu.output_buffer[i] as i16;
      }
      device.queue(&buffer);
      runtime_state.nes.apu.buffer_full = false;
      if dump_audio {
        runtime_state.nes.apu.dump_sample_buffer();
      }
    }

    // Draw all windows
    for i in 0 .. windows.len() {
      if windows[i].panel.shown() {
        windows[i].panel.handle_event(&runtime_state, events::Event::RequestFrame);
        windows[i].canvas.set_draw_color(Color::RGB(255, 255, 255));
        let _ = textures[i].update(None, &windows[i].panel.active_canvas().buffer, (windows[i].panel.active_canvas().width * 4) as usize);
        let _ = windows[i].canvas.copy(&textures[i], None, None);
        windows[i].canvas.present();
        windows[i].canvas.window_mut().show();
      } else {
        windows[i].canvas.window_mut().hide();
      }
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
}

