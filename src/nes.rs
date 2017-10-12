use apu::ApuState;
use cartridge;
use cycle_cpu;
use cpu::Registers;
use cycle_cpu::CpuState;
use memory;
use memory::CpuMemory;
use ppu::PpuState;
use mmc::mapper::Mapper;

pub struct NesState {
    pub apu: ApuState,
    pub cpu: CpuState,
    pub memory: CpuMemory,
    pub ppu: PpuState,
    pub registers: Registers,
    pub master_clock: u64,
    pub p1_input: u8,
    pub p1_data: u8,
    pub p2_input: u8,
    pub p2_data: u8,
    pub input_latch: bool,
    pub mapper: Box<Mapper>,
}

impl NesState {
    pub fn new(m: Box<Mapper>) -> NesState {
        return NesState {
            apu: ApuState::new(),
            cpu: CpuState::new(),
            memory: CpuMemory::new(),
            ppu: PpuState::new(),
            registers: Registers::new(),
            master_clock: 0,
            p1_input: 0,
            p1_data: 0,
            p2_input: 0,
            p2_data: 0,
            input_latch: false,
            mapper: m,
        }
    }

    pub fn cycle(&mut self) {
        cycle_cpu::run_one_clock(self);
        self.master_clock = self.master_clock + 12;
        self.ppu.run_to_cycle(&mut *self.mapper, self.master_clock);
        self.apu.clock_apu(&mut *self.mapper);
    }

    pub fn step(&mut self) {
        // Start this instruction
        self.cycle();
        let mut i = 0;
        while self.cpu.tick >= 1 && i < 10 {
            // Continue until this instruction terminates or halts
            self.cycle();
            i += 1;
        }
    }

    pub fn run_until_hblank(&mut self) {
        let old_scanline = self.ppu.current_scanline;
        while old_scanline == self.ppu.current_scanline {
            self.step();
        }
    }

    pub fn run_until_vblank(&mut self) {
        while self.ppu.current_scanline == 242 {
            self.step();
        }
        while self.ppu.current_scanline != 242 {
            self.step();
        }
    }
}

use std::error::Error;
use std::fs::File;
use std::io::Read;

pub fn open_file(file_path: &str) -> Option<NesState> {
    let mut file = match File::open(file_path) {
        Err(why) => panic!("Couldn't open {}: {}", file_path, why.description()),
        Ok(file) => file,
    };
    // Read the whole thing
    let mut cartridge = Vec::new();
    match file.read_to_end(&mut cartridge) {
        Err(why) => panic!("Couldn't read data: {}", why.description()),
        Ok(bytes_read) => {
            println!("Data read successfully: {}", bytes_read);

            let nes_header = cartridge::extract_header(&cartridge);
            cartridge::print_header_info(nes_header);
            let mapper = cartridge::load_from_cartridge(nes_header, &cartridge);
            let mut nes = NesState::new(mapper);
            nes.apu.buffer_full = false;

            // Initialize CPU register state for power-up sequence
            nes.registers.a = 0;
            nes.registers.y = 0;
            nes.registers.x = 0;
            nes.registers.s = 0xFD;

            let pc_low = memory::read_byte(&mut nes, 0xFFFC);
            let pc_high = memory::read_byte(&mut nes, 0xFFFD);
            nes.registers.pc = pc_low as u16 + ((pc_high as u16) << 8);
            return Some(nes);
        },
    };
    return None;
}