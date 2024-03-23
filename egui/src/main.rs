#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[macro_use]
extern crate lazy_static;
extern crate rusticnes_core;
extern crate rusticnes_ui_common;

mod worker;

use eframe::egui;
use rfd::FileDialog;
use rusticnes_ui_common::events;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::thread;

#[derive(Clone)]
pub enum ShellEvent {
    ImageRendered(String, Arc<worker::RenderedImage>)
}

struct RusticNesGameWindow {
    pub texture_handle: egui::TextureHandle,    
    pub old_p1_buttons_held: u8,
    pub sram_path: PathBuf,

    pub show_memory_viewer: bool,
    pub show_event_viewer: bool,
    pub show_ppu_viewer: bool,
    pub show_piano_roll: bool,

    pub runtime_tx: Sender<events::Event>,
    pub shell_rx: Receiver<ShellEvent>,

    pub last_rendered_frames: HashMap<String, VecDeque<Arc<worker::RenderedImage>>>,
}

impl RusticNesGameWindow {
    fn new(cc: &eframe::CreationContext, runtime_tx: Sender<events::Event>, shell_rx: Receiver<ShellEvent>) -> Self {
        let blank_canvas = vec![0u8; 256*240*4];
        let image = egui::ColorImage::from_rgba_unmultiplied([256,240], &blank_canvas);
        let texture_handle = cc.egui_ctx.load_texture("game_window_canvas", image, egui::TextureOptions::default());

        let mut last_rendered_frames = HashMap::new();
        last_rendered_frames.insert("game_window".to_string(), VecDeque::new());

        Self {
            texture_handle: texture_handle,
            old_p1_buttons_held: 0,
            sram_path: PathBuf::new(),

            show_memory_viewer: false,
            show_event_viewer: false,
            show_ppu_viewer: false,
            show_piano_roll: false,

            runtime_tx: runtime_tx,
            shell_rx: shell_rx,

            last_rendered_frames: last_rendered_frames,
        }
    }

    fn process_shell_events(&mut self) {
        loop {
            match self.shell_rx.try_recv() {
                Ok(event) => {
                    self.handle_event(event);
                },
                Err(error) => {
                    match error {
                        TryRecvError::Empty => {
                            // all done!
                            return
                        },
                        TryRecvError::Disconnected => {
                            // ... wat? WHO WROTE THIS PROGRAM? HOW DID THIS HAPPEN!?
                            panic!("shell_tx disconnected!!!1");
                        }
                    }
                }
            }
        }
    }

    pub fn handle_event(&mut self, event: ShellEvent) {
        // For now, I'm not going to allow shell events to fire off more shell events.
        // They'll mostly be coming from the worker thread as one-shot things
        match event {
            ShellEvent::ImageRendered(id, canvas) => {
                match self.last_rendered_frames.get_mut(id.as_str()) {
                    Some(frame_buffer) => {
                        frame_buffer.push_back(canvas);
                        if frame_buffer.len() > 2 {
                            _ = frame_buffer.pop_front();
                        }
                    },
                    None => {
                        println!("Received a rendered image named {} but I don't know how to draw that!", id);
                    }
                }
            }
            //_ => {}
        }
    }

    pub fn process_rendered_frames(&mut self) {
        match self.last_rendered_frames.get_mut("game_window") {
            Some(game_window_frame_buffer) => {
                match game_window_frame_buffer.pop_front() {
                    Some(canvas) => {
                        let image = egui::ColorImage::from_rgba_unmultiplied([canvas.width, canvas.height], &canvas.rgba_buffer);
                        let texture_options = egui::TextureOptions{
                            magnification: egui::TextureFilter::Nearest,
                            minification: egui::TextureFilter::Nearest,
                            ..egui::TextureOptions::default()
                        };
                        self.texture_handle.set(image, texture_options);
                    },
                    None => {}
                }
            },
            None => {
                panic!("Where did our game window frame buffer go!?");
            }
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
                self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::A));
            }
            if (p1_buttons_pressed & (1 << 1)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::B));
            }
            if (p1_buttons_pressed & (1 << 2)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::Select));
            }
            if (p1_buttons_pressed & (1 << 3)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::Start));
            }
            if (p1_buttons_pressed & (1 << 4)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::DPadUp));
            }
            if (p1_buttons_pressed & (1 << 5)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::DPadDown));
            }
            if (p1_buttons_pressed & (1 << 6)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::DPadLeft));
            }
            if (p1_buttons_pressed & (1 << 7)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::DPadRight));
            }

            if (p1_buttons_released & (1 << 0)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::A));
            }
            if (p1_buttons_released & (1 << 1)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::B));
            }
            if (p1_buttons_released & (1 << 2)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::Select));
            }
            if (p1_buttons_released & (1 << 3)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::Start));
            }
            if (p1_buttons_released & (1 << 4)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::DPadUp));
            }
            if (p1_buttons_released & (1 << 5)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::DPadDown));
            }
            if (p1_buttons_released & (1 << 6)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::DPadLeft));
            }
            if (p1_buttons_released & (1 << 7)) != 0 {
                self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::DPadRight));
            }


            self.old_p1_buttons_held = p1_buttons_held;
        });
    }

    fn open_cartridge_dialog(&mut self) {
        let files = FileDialog::new()
            .add_filter("nes", &["nes"])
            .add_filter("nsf", &["nsf"])
            .pick_file();
        match files {
            Some(file_path) => {
                self.open_cartridge(file_path);
            },
            None => {
                println!("User canceled the dialog.");
            }
        }
    }

    fn open_cartridge(&mut self, cartridge_path: PathBuf) {
        self.sram_path = cartridge_path.with_extension("sav");
        let cartridge_path_as_str = cartridge_path.clone().to_string_lossy().into_owned();
        let cartridge_load_event = match std::fs::read(cartridge_path) {
            Ok(cartridge_data) => {
                match std::fs::read(&self.sram_path.to_str().unwrap()) {
                    Ok(sram_data) => {
                        rusticnes_ui_common::Event::LoadCartridge(cartridge_path_as_str, Arc::new(cartridge_data), Arc::new(sram_data))
                    },
                    Err(reason) => {
                        println!("Failed to load SRAM: {}", reason);
                        println!("Continuing anyway.");
                        let bucket_of_nothing: Vec<u8> = Vec::new();
                        rusticnes_ui_common::Event::LoadCartridge(cartridge_path_as_str, Arc::new(cartridge_data), Arc::new(bucket_of_nothing))
                    }
                }
            },
            Err(reason) => {
                println!("{}", reason);
                rusticnes_ui_common::Event::LoadFailed(reason.to_string())
            }
        };
        self.runtime_tx.send(cartridge_load_event);
    }

    fn save_sram(&mut self) {
        // TODO: figure out how we're actually going to do this part!

        //let sram_path_as_str = self.sram_path.clone().to_string_lossy().into_owned();
        //if self.runtime_state.nes.mapper.has_sram() {
        //    let sram_contents = self.runtime_state.nes.sram();
        //    let file = File::create(self.sram_path.clone());
        //        match file {
        //            Err(why) => {
        //                println!("Couldn't open {}: {}", sram_path_as_str, why.to_string());
        //            },
        //            Ok(mut file) => {
        //                let _ = file.write_all(&sram_contents);
        //                println!("Wrote sram data to: {}", sram_path_as_str);
        //            },
        //        };
        //} else {
        //    println!("Cartridge has no SRAM! Nothing to do.");
        //}
    }
}

impl eframe::App for RusticNesGameWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Presumably this is called at some FPS? I guess we can find out!
        self.apply_player_input(ctx);
        self.process_shell_events();
        self.process_rendered_frames();

        egui::TopBottomPanel::top("game_window_top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        self.open_cartridge_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Save SRAM").clicked() {
                        self.save_sram();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close_menu();
                    }
                });
                ui.menu_button("Settings", |ui| {
                    ui.menu_button("Video Size", |ui| {
                        ui.add_enabled(false, egui::Button::new("1x"));
                        if ui.button("2x").clicked() {
                            ui.close_menu();
                        }
                        if ui.button("3x").clicked() {
                            ui.close_menu();
                        }
                        if ui.button("4x").clicked() {
                            ui.close_menu();
                        }
                        if ui.button("5x").clicked() {
                            ui.close_menu();
                        }
                    });
                    ui.separator();
                    if ui.button("Preferences").clicked() {
                        ui.close_menu();
                    }
                });
                ui.menu_button("Tools", |ui| {
                    if ui.button("Memory").clicked() {
                        self.show_memory_viewer = !self.show_memory_viewer;
                        ui.close_menu();
                    }
                    if ui.button("Events").clicked() {
                        self.show_event_viewer = !self.show_event_viewer;
                        ui.close_menu();
                    }
                    if ui.button("PPU").clicked() {
                        self.show_ppu_viewer = !self.show_ppu_viewer;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Piano Roll").clicked() {
                        self.show_piano_roll = !self.show_piano_roll;
                        ui.close_menu();
                    }
                });
            });
        });

        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            ui.add(
                egui::Image::new(egui::load::SizedTexture::from_handle(&self.texture_handle))
                    .fit_to_exact_size([512.0, 480.0].into())
            );
        });

        let menubar_height = ctx.style().spacing.interact_size[1];
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([512.0, 480.0 + menubar_height].into()));
        ctx.request_repaint();

        // TODO: break these out into separate files, the UI definitions are going to get very tall
        if self.show_memory_viewer {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("memory_viewer_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("Memory Viewer")
                    .with_inner_size([300.0, 200.0]),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "This egui backend doesn't support multiple viewports!"
                    );
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.label("Hello Memory Viewer!");
                    });
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_memory_viewer = false;
                    }
                }
            );
        }

        if self.show_event_viewer {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("event_viewer_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("Event Viewer")
                    .with_inner_size([300.0, 200.0]),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "This egui backend doesn't support multiple viewports!"
                    );
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.label("Hello Event Viewer!");
                    });
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_event_viewer = false;
                    }
                }
            );
        }

        if self.show_ppu_viewer {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("ppu_viewer_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("PPU Viewer")
                    .with_inner_size([300.0, 200.0]),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "This egui backend doesn't support multiple viewports!"
                    );
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.label("Hello PPU Viewer!");
                    });
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_ppu_viewer = false;
                    }
                }
            );
        }

        if self.show_piano_roll {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("piano_roll_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("Piano Roll")
                    .with_inner_size([300.0, 200.0]),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "This egui backend doesn't support multiple viewports!"
                    );
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.label("Hello Piano Roll!");
                    });
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_piano_roll = false;
                    }
                }
            );
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        println!("Application closing, attempting to save SRAM one last time...");
        self.save_sram();
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let (runtime_tx, runtime_rx) = channel::<events::Event>();
    let (shell_tx, shell_rx) = channel::<ShellEvent>();

    let worker_handle = thread::spawn(|| {
        worker::worker_main(runtime_rx, shell_tx);
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            //.with_inner_size([512.0, 480.0]),
            .with_resizable(false)
            .with_inner_size([512.0, 480.0]),
        ..Default::default()
    };

    let application_exit_state = eframe::run_native(
        "RusticNES egui - Single Window", 
        options, 
        Box::new(|cc| Box::new(RusticNesGameWindow::new(cc, runtime_tx, shell_rx))),
    );

    return application_exit_state;
}
