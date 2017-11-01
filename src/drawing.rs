use image;
use image::Pixel;

use std::path::Path;

pub struct SimpleBuffer {
    pub buffer: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl SimpleBuffer {
    pub fn new(width: u32, height: u32) -> SimpleBuffer {
        return SimpleBuffer{
            width: width,
            height: height,
            buffer: vec!(0u8; (width * height * 4) as usize)
        }
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, color: &[u8]) {
        let index = ((y * self.width + x) * 4) as usize;
        self.buffer[index .. (index + 4)].clone_from_slice(color);
    }
}

pub struct Font {
    pub raw_buffer: SimpleBuffer,
    pub glyph_width: u32,
}

impl Font {
    pub fn new(bitmap_filename: &str, glyph_width: u32) -> Font {
        let img = image::open(&Path::new(bitmap_filename)).unwrap().to_rgba();
        let dimensions = img.dimensions();
        let mut raw_buffer = SimpleBuffer::new(dimensions.0, dimensions.1);
        for x in 0 .. dimensions.0 {
            for y in 0 .. dimensions.1 {
                let pixel = img[(x, y)].to_rgba();
                raw_buffer.put_pixel(x, y, &pixel.data);
            }
        }
        return Font {
            raw_buffer: raw_buffer,
            glyph_width: glyph_width,
        }
    }
}