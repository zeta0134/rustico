use rusticnes_ui_common::panel::Panel;

use sdl2::pixels::Color;
use sdl2::VideoSubsystem;
use sdl2::render::WindowCanvas;

pub struct PlatformWindow {
  pub panel: Box<dyn Panel>,
  pub canvas: WindowCanvas,
}

impl<'a> PlatformWindow {
  pub fn from_panel(video_subsystem: &'a VideoSubsystem, panel: Box<dyn Panel>) -> PlatformWindow {
    let width = panel.active_canvas().width * panel.scale_factor();
    let height = panel.active_canvas().height * panel.scale_factor();
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
    }
  }

  pub fn size(&self) -> (u32, u32) {
    let px = self.panel.active_canvas().width * self.panel.scale_factor();
    let py = self.panel.active_canvas().height * self.panel.scale_factor();
    return (px, py);
  }

  pub fn needs_resize(&self) -> bool {
    let (wx, wy) = self.canvas.window().size();
    let (px, py) = self.size();
    return (wx != px) || (wy != py);
  }
}