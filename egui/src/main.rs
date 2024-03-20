#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

extern crate rusticnes_core;
extern crate rusticnes_ui_common;

use rusticnes_ui_common::application::RuntimeState as RusticNesRuntimeState;
use rusticnes_ui_common::events;
use rusticnes_ui_common::game_window::GameWindow;
use rusticnes_ui_common::panel::Panel;

struct RusticNesGameWindow {
    pub texture_handle: egui::TextureHandle,    
    pub runtime_state: RusticNesRuntimeState,
    pub game_window: GameWindow
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
        }   
    }
}

impl eframe::App for RusticNesGameWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Presumably this is called at some FPS? I guess we can find out!

        // For now, just run a frame and then draw it.
        // TODO: once we have working audio, sync to that properly. This
        // will run at completely the wrong speed if the host FPS isn't 60 Hz,
        // or we lag, or don't vsync, etc etc
        self.runtime_state.handle_event(events::Event::NesRunFrame);
        self.game_window.handle_event(&self.runtime_state, events::Event::RequestFrame);
        let image = egui::ColorImage::from_rgba_unmultiplied([256,240], &self.game_window.canvas.buffer);
        self.texture_handle.set(image, egui::TextureOptions::default());

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
