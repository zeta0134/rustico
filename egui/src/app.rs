use crate::worker;
use crate::game_window;

use eframe::egui;
use rustico_ui_common::events;

use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver, TryRecvError};

#[derive(Clone)]
pub enum ShellEvent {
    ImageRendered(String, Arc<worker::RenderedImage>),
    HasSram(bool),
    SettingsUpdated(Arc<rustico_ui_common::settings::SettingsState>)
}

pub struct RusticoApp {
    pub old_p1_buttons_held: u8,

    pub show_memory_viewer: bool,
    pub show_event_viewer: bool,
    pub show_ppu_viewer: bool,
    pub show_piano_roll: bool,

    pub runtime_tx: Sender<events::Event>,
    pub shell_rx: Receiver<ShellEvent>,

    pub settings_cache: rustico_ui_common::settings::SettingsState,

    pub game_window: game_window::GameWindow,
}

impl RusticoApp {
    pub fn new(cc: &eframe::CreationContext, runtime_tx: Sender<events::Event>, shell_rx: Receiver<ShellEvent>) -> Self {
        Self {
            old_p1_buttons_held: 0,

            show_memory_viewer: false,
            show_event_viewer: false,
            show_ppu_viewer: false,
            show_piano_roll: false,

            runtime_tx: runtime_tx,
            shell_rx: shell_rx,

            settings_cache: rustico_ui_common::settings::SettingsState::new(),

            game_window: game_window::GameWindow::new(cc),
        }
    }

    fn process_shell_events(&mut self) {
        loop {
            match self.shell_rx.try_recv() {
                Ok(event) => {
                    self.handle_event(event.clone());
                    self.game_window.handle_event(event.clone());
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
            ShellEvent::SettingsUpdated(settings_object) => {
                self.settings_cache = Arc::unwrap_or_clone(settings_object);
            },
            _ => {}
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
                let _ = self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::A));
            }
            if (p1_buttons_pressed & (1 << 1)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::B));
            }
            if (p1_buttons_pressed & (1 << 2)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::Select));
            }
            if (p1_buttons_pressed & (1 << 3)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::Start));
            }
            if (p1_buttons_pressed & (1 << 4)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::DPadUp));
            }
            if (p1_buttons_pressed & (1 << 5)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::DPadDown));
            }
            if (p1_buttons_pressed & (1 << 6)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::DPadLeft));
            }
            if (p1_buttons_pressed & (1 << 7)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerPress(0, events::StandardControllerButton::DPadRight));
            }

            if (p1_buttons_released & (1 << 0)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::A));
            }
            if (p1_buttons_released & (1 << 1)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::B));
            }
            if (p1_buttons_released & (1 << 2)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::Select));
            }
            if (p1_buttons_released & (1 << 3)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::Start));
            }
            if (p1_buttons_released & (1 << 4)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::DPadUp));
            }
            if (p1_buttons_released & (1 << 5)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::DPadDown));
            }
            if (p1_buttons_released & (1 << 6)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::DPadLeft));
            }
            if (p1_buttons_released & (1 << 7)) != 0 {
                let _ = self.runtime_tx.send(events::Event::StandardControllerRelease(0, events::StandardControllerButton::DPadRight));
            }


            self.old_p1_buttons_held = p1_buttons_held;
        });
    }

    fn request_sram_save(&mut self) {
        self.game_window.request_sram_save(&mut self.runtime_tx);
    }
}

impl eframe::App for RusticoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Presumably this is called at some FPS? I guess we can find out!
        self.apply_player_input(ctx);
        self.process_shell_events();

        // Always run the game window
        self.game_window.update(ctx, &self.settings_cache, &mut self.runtime_tx);

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
        println!("Application closing! Attempting to save SRAM one last time...");
        self.request_sram_save();
        let _ = self.runtime_tx.send(events::Event::CloseApplication);
    }
}