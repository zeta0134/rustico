// A very simple Mapper with no esoteric features or bank switching.
// Reference capabilities: https://wiki.nesdev.com/w/index.php/NROM

use ines::INesCartridge;
use memoryblock::MemoryBlock;

use mmc::mapper::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MagicCiramConnection {
    PpuA10,
    PpuA11,
    PpuA12,
    PpuA13,
}

pub struct MagicFloor {
    prg_rom: MemoryBlock,
    prg_ram: MemoryBlock,

    ciram_connection: MagicCiramConnection,
    vram: Vec<u8>,
}

impl MagicFloor {
    pub fn from_ines(ines: INesCartridge) -> Result<MagicFloor, String> {
        let prg_rom_block = ines.prg_rom_block();
        // Oh, this really *shouldn't* be on here, but it doesn't conflict
        // so we might as well.
        let prg_ram_block = ines.prg_ram_block()?;

        let ciram_connection = match ines.header.raw_bytes[6] & 0b0000_1001 {
            0b0000_0000 => MagicCiramConnection::PpuA10,
            0b0000_0001 => MagicCiramConnection::PpuA11,
            0b0000_1000 => MagicCiramConnection::PpuA12,
            0b0000_1001 => MagicCiramConnection::PpuA13,
            _ => unreachable!()
        };

        return Ok(MagicFloor {
            prg_rom: prg_rom_block.clone(),
            prg_ram: prg_ram_block.clone(),
            ciram_connection: ciram_connection,
            vram: vec![0u8; 0x1000],
        });
    }

    pub fn ciram_address(&self, ppu_address: u16) -> u16 {
        return match self.ciram_connection {
            MagicCiramConnection::PpuA10 => ((ppu_address & 0b00_0100_0000_0000) >> 0) | (ppu_address & 0b11_1111_1111),
            MagicCiramConnection::PpuA11 => ((ppu_address & 0b00_1000_0000_0000) >> 1) | (ppu_address & 0b11_1111_1111),
            MagicCiramConnection::PpuA12 => ((ppu_address & 0b01_0000_0000_0000) >> 2) | (ppu_address & 0b11_1111_1111),
            MagicCiramConnection::PpuA13 => ((ppu_address & 0b10_0000_0000_0000) >> 3) | (ppu_address & 0b11_1111_1111),
        }
    }
}

impl Mapper for MagicFloor {
    fn print_debug_status(&self) {
        println!("======= Magic Floor =======");
        println!("CIRAM Mode: {:?}", self.ciram_connection);
        println!("====================");
    }
    
    fn debug_read_cpu(&self, address: u16) -> Option<u8> {
        match address {
            0x6000 ..= 0x7FFF => {self.prg_ram.wrapping_read((address - 0x6000) as usize)},
            0x8000 ..= 0xFFFF => {self.prg_rom.wrapping_read((address - 0x8000) as usize)},
            _ => None
        }
    }

    fn write_cpu(&mut self, address: u16, data: u8) {
        match address {
            0x6000 ..= 0x7FFF => {self.prg_ram.wrapping_write((address - 0x6000) as usize, data);},
            _ => {}
        }
    }

    fn debug_read_ppu(&self, address: u16) -> Option<u8> {
        return Some(self.vram[self.ciram_address(address) as usize]);
    }

    fn write_ppu(&mut self, address: u16, data: u8) {
        let ciram_address = self.ciram_address(address) as usize;
        self.vram[ciram_address] = data;
    }
}
