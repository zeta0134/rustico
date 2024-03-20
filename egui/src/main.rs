#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

fn silly_paint(canvas: &mut [u8], width: usize, height: usize, frame_counter: u8) {
    for x in 0 .. width {
        for y in 0 .. height {
            canvas[y*256*4 + x*4 + 0] = (x % 256) as u8;
            canvas[y*256*4 + x*4 + 1] = (y % 256) as u8;
            canvas[y*256*4 + x*4 + 2] = x as u8 ^ y as u8 ^ frame_counter;
            canvas[y*256*4 + x*4 + 3] = 255;
        }
    }
}

struct RusticNesGameWindow {
    pub canvas: [u8; 256*240*4],
    pub texture_handle: egui::TextureHandle,
    pub frame_counter: u8,
}

impl RusticNesGameWindow {
    fn new(cc: &eframe::CreationContext) -> Self {
        let mut game_canvas = [0u8; 256*240*4];
        silly_paint(&mut game_canvas, 256, 240, 0);
        let image = egui::ColorImage::from_rgba_unmultiplied([256,240], &game_canvas);
        let texture_handle = cc.egui_ctx.load_texture("game_window_canvas", image, egui::TextureOptions::default());

        Self {
            canvas: game_canvas,
            texture_handle: texture_handle,
            frame_counter: 0,
        }   
    }
}

impl eframe::App for RusticNesGameWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        silly_paint(&mut self.canvas, 256, 240, self.frame_counter);
        let image = egui::ColorImage::from_rgba_unmultiplied([256,240], &self.canvas);
        self.texture_handle.set(image, egui::TextureOptions::default());
        self.frame_counter = self.frame_counter.wrapping_add(1);
        ctx.request_repaint();

        // Presumably this is called at some FPS? I guess we can find out!
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
