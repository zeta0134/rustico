use events::Event;

use rusticnes_core::nes::NesState;
use rusticnes_core::mmc::none::NoneMapper;

pub struct RuntimeState {
    pub nes: NesState,
}

impl RuntimeState {
    pub fn new() -> RuntimeState {
        return RuntimeState {
            nes: NesState::new(Box::new(NoneMapper::new())),
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
            _ => {}
        }
        return Vec::<Event>::new();
    }
}