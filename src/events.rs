use std::rc::Rc;

#[derive(Clone)]
pub enum StandardControllerButton {
    A,
    B,
    Select,
    Start,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

#[derive(Clone)]
pub enum Event {
    CloseWindow,
    CartridgeLoaded(String),
    CartridgeRejected(String, String),
    GameToggleOverscan,
    GameIncreaseScale,
    GameDecreaseScale,
    LoadCartridge(String, Rc<Vec<u8>>,Rc<Vec<u8>>),
    LoadSram(Rc<Vec<u8>>),
    LoadFailed(String),
    MouseMove(i32, i32),
    MouseClick(i32, i32),
    MouseRelease,
    MemoryViewerNextPage,
    MemoryViewerPreviousPage,
    MemoryViewerNextBus,
    MuteChannel(usize),
    UnmuteChannel(usize),
    NesPauseEmulation,
    NesResumeEmulation,
    NesToggleEmulation,
    NesReset,
    NesRunCycle,
    NesRunFrame,
    NesRunOpcode,
    NesRunScanline,
    RequestFrame,
    RequestCartridgeDialog,
    RequestSramSave(String),
    SaveSram(String, Rc<Vec<u8>>),
    ShowApuWindow,
    ShowCpuWindow,
    ShowGameWindow,
    ShowMemoryWindow,
    ShowPpuWindow,
    ShowTestWindow,
    StandardControllerPress(usize, StandardControllerButton),
    StandardControllerRelease(usize, StandardControllerButton),
    Update,
}