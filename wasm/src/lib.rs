#[macro_use]
extern crate lazy_static;
extern crate rusticnes_core;
extern crate rusticnes_ui_common;
extern crate wasm_bindgen;

use std::sync::Mutex;
use std::rc::Rc;

use rusticnes_core::palettes::NTSC_PAL;
use rusticnes_core::apu::FilterType;
use wasm_bindgen::prelude::*;

use rusticnes_ui_common::application::RuntimeState;
use rusticnes_ui_common::settings::SettingsState;
use rusticnes_ui_common::events::Event;
use rusticnes_ui_common::apu_window::ApuWindow;
use rusticnes_ui_common::piano_roll_window::PianoRollWindow;

use rusticnes_ui_common::panel::Panel;
use rusticnes_ui_common::drawing::SimpleBuffer;

const WASM_CONFIG: &str = r###"
[piano_roll]
canvas_width = 480
canvas_height = 270
key_length = 24
key_thickness = 4
octave_count = 9
scale_factor = 1
speed_multiplier = 1
starting_octave = 0
waveform_height = 32
oscilloscope_glow_thickness = 2.5
oscilloscope_line_thickness = 0.5
divider_width = 2
draw_text_labels = false
draw_piano_strings = false
"###;

/* There is no "main" scope, so our application globals need to be actual globals.
   These will be resolved any time a JS event needs to use them; JavaScript will only
   ever process one event in a single thread, but Rust doesn't know that, so we get to
   use Mutexes for no particular reason. */
lazy_static! {
    static ref RUNTIME: Mutex<RuntimeState> = Mutex::new(RuntimeState::new());
    static ref APU_WINDOW: Mutex<ApuWindow> = Mutex::new(ApuWindow::new());
    static ref PIANO_ROLL_WINDOW: Mutex<PianoRollWindow> = Mutex::new(PianoRollWindow::new());

    /* used for blitting the game window */
    static ref CRT_OVERLAY: Mutex<SimpleBuffer> = Mutex::new(SimpleBuffer::from_raw(include_bytes!("assets/overlay.png")));
    static ref GAME_RENDER: Mutex<SimpleBuffer> = Mutex::new(SimpleBuffer::new(256, 240));
}

pub fn dispatch_event(event: Event, runtime: &mut RuntimeState) -> Vec<Event> {
  let mut responses: Vec<Event> = Vec::new();

  let mut apu_window = APU_WINDOW.lock().expect("wat");
  let mut piano_roll_window = PIANO_ROLL_WINDOW.lock().expect("wat");
  
  // windows get an immutable reference to the runtime
  responses.extend(apu_window.handle_event(&runtime, event.clone()));
  responses.extend(piano_roll_window.handle_event(&runtime, event.clone()));

  // ... but RuntimeState needs a mutable reference to itself
  responses.extend(runtime.handle_event(event.clone()));
  
  return responses;
}

pub fn resolve_events(mut events: Vec<Event>, runtime: &mut RuntimeState) {
  while events.len() > 0 {
    let event = events.remove(0);
    let responses = dispatch_event(event, runtime);
    events.extend(responses);
  }
}

#[wasm_bindgen]
pub fn wasm_init() {
  let mut runtime = RUNTIME.lock().expect("wat");

  runtime.settings = SettingsState::new();
  let settings_events = runtime.settings.apply_settings();
  resolve_events(settings_events, &mut runtime);

  runtime.settings.load_str(WASM_CONFIG);
  let settings_events = runtime.settings.apply_settings();
  resolve_events(settings_events, &mut runtime);
}

#[wasm_bindgen]
pub fn load_rom(cart_data: &[u8]) {
  let mut runtime = RUNTIME.lock().expect("wat");
  let mut events: Vec<Event> = Vec::new();
  let bucket_of_nothing: Vec<u8> = Vec::new();
  let cartridge_data = cart_data.to_vec();
  events.push(Event::LoadCartridge("cartridge".to_string(), Rc::new(cartridge_data), Rc::new(bucket_of_nothing)));
  resolve_events(events, &mut runtime);
}

#[wasm_bindgen]
pub fn run_until_vblank() {
  let mut runtime = RUNTIME.lock().expect("wat");
  while runtime.nes.ppu.current_scanline == 242 {
    let mut events: Vec<Event> = Vec::new();
    events.push(Event::NesRunScanline);
    resolve_events(events, &mut runtime);
  }
  while runtime.nes.ppu.current_scanline != 242 {
    let mut events: Vec<Event> = Vec::new();
    events.push(Event::NesRunScanline);
    resolve_events(events, &mut runtime);
  }
}

#[wasm_bindgen]
pub fn update_windows() {
  let mut runtime = RUNTIME.lock().expect("wat");
  let mut events: Vec<Event> = Vec::new();
  events.push(Event::Update);
  resolve_events(events, &mut runtime);
}

pub fn render_screen_pixels() {
  let runtime = RUNTIME.lock().expect("wat");
  let nes = &runtime.nes;

  let overlay = CRT_OVERLAY.lock().expect("wat");
  let mut render_canvas = GAME_RENDER.lock().expect("wat");
  let pixels = &mut render_canvas.buffer;

  for x in 0 .. 256 {
    for y in 0 .. 240 {
      let palette_index = ((nes.ppu.screen[y * 256 + x]) as usize) * 3;
      let pixel_offset = (y * 256 + x) * 4;
      // overlay with direct buffer reading
      let alpha = overlay.buffer[pixel_offset] as u16;
      let background_color = [3, 3, 3];
      let r = (((NTSC_PAL[palette_index + 0] as u16 * alpha) + (background_color[0] * (256 - alpha))) / 256) as u8;
      let g = (((NTSC_PAL[palette_index + 1] as u16 * alpha) + (background_color[1] * (256 - alpha))) / 256) as u8;
      let b = (((NTSC_PAL[palette_index + 2] as u16 * alpha) + (background_color[2] * (256 - alpha))) / 256) as u8;
      pixels[pixel_offset + 0] = r;
      pixels[pixel_offset + 1] = g;
      pixels[pixel_offset + 2] = b;
      pixels[((y * 256 + x) * 4) + 3] = 255;
    }
  }
}

#[wasm_bindgen]
pub fn draw_screen_pixels(pixels: &mut [u8]) {
  render_screen_pixels();
  let render_canvas = GAME_RENDER.lock().expect("wat");
  pixels.copy_from_slice(&render_canvas.buffer[0..(256*240*4)]);
}

#[wasm_bindgen]
pub fn draw_apu_window(dest: &mut [u8]) {
  let mut runtime = RUNTIME.lock().expect("wat");
  let mut apu_window = APU_WINDOW.lock().expect("wat");
  resolve_events(apu_window.handle_event(&runtime, Event::RequestFrame), &mut runtime);
  dest.copy_from_slice(&apu_window.active_canvas().buffer[0..(256*500*4)]);
}

#[wasm_bindgen]
pub fn draw_piano_roll_window(dest: &mut [u8]) {
  let mut runtime = RUNTIME.lock().expect("wat");
  let mut piano_roll_window = PIANO_ROLL_WINDOW.lock().expect("wat");
  resolve_events(piano_roll_window.handle_event(&runtime, Event::RequestFrame), &mut runtime);
  dest.copy_from_slice(&piano_roll_window.active_canvas().buffer);
}

#[wasm_bindgen]
pub fn set_p1_input(keystate: u8) {
  let mut runtime = RUNTIME.lock().expect("wat");
  let nes = &mut runtime.nes;
  nes.p1_input = keystate;
}

#[wasm_bindgen]
pub fn set_p2_input(keystate: u8) {
  let mut runtime = RUNTIME.lock().expect("wat");
  let nes = &mut runtime.nes;
  nes.p2_input = keystate;
}

#[wasm_bindgen]
pub fn set_audio_samplerate(sample_rate: u32) {
  let mut runtime = RUNTIME.lock().expect("wat");
  let nes = &mut runtime.nes;
  
  nes.apu.set_sample_rate(sample_rate as u64);
  // while we're here, set the filter to low quality
  nes.apu.set_filter(FilterType::FamiCom, false);
  // and if this happens to be an N163 ROM, tell it not
  // to use multiplexing, as this sounds *awful* when passed
  // through the LQ filtering chain
  nes.mapper.audio_multiplexing(false);
}

#[wasm_bindgen]
pub fn set_audio_buffersize(buffer_size: u32) {
  let mut runtime = RUNTIME.lock().expect("wat");
  let nes = &mut runtime.nes;
  
  nes.apu.set_buffer_size(buffer_size as usize);
}

#[wasm_bindgen]
pub fn audio_buffer_full() -> bool {
  let runtime = RUNTIME.lock().expect("wat");
  let nes = &runtime.nes;
  
  return nes.apu.buffer_full;
}

#[wasm_bindgen]
pub fn get_audio_buffer() -> Vec<i16> {
  let mut runtime = RUNTIME.lock().expect("wat");
  let nes = &mut runtime.nes;

  nes.apu.buffer_full = false;
  return nes.apu.output_buffer.to_owned();
}

#[wasm_bindgen]
pub fn consume_audio_samples() -> Vec<i16> {
  let mut runtime = RUNTIME.lock().expect("wat");
  let nes = &mut runtime.nes;

  return nes.apu.consume_samples();
}

#[wasm_bindgen]
pub fn get_sram() -> Vec<u8> {
  let runtime = RUNTIME.lock().expect("wat");
  let nes = &runtime.nes;

  return nes.sram().to_owned();
}

#[wasm_bindgen]
pub fn set_sram(sram: Vec<u8>) {
  let mut runtime = RUNTIME.lock().expect("wat");
  let nes = &mut runtime.nes;
  
  nes.set_sram(sram);
}

#[wasm_bindgen]
pub fn has_sram() -> bool {
  let runtime = RUNTIME.lock().expect("wat");
  let nes = &runtime.nes;
  
  return nes.mapper.has_sram();
}

#[wasm_bindgen]
pub fn piano_roll_window_click(mx: i32, my: i32) {
  let mut runtime = RUNTIME.lock().expect("wat");
  let mut events: Vec<Event> = Vec::new();
  let mut piano_roll_window = PIANO_ROLL_WINDOW.lock().expect("wat");
  events.extend(piano_roll_window.handle_event(&runtime, Event::MouseClick(mx, my)));
  drop(piano_roll_window);
  resolve_events(events, &mut runtime);
}