// A very simple Mapper with no esoteric features or bank switching.
// Reference capabilities: https://wiki.nesdev.com/w/index.php/NROM

use cartridge::NesHeader;
use mmc::mapper::*;
use mmc::mirroring;

pub struct Nrom {
    pub prg_rom: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mirroring: Mirroring,
    pub has_chr_ram: bool,
    pub vram: Vec<u8>,
}

impl Nrom {
    pub fn new(header: NesHeader, chr: &[u8], prg: &[u8]) -> Nrom {
        let chr_rom = match header.has_chr_ram {
            true => vec![0u8; 8 * 1024],
            false => chr.to_vec()
        };

        return Nrom {
            prg_rom: prg.to_vec(),
            prg_ram: vec![0u8; 8 * 1024],
            chr_rom: chr_rom,
            mirroring: header.mirroring,
            has_chr_ram: header.has_chr_ram,
            vram: vec![0u8; 0x1000],
        }
    }
}

impl Mapper for Nrom {
        fn print_debug_status(&self) {
        println!("======= NROM =======");
        println!("Mirroring Mode: {}", mirroring_mode_name(self.mirroring));
        println!("====================");
    }

    fn mirroring(&self) -> Mirroring {
        return self.mirroring;
    }
    
    fn read_cpu(&mut self, address: u16) -> Option<u8> {
        match address {
            0x6000 ... 0x7FFF => {
                let prg_ram_len = self.prg_ram.len();
                if prg_ram_len > 0 {
                    return Some(self.prg_ram[((address - 0x6000) % (prg_ram_len as u16)) as usize]);
                } else {
                    return None;
                }
            },
            0x8000 ... 0xFFFF => {
                let prg_rom_len = self.prg_rom.len();
                return Some(self.prg_rom[(address % (prg_rom_len as u16)) as usize]);
            },
            _ => return None
        }
    }

    fn write_cpu(&mut self, address: u16, data: u8) {
        match address {
            0x6000 ... 0x7FFF => {
                let prg_ram_len = self.prg_ram.len();
                if prg_ram_len > 0 {
                    self.prg_ram[((address - 0x6000) % (prg_ram_len as u16)) as usize] = data;
                }
            },
            _ => {}
        }
    }

    fn read_ppu(&mut self, address: u16) -> Option<u8> {
        match address {
            0x0000 ... 0x1FFF => return Some(self.chr_rom[address as usize]),
            0x2000 ... 0x3FFF => return match self.mirroring {
                Mirroring::Horizontal => Some(self.vram[mirroring::horizontal_mirroring(address) as usize]),
                Mirroring::Vertical   => Some(self.vram[mirroring::vertical_mirroring(address) as usize]),
                _ => None
            },
            _ => return None
        }
    }

    fn write_ppu(&mut self, address: u16, data: u8) {
        match address {
            0x0000 ... 0x1FFF => {
                if self.has_chr_ram {
                    self.chr_rom[address as usize] = data;
                }
            },
            0x2000 ... 0x3FFF => match self.mirroring {
                Mirroring::Horizontal => self.vram[mirroring::horizontal_mirroring(address) as usize] = data,
                Mirroring::Vertical   => self.vram[mirroring::vertical_mirroring(address) as usize] = data,
                _ => {}
            },
            _ => {}
        }
    }
}
