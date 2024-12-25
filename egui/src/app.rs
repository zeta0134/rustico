use crate::worker;
use crate::game_window;
use crate::input_window;

use eframe::egui;
use rustico_ui_common::events;

use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver, TryRecvError};

#[derive(Clone, Debug)]
pub enum ShellEvent {
    ImageRendered(String, Arc<worker::RenderedImage>),
    HasSram(bool),
    SettingsUpdated(Arc<rustico_ui_common::settings::SettingsState>),
    ToggleInputWindowShown,
    RawButtonPress(String),
    RawButtonRelease(String),
    RawHotkeyPress(String),
}

fn normalized_key_name(key: &egui::Key) -> String {
    return key.name().to_string()
}

fn normalized_hotkey_name(key: &egui::Key, modifiers: &egui::Modifiers) -> String {
    let mut key_components: Vec<String> = Vec::new();
    if modifiers.mac_cmd == true {key_components.push("Cmd".to_string())}
    if modifiers.ctrl == true {key_components.push("Ctrl".to_string())}
    if modifiers.alt == true {key_components.push("Alt".to_string())}
    if modifiers.shift == true {key_components.push("Shift".to_string())}
    key_components.push(key.name().to_string());
    let key_string = key_components.join("+");
    return key_string
}

pub fn detect_key_events(input_state: &egui::InputState) -> Vec<ShellEvent> {
    let mut shell_events: Vec<ShellEvent> = Vec::new();
    for event in &input_state.events {
        match event {
            egui::Event::Key{key, physical_key: _, pressed, repeat: _, modifiers} => {
                let key_name = normalized_key_name(key);
                if *pressed == true {
                    //println!("Firing press event");
                    let hotkey_name = normalized_hotkey_name(key, modifiers);
                    shell_events.push(ShellEvent::RawButtonPress(key_name));
                    shell_events.push(ShellEvent::RawHotkeyPress(hotkey_name));
                } else {
                    shell_events.push(ShellEvent::RawButtonRelease(key_name));
                }
            },
            _ => {/* don't care */}
        }
    }
    return shell_events;
}

pub struct RusticoApp {
    pub old_p1_buttons_held: u8,

    pub runtime_tx: Sender<events::Event>,
    pub shell_rx: Receiver<ShellEvent>,

    pub settings_cache: rustico_ui_common::settings::SettingsState,
    pub game_window: game_window::GameWindow,
    pub input_window: input_window::InputWindow,
}

impl RusticoApp {
    pub fn new(cc: &eframe::CreationContext, runtime_tx: Sender<events::Event>, shell_rx: Receiver<ShellEvent>) -> Self {
        Self {
            old_p1_buttons_held: 0,

            runtime_tx: runtime_tx,
            shell_rx: shell_rx,

            settings_cache: rustico_ui_common::settings::SettingsState::new(),

            game_window: game_window::GameWindow::new(cc),
            input_window: input_window::InputWindow::new(),
        }
    }

    fn receive_worker_shell_events(&mut self) {
        loop {
            match self.shell_rx.try_recv() {
                Ok(event) => {
                    self.handle_event(event.clone());
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
        match event.clone() {
            ShellEvent::SettingsUpdated(settings_object) => {
                self.settings_cache = Arc::unwrap_or_clone(settings_object);
            },
            ShellEvent::ToggleInputWindowShown => {
                self.input_window.shown = !self.input_window.shown;
            }
            _ => {}
        }
        self.game_window.handle_event(event.clone());
        self.input_window.handle_event(event.clone(), &mut self.runtime_tx);
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
        let mut shell_events: Vec<ShellEvent> = Vec::new();

        ctx.input(|i| {
            shell_events.extend(detect_key_events(i));
        });
        self.apply_player_input(ctx);
        self.receive_worker_shell_events();

        // Always run all viewport update routines
        // (the viewports contain logic to show/hide themselves as appropriate)
        shell_events.extend(self.game_window.update(ctx, &self.settings_cache, &mut self.runtime_tx));
        shell_events.extend(self.input_window.update(ctx, &self.settings_cache));

        for event in shell_events {
            self.handle_event(event);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        println!("Application closing! Attempting to save SRAM one last time...");
        self.request_sram_save();
        let _ = self.runtime_tx.send(events::Event::CloseApplication);
    }
}