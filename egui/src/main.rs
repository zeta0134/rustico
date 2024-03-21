#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use eframe::egui;

#[macro_use]
extern crate lazy_static;
extern crate rusticnes_core;
extern crate rusticnes_ui_common;

use rusticnes_ui_common::application::RuntimeState as RusticNesRuntimeState;
use rusticnes_ui_common::events;
use rusticnes_ui_common::game_window::GameWindow;
use rusticnes_ui_common::panel::Panel;

use std::collections::VecDeque;
use std::sync::Mutex;

lazy_static! {
    static ref AUDIO_OUTPUT_BUFFER: Mutex<VecDeque<f32>> = Mutex::new(VecDeque::new());
}

struct RusticNesGameWindow {
    pub texture_handle: egui::TextureHandle,
    pub runtime_state: RusticNesRuntimeState,
    pub game_window: GameWindow,
    pub old_p1_buttons_held: u8,
}

impl RusticNesGameWindow {
    fn new(cc: &eframe::CreationContext) -> Self {
        let game_window = GameWindow::new();
        let image = egui::ColorImage::from_rgba_unmultiplied([256,240], &game_window.canvas.buffer);
        let texture_handle = cc.egui_ctx.load_texture("game_window_canvas", image, egui::TextureOptions::default());

        let mut runtime_state = RusticNesRuntimeState::new();

        let cartridge_data = std::fs::read("cartridge.nes").unwrap();
        let bucket_of_nothing: Vec<u8> = Vec::new();
        let cartridge_load_event = rusticnes_ui_common::Event::LoadCartridge(
            "cartridge.nes".to_string(), std::rc::Rc::new(cartridge_data), std::rc::Rc::new(bucket_of_nothing));
        runtime_state.handle_event(cartridge_load_event);

        Self {
            game_window: game_window,
            texture_handle: texture_handle,
            runtime_state: runtime_state,
            old_p1_buttons_held: 0,
        }
    }

    fn apply_player_input(&mut self, ctx: &egui::Context) {
        // For now, use the same hard-coded input setup from the SDL build.
        // We will eventually completely throw this out and replace it with the input mapping system
        // TODO: how does this handle the application being unfocused on various platforms?

        ctx.input(|i| {
            let mut p1_buttons_held = 0;

            if i.keys_down.contains(&egui::Key::X)          {p1_buttons_held |= 1 << 0;}
            if i.keys_down.contains(&egui::Key::Z)          {p1_buttons_held |= 1 << 1;}
            if i.keys_down.contains(&egui::Key::Backspace)  {p1_buttons_held |= 1 << 2;}
            if i.keys_down.contains(&egui::Key::Enter)      {p1_buttons_held |= 1 << 3;}
            if i.keys_down.contains(&egui::Key::ArrowUp)    {p1_buttons_held |= 1 << 4;}
            if i.keys_down.contains(&egui::Key::ArrowDown)  {p1_buttons_held |= 1 << 5;}
            if i.keys_down.contains(&egui::Key::ArrowLeft)  {p1_buttons_held |= 1 << 6;}
            if i.keys_down.contains(&egui::Key::ArrowRight) {p1_buttons_held |= 1 << 7;}

            let p1_buttons_pressed = p1_buttons_held & !self.old_p1_buttons_held;
            let p1_buttons_released = !p1_buttons_held & self.old_p1_buttons_held;

            if (p1_buttons_pressed & (1 << 0)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerPress(0, events::StandardControllerButton::A));
            }
            if (p1_buttons_pressed & (1 << 1)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerPress(0, events::StandardControllerButton::B));
            }
            if (p1_buttons_pressed & (1 << 2)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerPress(0, events::StandardControllerButton::Select));
            }
            if (p1_buttons_pressed & (1 << 3)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerPress(0, events::StandardControllerButton::Start));
            }
            if (p1_buttons_pressed & (1 << 4)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerPress(0, events::StandardControllerButton::DPadUp));
            }
            if (p1_buttons_pressed & (1 << 5)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerPress(0, events::StandardControllerButton::DPadDown));
            }
            if (p1_buttons_pressed & (1 << 6)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerPress(0, events::StandardControllerButton::DPadLeft));
            }
            if (p1_buttons_pressed & (1 << 7)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerPress(0, events::StandardControllerButton::DPadRight));
            }

            if (p1_buttons_released & (1 << 0)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerRelease(0, events::StandardControllerButton::A));
            }
            if (p1_buttons_released & (1 << 1)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerRelease(0, events::StandardControllerButton::B));
            }
            if (p1_buttons_released & (1 << 2)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerRelease(0, events::StandardControllerButton::Select));
            }
            if (p1_buttons_released & (1 << 3)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerRelease(0, events::StandardControllerButton::Start));
            }
            if (p1_buttons_released & (1 << 4)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerRelease(0, events::StandardControllerButton::DPadUp));
            }
            if (p1_buttons_released & (1 << 5)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerRelease(0, events::StandardControllerButton::DPadDown));
            }
            if (p1_buttons_released & (1 << 6)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerRelease(0, events::StandardControllerButton::DPadLeft));
            }
            if (p1_buttons_released & (1 << 7)) != 0 {
                self.runtime_state.handle_event(events::Event::StandardControllerRelease(0, events::StandardControllerButton::DPadRight));
            }


            self.old_p1_buttons_held = p1_buttons_held;
        });
    }
}

impl eframe::App for RusticNesGameWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Presumably this is called at some FPS? I guess we can find out!

        self.apply_player_input(ctx);

        // Quickly poll the length of the audio buffer
        let audio_output_buffer = AUDIO_OUTPUT_BUFFER.lock().expect("wat");
        let output_buffer_len = audio_output_buffer.len();
        drop(audio_output_buffer); // immediately free the mutex, so running the emulator doesn't starve the audio thread


        let mut samples_i16: Vec<i16> = Vec::new();
        // 2048 is arbitrary, make this configurable!
        while output_buffer_len + samples_i16.len() < 2048 {
            self.runtime_state.handle_event(events::Event::NesRunFrame);
            self.game_window.handle_event(&self.runtime_state, events::Event::RequestFrame);
            samples_i16.extend(self.runtime_state.nes.apu.consume_samples());
        }
        if samples_i16.len() > 0 {
            let image = egui::ColorImage::from_rgba_unmultiplied([256,240], &self.game_window.canvas.buffer);
            self.texture_handle.set(image, egui::TextureOptions::default());

            let samples_float: Vec<f32> = samples_i16.into_iter().map(|x| <i16 as Into<f32>>::into(x) / 32767.0).collect();
            let mut audio_output_buffer = AUDIO_OUTPUT_BUFFER.lock().expect("wat");
            audio_output_buffer.extend(samples_float);
            drop(audio_output_buffer);
        }

        egui::TopBottomPanel::top("game_window_top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        // would open file
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        // would exit application
                    }
                })
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Simple Canvas Painting");
            ui.image(egui::load::SizedTexture::from_handle(&self.texture_handle));
        });

        ctx.request_repaint();
    }
}


fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    // Setup the audio callback, which will ultimately be in charge of trying to step emulation
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");

    // TODO: eventually we want to present the supported configs to the end user, and let
    // them pick
    let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        //.with_max_sample_rate();
        .with_sample_rate(cpal::SampleRate(44100));
    println!("selected output sample rate: {:?}", supported_config);

    let mut stream_config: cpal::StreamConfig = supported_config.into();
    stream_config.buffer_size = cpal::BufferSize::Fixed(256);

    let stream = device.build_output_stream(
        &stream_config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut audio_output_buffer = AUDIO_OUTPUT_BUFFER.lock().expect("wat");
            if audio_output_buffer.len() > data.len() {
                let output_samples = audio_output_buffer.drain(0..data.len()).collect::<VecDeque<f32>>();
                for i in 0 .. data.len() {
                    data[i] = output_samples[i];
                }
            } else {
                for sample in data.iter_mut() {
                    *sample = cpal::Sample::EQUILIBRIUM;
                }
            }
        },
        move |err| {
            println!("Audio error occurred: {}", err)
        },
        None // None=blocking, Some(Duration)=timeout
    ).unwrap();

    stream.play().unwrap();
    

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([512.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RusticNES egui - Single Window", 
        options, 
        Box::new(|cc| Box::new(RusticNesGameWindow::new(cc))),
    )
}
