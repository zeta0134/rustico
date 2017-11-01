pub struct SimpleBuffer {
    pub buffer: Vec<u8>,
    pub width: u32
}

impl SimpleBuffer {
    pub fn new(width: u32, height: u32) -> SimpleBuffer {
        return SimpleBuffer{
            width: width,
            buffer: vec!(0u8; (width * height * 4) as usize)
        }
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, color: &[u8]) {
        let index = ((y * self.width + x) * 4) as usize;
        self.buffer[index .. (index + 4)].clone_from_slice(color);
    }
}