use crate::app;
use crate::worker;

use app::ShellEvent;

use eframe::egui;
use rfd::FileDialog;
use rustico_ui_common::events;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Sender};

use rustico_ui_common::settings::SettingsState;

pub struct GameWindow {
    pub texture_handle: egui::TextureHandle,
    pub last_rendered_frames: VecDeque<Arc<worker::RenderedImage>>,
    pub game_window_scale: usize,
    pub sram_path: PathBuf,
    pub has_sram: bool,
}

impl GameWindow {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let blank_canvas = vec![0u8; 256*240*4];
        let image = egui::ColorImage::from_rgba_unmultiplied([256,240], &blank_canvas);
        let texture_handle = cc.egui_ctx.load_texture("game_window_canvas", image, egui::TextureOptions::default());

        return GameWindow {
            texture_handle: texture_handle,
            last_rendered_frames: VecDeque::new(),
            game_window_scale: 2,
            sram_path: PathBuf::new(),
            has_sram: false,
        };
    }

    pub fn handle_event(&mut self, event: ShellEvent) {
        match event {
            ShellEvent::HasSram(has_sram) => {
                self.has_sram = has_sram;
            },
            ShellEvent::ImageRendered(id, canvas) => {
                if id == "game_window" {
                    self.last_rendered_frames.push_back(canvas);
                    if self.last_rendered_frames.len() > 2 {
                        _ = self.last_rendered_frames.pop_front();
                    }
                }
            },
            _ => {}
        }
    }

    fn process_rendered_frames(&mut self) {
        match self.last_rendered_frames.pop_front() {
            Some(canvas) => {
                let image = egui::ColorImage::from_rgba_unmultiplied([canvas.width, canvas.height], &canvas.rgba_buffer);
                let texture_options = egui::TextureOptions{
                    magnification: egui::TextureFilter::Nearest,
                    minification: egui::TextureFilter::Nearest,
                    ..egui::TextureOptions::default()
                };
                self.texture_handle.set(image, texture_options);
                self.game_window_scale = canvas.scale;
            },
            None => {}
        }
    }

    pub fn request_sram_save(&mut self, runtime_tx: &mut Sender<events::Event>) {
        let _ = runtime_tx.send(events::Event::RequestSramSave(self.sram_path.clone().to_string_lossy().into_owned()));
    }

    fn open_cartridge_dialog(&mut self, runtime_tx: &mut Sender<events::Event>) {
        let files = FileDialog::new()
            .add_filter("compatible files", &["nes", "nsf"])
            .pick_file();
        match files {
            Some(file_path) => {
                self.open_cartridge(file_path, runtime_tx);
            },
            None => {
                println!("User canceled the dialog.");
            }
        }
    }

    fn open_cartridge(&mut self, cartridge_path: PathBuf, runtime_tx: &mut Sender<events::Event>) {
        // Before we open a new cartridge, save the SRAM for the old one
        self.request_sram_save(runtime_tx);

        self.sram_path = cartridge_path.with_extension("sav");
        let cartridge_path_as_str = cartridge_path.clone().to_string_lossy().into_owned();
        let cartridge_load_event = match std::fs::read(cartridge_path) {
            Ok(cartridge_data) => {
                match std::fs::read(&self.sram_path.to_str().unwrap()) {
                    Ok(sram_data) => {
                        rustico_ui_common::Event::LoadCartridge(cartridge_path_as_str, Arc::new(cartridge_data), Arc::new(sram_data))
                    },
                    Err(reason) => {
                        println!("Failed to load SRAM: {}", reason);
                        println!("Continuing anyway.");
                        let bucket_of_nothing: Vec<u8> = Vec::new();
                        rustico_ui_common::Event::LoadCartridge(cartridge_path_as_str, Arc::new(cartridge_data), Arc::new(bucket_of_nothing))
                    }
                }
            },
            Err(reason) => {
                println!("{}", reason);
                rustico_ui_common::Event::LoadFailed(reason.to_string())
            }
        };
        let _ = runtime_tx.send(cartridge_load_event);
    }

    pub fn update(&mut self, ctx: &egui::Context, settings: &SettingsState, runtime_tx: &mut Sender<events::Event>) {
        self.process_rendered_frames();

        egui::TopBottomPanel::top("game_window_top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        self.open_cartridge_dialog(runtime_tx);
                        ui.close_menu();
                    }
                    if ui.add_enabled(self.has_sram, egui::Button::new("Save SRAM")).clicked() {
                        self.request_sram_save(runtime_tx);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close_menu();
                    }
                });
                ui.menu_button("Settings", |ui| {
                    ui.menu_button("Video", |ui| {
                        let mut overscan_checked = settings.get_boolean("video.simulate_overscan".into()).unwrap_or(false);
                        if ui.checkbox(&mut overscan_checked, "Hide Overscan").clicked() {
                            let _ = runtime_tx.send(events::Event::ToggleBooleanSetting("video.simulate_overscan".into()));
                            ui.close_menu();
                        }
                        let mut ntsc_checked = settings.get_boolean("video.ntsc_filter".into()).unwrap_or(false);
                        if ui.checkbox(&mut ntsc_checked, "NTSC Filter").clicked() {
                            let _ = runtime_tx.send(events::Event::ToggleBooleanSetting("video.ntsc_filter".into()));
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.radio(settings.get_integer("video.scale_factor".into()).unwrap_or(0) == 1, "1x scale").clicked() {
                            let _ = runtime_tx.send(events::Event::StoreIntegerSetting("video.scale_factor".into(), 1));
                            ui.close_menu();
                        }
                        if ui.radio(settings.get_integer("video.scale_factor".into()).unwrap_or(0) == 2, "2x scale").clicked() {
                            let _ = runtime_tx.send(events::Event::StoreIntegerSetting("video.scale_factor".into(), 2));
                            ui.close_menu();
                        }
                        if ui.radio(settings.get_integer("video.scale_factor".into()).unwrap_or(0) == 3, "3x scale").clicked() {
                            let _ = runtime_tx.send(events::Event::StoreIntegerSetting("video.scale_factor".into(), 3));
                            ui.close_menu();
                        }
                        if ui.radio(settings.get_integer("video.scale_factor".into()).unwrap_or(0) == 4, "4x scale").clicked() {
                            let _ = runtime_tx.send(events::Event::StoreIntegerSetting("video.scale_factor".into(), 4));
                            ui.close_menu();
                        }
                        if ui.radio(settings.get_integer("video.scale_factor".into()).unwrap_or(0) == 5, "5x scale").clicked() {
                            let _ = runtime_tx.send(events::Event::StoreIntegerSetting("video.scale_factor".into(), 5));
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
                        //self.show_memory_viewer = !self.show_memory_viewer;
                        ui.close_menu();
                    }
                    if ui.button("Events").clicked() {
                        //self.show_event_viewer = !self.show_event_viewer;
                        ui.close_menu();
                    }
                    if ui.button("PPU").clicked() {
                        //self.show_ppu_viewer = !self.show_ppu_viewer;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Piano Roll").clicked() {
                        //self.show_piano_roll = !self.show_piano_roll;
                        ui.close_menu();
                    }
                });
            });
        });

        let game_window_width = (self.texture_handle.size()[0] * self.game_window_scale) as f32;
        let game_window_height = (self.texture_handle.size()[1] * self.game_window_scale) as f32;
        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            ui.add(
                egui::Image::new(egui::load::SizedTexture::from_handle(&self.texture_handle))
                    .fit_to_exact_size([
                        game_window_width,
                        game_window_height
                    ].into())
            );
        });

        let menubar_height = ctx.style().spacing.interact_size[1];
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([
            game_window_width, 
            game_window_height + menubar_height].into()));
        ctx.request_repaint();
    }
}

