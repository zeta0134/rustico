use std::rc::Rc;

#[derive(Clone)]
pub enum Event {
    CloseWindow,
    ApuTogglePulse1,
    ApuTogglePulse2,
    ApuToggleTriangle,
    ApuToggleNoise,
    ApuToggleDmc,
    CartridgeLoaded(String),
    CartridgeRejected(String, String),
    LoadCartridge(String, Rc<Vec<u8>>,Rc<Vec<u8>>),
    LoadSram(Rc<Vec<u8>>),
    LoadFailed(String),
    MouseMove(i32, i32),
    MouseClick(i32, i32),
    MouseRelease,
    MemoryViewerNextPage,
    MemoryViewerPreviousPage,
    MemoryViewerNextBus,
    NesRunFrame,
    RequestFrame,
    RequestCartridgeDialog,
    RequestSramSave(String),
    SaveSram(String, Rc<Vec<u8>>),
    ShowApuWindow,
    ShowCpuWindow,
    ShowMemoryWindow,
    ShowPpuWindow,
    ShowTestWindow,
    Update,
}