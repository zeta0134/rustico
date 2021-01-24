use events::Event;

use rusticnes_core::nes::NesState;
use rusticnes_core::mmc::none::NoneMapper;

pub struct RuntimeState {
    pub nes: NesState,
    pub running: bool,
    pub file_loaded: bool,
}

impl RuntimeState {
    pub fn new() -> RuntimeState {
        return RuntimeState {
            nes: NesState::new(Box::new(NoneMapper::new())),
            file_loaded: false,
            running: false,
        }
    }

    pub fn load_cartridge(&mut self, file_data: &[u8]) {
        let maybe_nes = NesState::from_rom(&file_data);
        match maybe_nes {
            Ok(nes_state) => {
                self.nes = nes_state;
                self.running = true;
                self.file_loaded = true;
            },
            Err(why) => {
                println!("{}", why);
            }
        }
    }

    pub fn load_sram(&mut self, file_data: &[u8]) {
        if self.nes.mapper.has_sram() {
            self.nes.set_sram(file_data.to_vec());
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Vec<Event> {
        match event {
            Event::ApuTogglePulse1 => {
                self.nes.apu.pulse_1.debug_disable = !self.nes.apu.pulse_1.debug_disable;
            },
            Event::ApuTogglePulse2 => {
                self.nes.apu.pulse_2.debug_disable = !self.nes.apu.pulse_2.debug_disable;
            },
            Event::ApuToggleTriangle => {
                self.nes.apu.triangle.debug_disable = !self.nes.apu.triangle.debug_disable;
            },
            Event::ApuToggleNoise => {
                self.nes.apu.noise.debug_disable = !self.nes.apu.noise.debug_disable;
            },
            Event::ApuToggleDmc => {
                self.nes.apu.dmc.debug_disable = !self.nes.apu.dmc.debug_disable;
            },
            Event::LoadCartridge(file_data_rc) => {
                self.load_cartridge(&file_data_rc);
            },
            Event::NesRunFrame => {
                self.nes.run_until_vblank();
            }
            _ => {}
        }
        return Vec::<Event>::new();
    }
}