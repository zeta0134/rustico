use crate::app;

use app::ShellEvent;
use app::detect_key_events;
use eframe::egui;
use std::sync::mpsc::{Sender};
use rustico_ui_common::events;
use rustico_ui_common::settings::SettingsState;

pub struct InputWindow {
    pub shown: bool,
    pub mapping_in_progress: bool,
    pub mapping_hotkey: bool,
    pub mapping_action: String,
}

impl InputWindow {
    pub fn new() -> Self {
        return Self {
            shown: false,
            mapping_in_progress: false,
            mapping_hotkey: false,
            mapping_action: "".to_string(),
        };
    }

    pub fn handle_event(&mut self, event: ShellEvent, runtime_tx: &mut Sender<events::Event>) {
        match event {
            ShellEvent::RawButtonPress(button_name) => {
                //println!("got button press event: {:?}", button_name);
                if self.mapping_hotkey == false {
                    let _ = runtime_tx.send(events::Event::StoreStringSetting(self.mapping_action.clone(), button_name));
                    self.mapping_in_progress = false;
                }
            }
            _ => {}
        }
    }

    fn key_mapping_button(&mut self, ui: &mut egui::Ui, settings: &SettingsState, action: &str) {
        if self.mapping_in_progress == true && self.mapping_action == action {
            if ui.add(egui::Button::new("[ press any key... ]")
                .selected(true)
                .min_size([150.0, 0.0].into())
            ).clicked() {
                self.mapping_in_progress = false;
            }
        } else {
            let mapping_label = settings.get_string(action.to_string()).unwrap_or("(none)".to_string());
            if ui.add(egui::Button::new(mapping_label)
                .min_size([150.0, 0.0].into())
            ).clicked() {
                self.mapping_in_progress = true;
                self.mapping_action = action.to_string();
            }
        }
    }

    fn key_mapping_row(&mut self, ui: &mut egui::Ui, settings: &SettingsState, label: &str, action: &str) {
        ui.label(label);
        self.key_mapping_button(ui, settings, action);
        ui.end_row();
    }

    pub fn update(&mut self, ctx: &egui::Context, settings: &SettingsState) -> Vec<ShellEvent> {
        let mut shell_events: Vec<ShellEvent> = Vec::new();

        if self.shown == false {
            return shell_events;
        }

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("input_settings_viewport"),
            egui::ViewportBuilder::default()
                .with_title("Configure Input")
                .with_inner_size([400.0, 600.0]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports!"
                );
                ctx.input(|i| {
                    shell_events.extend(detect_key_events(i));
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.shown = false;
                    self.mapping_in_progress = false; // if we were mapping something, cancel that out
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        egui::CollapsingHeader::new("Standard Controller P1")
                            .default_open(true)
                            .show(ui, |ui| {
                                egui::Grid::new("p1_inputs")
                                    .min_col_width(150.0)
                                    .num_columns(2)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        self.key_mapping_row(ui, settings, "D-Pad: Up",     "input.standard_controller_p1.up");
                                        self.key_mapping_row(ui, settings, "D-Pad: Down",   "input.standard_controller_p1.down");
                                        self.key_mapping_row(ui, settings, "D-Pad: Left",   "input.standard_controller_p1.left");
                                        self.key_mapping_row(ui, settings, "D-Pad: Right",  "input.standard_controller_p1.right");
                                        self.key_mapping_row(ui, settings, "Button: A",      "input.standard_controller_p1.a");
                                        self.key_mapping_row(ui, settings, "Button: B",      "input.standard_controller_p1.b");
                                        self.key_mapping_row(ui, settings, "Button: Start",  "input.standard_controller_p1.start");
                                        self.key_mapping_row(ui, settings, "Button: Select", "input.standard_controller_p1.select");
                                    });
                        });
                        egui::CollapsingHeader::new("Standard Controller P2")
                            .default_open(true)
                            .show(ui, |ui| {
                                egui::Grid::new("p2_inputs")
                                    .min_col_width(150.0)
                                    .num_columns(2)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        self.key_mapping_row(ui, settings, "D-Pad: Up",     "input.standard_controller_p2.up");
                                        self.key_mapping_row(ui, settings, "D-Pad: Down",   "input.standard_controller_p2.down");
                                        self.key_mapping_row(ui, settings, "D-Pad: Left",   "input.standard_controller_p2.left");
                                        self.key_mapping_row(ui, settings, "D-Pad: Right",  "input.standard_controller_p2.right");
                                        self.key_mapping_row(ui, settings, "Button: A",      "input.standard_controller_p2.a");
                                        self.key_mapping_row(ui, settings, "Button: B",      "input.standard_controller_p2.b");
                                        self.key_mapping_row(ui, settings, "Button: Start",  "input.standard_controller_p2.start");
                                        self.key_mapping_row(ui, settings, "Button: Select", "input.standard_controller_p2.select");
                                    });
                        });
                        egui::CollapsingHeader::new("Hotkeys")
                            .default_open(true)
                            .show(ui, |ui| {
                                egui::Grid::new("hotkeys")
                                    .min_col_width(150.0)
                                    .num_columns(2)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        self.key_mapping_row(ui, settings, "Toggle Overscan",      "input.hotkeys.toggle_overscan");
                                        self.key_mapping_row(ui, settings, "Toggle NTSC Filter",   "input.hotkeys.toggle_ntsc");
                                        self.key_mapping_row(ui, settings, "Increase Video Scale", "input.hotkeys.increase_game_scale");
                                        self.key_mapping_row(ui, settings, "Decrease Video Scale", "input.hotkeys.decrease_game_scale");
                                        self.key_mapping_row(ui, settings, "Open Cartridge",       "input.hotkeys.open_cartridge");
                                        self.key_mapping_row(ui, settings, "Exit Application",     "input.hotkeys.exit_application");
                                    });
                        });
                    });
                    
                });
            }
        );

        return shell_events;
    }
}