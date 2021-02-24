use std::rc::Rc;

use events::Event;
use events::StandardControllerButton;

use rusticnes_core::nes::NesState;
use rusticnes_core::mmc::none::NoneMapper;
use rusticnes_core::cartridge::mapper_from_file;

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
        let maybe_mapper = mapper_from_file(file_data);
        match maybe_mapper {
            Ok(mapper) => {
                self.nes = NesState::new(mapper);
                self.nes.power_on();
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

    pub fn button_press(&mut self, player_index: usize, button: StandardControllerButton) {
        let controllers = [
            &mut self.nes.p1_input,
            &mut self.nes.p2_input
        ];

        if player_index > controllers.len() {
            return;
        }

        let old_controller_byte = *controllers[player_index];
        let pressed_button = 0b1 << (button.clone() as u8);
        let new_controller_byte = old_controller_byte | pressed_button;
        let fixed_controller_byte = fix_dpad(new_controller_byte, button.clone());
        *controllers[player_index] = fixed_controller_byte;
    }

    pub fn button_release(&mut self, player_index: usize, button: StandardControllerButton) {
        let controllers = [
            &mut self.nes.p1_input,
            &mut self.nes.p2_input
        ];

        if player_index > controllers.len() {
            return;
        }

        let old_controller_byte = *controllers[player_index];
        let released_button = 0b1 << (button as u8);
        let release_mask = 0b1111_1111 ^ released_button;
        let new_controller_byte = old_controller_byte & release_mask;
        *controllers[player_index] = new_controller_byte;
    }

    pub fn handle_event(&mut self, event: Event) -> Vec<Event> {
        let mut responses: Vec<Event> = Vec::new();
        match event {
            Event::MuteChannel(channel_index) => {
                self.nes.apu.mute_channel(&mut *self.nes.mapper, channel_index);
            },
            Event::UnmuteChannel(channel_index) => {
                self.nes.apu.unmute_channel(&mut *self.nes.mapper, channel_index);  
            },
            
            Event::LoadCartridge(cart_id, file_data, sram_data) => {
                responses.push(self.load_cartridge(cart_id, &file_data));
                self.load_sram(&sram_data);
            },
            Event::LoadSram(sram_data) => {
                self.load_sram(&sram_data);
            },
            Event::NesRunCycle => {
                self.nes.cycle();
            },
            Event::NesRunFrame => {
                self.nes.run_until_vblank();
            },
            Event::NesRunOpcode => {
                self.nes.step();
            },
            Event::NesRunScanline => {
                self.nes.run_until_hblank();
            },
            Event::NesReset => {
                self.nes.reset();
            },
            
            // These three events should ideally move to some sort of FrameTiming manager
            Event::NesPauseEmulation => {
                self.running = false;
            },
            Event::NesResumeEmulation => {
                self.running = true;
            },
            Event::NesToggleEmulation => {
                self.running = !self.running;
            },

            Event::RequestSramSave(sram_id) => {
                if self.nes.mapper.has_sram()  {
                    responses.push(Event::SaveSram(sram_id, Rc::new(self.nes.sram())));
                }
            },

            // Input is due for an overhaul. Ideally the IoBus should handle its own
            // events, rather than doing this here.
            Event::StandardControllerPress(controller_index, button) => {
                self.button_press(controller_index, button);
            },
            Event::StandardControllerRelease(controller_index, button) => {
                self.button_release(controller_index, button);
            },
            _ => {}
        }
        return responses;
    }
}

pub fn fix_dpad(controller_byte: u8, last_button_pressed: StandardControllerButton) -> u8 {
    let mut fixed_byte = controller_byte;
    match last_button_pressed {
        StandardControllerButton::DPadUp => {fixed_byte &= 0b1101_1111},
        StandardControllerButton::DPadDown => {fixed_byte &= 0b1110_1111},
        StandardControllerButton::DPadLeft => {fixed_byte &= 0b0111_1111},
        StandardControllerButton::DPadRight => {fixed_byte &= 0b1011_1111},
        _ => {}
    }

    return fixed_byte;
}