extern crate image;
extern crate nfd;
extern crate sdl2;

extern crate rusticnes_core;
extern crate rusticnes_ui_common;

mod game_window;
mod piano_roll_window;
mod platform_window;

use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureAccess;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;

use std::env;
use std::fs::File;
use std::fs::remove_file;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use rusticnes_ui_common::application::RuntimeState as RusticNesRuntimeState;
use rusticnes_ui_common::events;
use rusticnes_ui_common::apu_window::ApuWindow;
use rusticnes_ui_common::cpu_window::CpuWindow;
use rusticnes_ui_common::memory_window::MemoryWindow;
use rusticnes_ui_common::test_window::TestWindow;
use rusticnes_ui_common::ppu_window::PpuWindow;

use platform_window::PlatformWindow;

pub struct SdlCartridgeManager {
  pub game_path: String,
  pub sram_path: String,
}

impl SdlCartridgeManager {
  pub fn new() -> SdlCartridgeManager {
    return SdlCartridgeManager {
      game_path: String::from(""),
      sram_path: String::from(""),
    }
  }

  pub fn open_cartridge_with_sram(&mut self, file_path: &str) -> events::Event {
    match std::fs::read(file_path) {
      Ok(cartridge_data) => {
        let cartridge_path = PathBuf::from(file_path);
        let sram_path = cartridge_path.with_extension("sav");
        match std::fs::read(&sram_path.to_str().unwrap()) {
          Ok(sram_data) => {
            return events::Event::LoadCartridge(file_path.to_string(), Rc::new(cartridge_data), Rc::new(sram_data));
          },
          Err(reason) => {
            println!("Failed to load SRAM: {}", reason);
            println!("Continuing anyway.");
            let bucket_of_nothing: Vec<u8> = Vec::new();
            return events::Event::LoadCartridge(file_path.to_string(), Rc::new(cartridge_data), Rc::new(bucket_of_nothing));
          }
        }
      },
      Err(reason) => {
        println!("{}", reason);
        return events::Event::LoadFailed(reason.to_string());
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

  pub fn handle_event(&mut self, event: events::Event) -> Vec<events::Event> {
    let mut responses: Vec<events::Event> = Vec::new();
    match event {
      events::Event::RequestCartridgeDialog => {
        match open_file_dialog() {
          Ok(file_path) => {
            responses.push(events::Event::RequestSramSave(self.sram_path.clone()));
            responses.push(self.open_cartridge_with_sram(&file_path));
          },
          Err(reason) => {
            println!("{}", reason);
            responses.push(events::Event::LoadFailed(reason));
          }
        }
      },
      events::Event::CartridgeLoaded(cart_id) => {
        self.game_path = cart_id.to_string();
        self.sram_path = PathBuf::from(cart_id).with_extension("sav").to_str().unwrap().to_string();
        println!("Cartridge loading success! Storing save path as: {}", self.sram_path);
      },
      events::Event::LoadFailed(reason) => {
        println!("Loading failed: {}", reason);
      },
      events::Event::CartridgeRejected(cart_id, reason) => {
        println!("Cartridge {} could not be played: {}", cart_id, reason);
      },
      events::Event::SaveSram(sram_id, sram_data) => {
        self.save_sram(sram_id, &sram_data);
      },
      _ => {}
    }
    return responses;
  }
}

pub fn dispatch_event(windows: &mut Vec<PlatformWindow>, runtime_state: &mut RusticNesRuntimeState, cartridge_state: &mut SdlCartridgeManager, event: events::Event) -> Vec<events::Event> {
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

pub fn open_file_dialog() -> Result<String, String> {
  let result = nfd::dialog().filter("nes").open().unwrap_or_else(|e| { panic!(e); });

  match result {
    nfd::Response::Okay(file_path) => {
      return Ok(file_path);
    },
    nfd::Response::OkayMultiple(_files) => return Err(String::from("Unexpected multiple files.")),
    nfd::Response::Cancel => return Err(String::from("No file opened.")),
  }
}

pub fn main() {
  let mut runtime_state = RusticNesRuntimeState::new();
  let mut cartridge_state = SdlCartridgeManager::new();

  let sdl_context = sdl2::init().unwrap();
  let audio_subsystem = sdl_context.audio().unwrap();
  let video_subsystem = sdl_context.video().unwrap();

  let mut windows: Vec<PlatformWindow> = Vec::new();

  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(ApuWindow::new())));
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(CpuWindow::new())));
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(MemoryWindow::new())));
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(PpuWindow::new())));
  windows.push(PlatformWindow::from_panel(&video_subsystem, Box::new(TestWindow::new())));

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

  // Setup the main window for rendering
  let sdl_game_window = video_subsystem.window("RusticNES NEW", (256 - 16) * 2, (240 - 16) * 2)
    .position(5, 40)
    .opengl()
    .build()
    .unwrap();
  let mut game_canvas = sdl_game_window.into_canvas().present_vsync().build().unwrap();
  game_canvas.set_draw_color(Color::RGB(0, 0, 0));
  game_canvas.clear();
  game_canvas.present();
  let game_screen_texture_creator = game_canvas.texture_creator();
  let mut game_screen_texture = game_screen_texture_creator.create_texture(PixelFormatEnum::RGBA8888, TextureAccess::Streaming, 256, 240).unwrap();
  let mut game_window = game_window::GameWindow::new();

  // Setup various debug windows
  let mut piano_roll_window = piano_roll_window::PianoRollWindow::new();
  let sdl_piano_roll_window = video_subsystem.window("Piano Roll", piano_roll_window.buffer.width, piano_roll_window.buffer.height)
    .position(490, 40)
    .hidden()
    .opengl()
    .build()
    .unwrap();
  let mut piano_roll_canvas = sdl_piano_roll_window.into_canvas().build().unwrap();
  piano_roll_canvas.set_draw_color(Color::RGB(0, 0, 0));
  piano_roll_canvas.clear();
  piano_roll_canvas.present();
  let piano_roll_texture_creator = piano_roll_canvas.texture_creator();
  let mut piano_roll_screen_texture = piano_roll_texture_creator.create_texture(PixelFormatEnum::ABGR8888, TextureAccess::Streaming, piano_roll_window.buffer.width, piano_roll_window.buffer.height).unwrap();

  let mut ctrl_mod = false;
  let mut trigger_resize = false;
  let mut dump_audio = false;

  let args: Vec<_> = env::args().collect();
  if args.len() > 1 {
    application_events.push(cartridge_state.open_cartridge_with_sram(&args[1]));
  }

  'running: loop {
    if !game_window.shown {
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
            let mut application_focused = 
              game_canvas.window().id() == focused_window_id ||
              piano_roll_canvas.window().id() == focused_window_id;
            for i in 0 .. windows.len() {
              if windows[i].canvas.window().id() == focused_window_id {
                application_focused = true;
              }
            }

            if application_focused {
              match event {
                Event::KeyDown { keycode: Some(key), .. } => {
                  // Pass keydown events into sub-windows
                  game_window.handle_key_down(&mut runtime_state.nes, key);
                  // Handle global keydown events
                  if key == Keycode::LCtrl || key == Keycode::RCtrl {
                    ctrl_mod = true;
                  }
                },
                Event::KeyUp { keycode: Some(key), .. } => {
                  // Pass keyup events into sub-windows
                  game_window.handle_key_up(&mut runtime_state.nes, key);
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
                      _ => ()
                    }
                  } else {
                    // Previous implementation handled debug window showing / hiding here
                    match key {
                      Keycode::Num5 => {application_events.push(events::Event::ApuTogglePulse1);},
                      Keycode::Num6 => {application_events.push(events::Event::ApuTogglePulse2);},
                      Keycode::Num7 => {application_events.push(events::Event::ApuToggleTriangle);},
                      Keycode::Num8 => {application_events.push(events::Event::ApuToggleNoise);},
                      Keycode::Num9 => {application_events.push(events::Event::ApuToggleDmc);},

                      Keycode::F1 => {application_events.push(events::Event::ShowPpuWindow);},
                      Keycode::F2 => {application_events.push(events::Event::ShowApuWindow);},
                      Keycode::F3 => {application_events.push(events::Event::ShowMemoryWindow);},
                      Keycode::F4 => {application_events.push(events::Event::ShowCpuWindow);},
                      Keycode::F6 => {application_events.push(events::Event::ShowTestWindow);},

                      Keycode::Period => {application_events.push(events::Event::MemoryViewerNextPage);},
                      Keycode::Comma => {application_events.push(events::Event::MemoryViewerPreviousPage);},
                      Keycode::Slash => {application_events.push(events::Event::MemoryViewerNextBus);},

                      Keycode::S => {application_events.push(events::Event::RequestSramSave(cartridge_state.sram_path.clone()));},

                      Keycode::F5 => {
                        if !piano_roll_window.shown {
                          piano_roll_window.shown = true;
                          piano_roll_canvas.window_mut().show();
                        } else {
                          piano_roll_window.shown = false;
                          piano_roll_canvas.window_mut().hide();
                        }
                      },
                      Keycode::Equals | Keycode::KpPlus | Keycode::Plus => {
                        if game_window.scale < 8 {
                          game_window.scale += 1;
                          trigger_resize = true;
                        }
                      },
                      Keycode::KpMinus | Keycode::Minus => {
                        if game_window.scale > 1 {
                          game_window.scale -= 1;
                          trigger_resize = true;
                        }
                      },
                      Keycode::KpMultiply=> {
                        game_window.display_overscan = !game_window.display_overscan;
                        trigger_resize = true;
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
                  }
                },
                Event::MouseButtonDown{ window_id: id, mouse_btn: MouseButton::Left, x: omx, y: omy, .. } => {
                  for i in 0 .. windows.len() {
                    if id == windows[i].canvas.window().id() {
                      //memory_window.handle_click(&mut runtime_state.nes, omx / 2, omy / 2);
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
                  if id == game_canvas.window().id() {
                    game_window.shown = false;
                    game_canvas.window_mut().hide();
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

    // Update windows
    if game_window.shown {
      game_window.update(&mut runtime_state.nes);

      if trigger_resize {
        trigger_resize = false;
        if game_window.display_overscan {
          let _ = game_canvas.window_mut().set_size(game_window.scale * 256, game_window.scale * 240);
        } else {
          let _ = game_canvas.window_mut().set_size(game_window.scale * (256 - 16), game_window.scale * (240 - 16));
        }
      }
    } else {
      // The main game window was closed! Exit the program.
      break 'running
    }
    if piano_roll_window.shown {
      piano_roll_window.update(&mut runtime_state.nes);
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
    if piano_roll_window.shown {
      piano_roll_canvas.set_draw_color(Color::RGB(255, 255, 255));
      let _ = piano_roll_screen_texture.update(None, &piano_roll_window.buffer.buffer, (piano_roll_window.buffer.width * 4) as usize);
      let _ = piano_roll_canvas.copy(&piano_roll_screen_texture, None, None);
      piano_roll_canvas.present();
    }

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

    game_canvas.set_draw_color(Color::RGB(255, 255, 255));
    let _ = game_screen_texture.update(None, &game_window.screen_buffer, 256 * 4);
    if game_window.display_overscan {
      let _ = game_canvas.copy(&game_screen_texture, None, None);
    } else {
      let borderless_rectangle = Rect::new(8, 8, 256 - 16, 240 - 16);
      let _ = game_canvas.copy(&game_screen_texture, borderless_rectangle, None);
    }
    game_canvas.present();
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

