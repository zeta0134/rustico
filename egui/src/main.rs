#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[macro_use]
extern crate lazy_static;
extern crate rustico_core;
extern crate rustico_ui_common;

mod app;
mod game_window;
mod input_window;
mod worker;

use eframe::egui;
use rustico_ui_common::events;

use std::sync::mpsc::{channel};
use std::thread;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let (runtime_tx, runtime_rx) = channel::<events::Event>();
    let (shell_tx, shell_rx) = channel::<app::ShellEvent>();

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
        "Rustico", 
        options, 
        Box::new(|cc| Box::new(app::RusticoApp::new(cc, runtime_tx, shell_rx))),
    );

    // Wait for the worker thread to exit here, so it has time to process any final
    // file operations before it terminates. (By this stage, we have already gracefully
    // requested that it shut down)
    worker_handle.join().expect("Failed to gracefully shut down worker thread. Did it crash? Data may be lost!");

    return application_exit_state;
}
