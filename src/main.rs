extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

use std::time::Duration;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let keyboard = sdl_context.keyboard();

    let game_window = video_subsystem.window("Game Window", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let debug_window = video_subsystem.window("Debug Window", 400, 300)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut game_canvas = game_window.into_canvas().build().unwrap();
    game_canvas.set_draw_color(Color::RGB(192, 96, 96));
    game_canvas.clear();
    game_canvas.present();

    let mut debug_canvas = debug_window.into_canvas().build().unwrap();
    debug_canvas.set_draw_color(Color::RGB(96, 96, 192));
    debug_canvas.clear();
    debug_canvas.present();

    let mut game_screen_texture_creator = game_canvas.texture_creator();
    let mut game_screen_texture = game_screen_texture_creator.create_texture(PixelFormatEnum::RGBA8888, TextureAccess::Streaming, 800, 600).unwrap();

    let mut game_screen_buffer = [0u8; 800 * 600 * 4];

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut frame_counter = 0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { .. } => {
                    if sdl_context.keyboard().focused_window_id().is_some() {
                        let focused_window_id = sdl_context.keyboard().focused_window_id().unwrap();
                        if game_canvas.window().id() == focused_window_id {
                            game_canvas.set_draw_color(Color::RGB(192, 96, 192));
                        }
                        if debug_canvas.window().id() == focused_window_id {
                            debug_canvas.set_draw_color(Color::RGB(192, 96, 192));
                        }
                    }
                }
                _ => {}
            }
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        // The rest of the game loop goes here...

        // Wheeeee....
        for x in 0 .. 800 {
            for y in 0 .. 600 {
                game_screen_buffer[((y * 800 + x) * 4) + 3] = x as u8;
                game_screen_buffer[((y * 800 + x) * 4) + 2] = y as u8;
                game_screen_buffer[((y * 800 + x) * 4) + 1] = (x ^ y ^ frame_counter) as u8;
                game_screen_buffer[((y * 800 + x) * 4) + 0] = 255;
            }
        }
        game_screen_texture.update(None, &game_screen_buffer, 800 * 4);

        game_canvas.set_draw_color(Color::RGB(0, 0, 0));
        game_canvas.clear();
        game_canvas.set_draw_color(Color::RGB(255, 255, 255));
        game_canvas.copy(&game_screen_texture, None, None);
        game_canvas.present();

        debug_canvas.clear();
        debug_canvas.present();
        frame_counter += 1;
    }
}