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
    ApplyBooleanSetting(String, bool),
    ApplyFloatSetting(String, f64),
    ApplyIntegerSetting(String, i64),
    ApplyStringSetting(String, String),
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
    NesNudgeAlignment,
    NesNewApuHalfFrame,
    NesNewApuQuarterFrame,
    NesNewFrame,
    NesNewScanline,
    NesPauseEmulation,
    NesRenderNTSC(usize),
    NesResumeEmulation,
    NesReset,
    NesRunCycle,
    NesRunFrame,
    NesRunOpcode,
    NesRunScanline,
    NesToggleEmulation,
    RequestFrame,
    RequestCartridgeDialog,
    RequestSramSave(String),
    SaveSram(String, Rc<Vec<u8>>),
    ShowApuWindow,
    ShowCpuWindow,
    ShowGameWindow,
    ShowEventWindow,
    ShowMemoryWindow,
    ShowPianoRollWindow,
    ShowPpuWindow,
    ShowTestWindow,
    StandardControllerPress(usize, StandardControllerButton),
    StandardControllerRelease(usize, StandardControllerButton),
    StoreBooleanSetting(String, bool),
    StoreFloatSetting(String, f64),
    StoreIntegerSetting(String, i64),
    StoreStringSetting(String, String),
    ToggleBooleanSetting(String),
    Update,
}
