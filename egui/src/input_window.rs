use crate::app;

use app::ShellEvent;
use eframe::egui;
use std::sync::mpsc::{Sender};
use rustico_ui_common::events;
use rustico_ui_common::settings::SettingsState;

pub struct InputWindow {
    pub shown: bool
}

impl InputWindow {
    pub fn new() -> Self {
        return Self {
            shown: false,
        };
    }

    pub fn handle_event(&mut self, event: ShellEvent) {
        match event {
            _ => {}
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _settings: &SettingsState, _runtime_tx: &mut Sender<events::Event>) -> Vec<ShellEvent> {
        let shell_events: Vec<ShellEvent> = Vec::new();

        if self.shown == false {
            return shell_events;
        }

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("input_settings_viewport"),
            egui::ViewportBuilder::default()
                .with_title("Configure Input")
                .with_inner_size([300.0, 600.0]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports!"
                );
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Hello Input Mapper!");
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.shown = false;
                }
            }
        );

        return shell_events;
    }
}