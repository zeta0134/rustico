use image;
use image::Pixel;
use image::RgbaImage;

#[derive(Clone)]
pub struct SimpleBuffer {
    pub buffer: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

fn blend_component(a: u8, b: u8, alpha: u8) -> u8 {
    return (
        (a as u16 * (255 - alpha as u16) / 255) + 
        (b as u16 * (alpha as u16) / 255)
    ) as u8;
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

    pub fn blend_pixel(&mut self, x: u32, y: u32, color: &[u8]) {
        let index = ((y * self.width + x) * 4) as usize;
        let original = self.get_pixel(x, y);
        let alpha = color[3];
        let r = blend_component(original[0], color[0], alpha);
        let g = blend_component(original[1], color[1], alpha);
        let b = blend_component(original[2], color[2], alpha);
        self.buffer[index .. (index + 4)].clone_from_slice(&[r, g, b, 255]);
    }    

    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let index = ((y * self.width + x) * 4) as usize;
        return [
            self.buffer[index],
            self.buffer[index + 1],
            self.buffer[index + 2],
            self.buffer[index + 3]];
    }
}

pub struct Font {
    pub glyph_width: u32,
    pub glyphs: Vec<SimpleBuffer>,
}

impl Font {
    pub fn from_image(img: RgbaImage, glyph_width: u32) -> Font {
        let (img_width, img_height) = img.dimensions();

        // First, read everything into a raw buffer
        let mut raw_buffer = SimpleBuffer::new(img_width, img_height);
        for x in 0 .. img_width {
            for y in 0 .. img_height {
                let pixel = img[(x, y)].to_rgba();
                raw_buffer.put_pixel(x, y, &pixel.data);
            }
        }

        // Now, run through the newly read raw buffer, and convert each individual character into its
        // own glyph:
        let mut glyphs = vec!(SimpleBuffer::new(glyph_width, img_height); 128 - 32);
        for i in 0 .. (128 - 32) {
            for y in 0 .. img_height {
                for x in 0 .. glyph_width {
                    glyphs[i].put_pixel(x, y, &raw_buffer.get_pixel((i as u32) * glyph_width + (x as u32), y as u32));
                }
            }
        }

        return Font {
            glyph_width: glyph_width,
            glyphs: glyphs,
        }
    }
    pub fn from_raw(bitmap_data: &[u8], glyph_width: u32) -> Font {
        let img = image::load_from_memory(bitmap_data).unwrap().to_rgba();
        return Font::from_image(img, glyph_width);
    }
}

pub fn blit(destination: &mut SimpleBuffer, source: &SimpleBuffer, dx: u32, dy: u32, color: &[u8]) {
    for x in 0 .. source.width {
        for y in 0 .. source.height {
            let mut source_color = source.get_pixel(x, y);
            let destination_color = destination.get_pixel(dx + x, dy + y);
            // Multiply by target color
            for i in 0 .. 4 {
                source_color[i] = ((source_color[i] as u16 * color[i] as u16) / 255) as u8;
            }
            // Blend to apply alpha transparency
            let source_alpha = source_color[3] as u16;
            let destination_alpha = 255 - source_alpha;
            let final_color = [
                ((destination_color[0] as u16 * destination_alpha + source_color[0] as u16 * source_alpha) / 255) as u8,
                ((destination_color[1] as u16 * destination_alpha + source_color[1] as u16 * source_alpha) / 255) as u8,
                ((destination_color[2] as u16 * destination_alpha + source_color[2] as u16 * source_alpha) / 255) as u8,
                255];
            destination.put_pixel(dx + x, dy + y, &final_color);
        }
    }
}

pub fn char(destination: &mut SimpleBuffer, font: &Font, x: u32, y: u32, c: char, color: &[u8]) {
    if c.is_ascii() {
        let ascii_code_point = c as u32;
        if ascii_code_point >= 32 && ascii_code_point < 127 {
            blit(destination, &font.glyphs[(ascii_code_point - 32) as usize], x, y, color);
        }
    }
}

pub fn text(destination: &mut SimpleBuffer, font: &Font, x: u32, y: u32, s: &str, color: &[u8]) {
    for i in 0 .. s.len() {
        char(destination, font, x + ((i as u32) * font.glyph_width), y, s.chars().nth(i).unwrap(), color);
    }
}

pub fn hex(destination: &mut SimpleBuffer, font: &Font, x: u32, y: u32, value: u32, nybbles: u32, color: &[u8]) {
    let char_map = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'];
    for i in 0 .. nybbles {
        let c = char_map[((value & (0xF << (i * 4))) >> (i * 4)) as usize];
        char(destination, font, x + (nybbles - 1 - (i as u32)) * font.glyph_width, y, c, color);
    }
}

pub fn rect(destination: &mut SimpleBuffer, x: u32, y: u32, width: u32, height: u32, color: &[u8]) {
    for dx in x .. (x + width) {
        for dy in y .. (y + height) {
            destination.put_pixel(dx, dy, &color);
        }
    }
}