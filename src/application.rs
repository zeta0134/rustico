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
}