use std::rc::Rc;

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

    pub fn load_cartridge(&mut self, cart_id: String, file_data: &[u8]) -> Event {
        let maybe_nes = NesState::from_rom(&file_data);
        match maybe_nes {
            Ok(nes_state) => {
                self.nes = nes_state;
                self.running = true;
                self.file_loaded = true;
                return Event::CartridgeLoaded(cart_id);
            },
            Err(why) => {
                return Event::CartridgeRejected(cart_id, why);
            }
        }
    }

    pub fn load_sram(&mut self, file_data: &[u8]) {
        if self.nes.mapper.has_sram() {
            if file_data.len() > 0 {
                self.nes.set_sram(file_data.to_vec());
            }
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Vec<Event> {
        let mut responses: Vec<Event> = Vec::new();
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
            Event::LoadCartridge(cart_id, file_data, sram_data) => {
                responses.push(self.load_cartridge(cart_id, &file_data));
                self.load_sram(&sram_data);
            },
            Event::LoadSram(sram_data) => {
                self.load_sram(&sram_data);
            },
            Event::NesRunFrame => {
                self.nes.run_until_vblank();
            },
            Event::RequestSramSave(sram_id) => {
                if self.nes.mapper.has_sram()  {
                    responses.push(Event::SaveSram(sram_id, Rc::new(self.nes.sram())));
                }
            }
            _ => {}
        }
        return responses;
    }
}