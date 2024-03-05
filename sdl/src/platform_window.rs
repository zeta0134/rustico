use rusticnes_ui_common::panel::Panel;

use sdl2::pixels::Color;
use sdl2::VideoSubsystem;
use sdl2::render::WindowCanvas;

pub struct PlatformWindow {
  pub panel: Box<dyn Panel>,
  pub canvas: WindowCanvas,
  pub texture_size_x: u32,
  pub texture_size_y: u32,
}

impl<'a> PlatformWindow {
  pub fn from_panel(video_subsystem: &'a VideoSubsystem, panel: Box<dyn Panel>) -> PlatformWindow {
    let canvas_width = panel.active_canvas().width;
    let canvas_height = panel.active_canvas().height;
    let width = canvas_width * panel.scale_factor();
    let height = canvas_height * panel.scale_factor();
    let sdl_window = video_subsystem.window(panel.title(), width, height)
      .position(490, 40)
      .opengl()
      .hidden()
      .build()
      .unwrap();
    let mut sdl_canvas = sdl_window.into_canvas().present_vsync().build().unwrap();
    sdl_canvas.set_draw_color(Color::RGB(0, 0, 0));
    sdl_canvas.clear();
    sdl_canvas.present();

    return PlatformWindow {
      panel: panel,
      canvas: sdl_canvas,
      texture_size_x: canvas_width,
      texture_size_y: canvas_height,
    }
  }

  pub fn canvas_size(&self) -> (u32, u32) {
    let cx = self.panel.active_canvas().width;
    let cy = self.panel.active_canvas().height;
    return (cx, cy);
  }

  pub fn window_size(&self) -> (u32, u32) {
    let px = self.panel.active_canvas().width * self.panel.scale_factor();
    let py = self.panel.active_canvas().height * self.panel.scale_factor();
    return (px, py);
  }

  pub fn needs_resize(&self) -> bool {
    let (wx, wy) = self.canvas.window().size();
    let (px, py) = self.window_size();
    let window_resized = (wx != px) || (wy != py);
    let (cx, cy) = self.canvas_size();
    let canvas_resized = (cx != self.texture_size_x) || (cy != self.texture_size_y);
    return window_resized || canvas_resized;
  }
}