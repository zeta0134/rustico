use std::rc::Rc;

#[derive(Clone)]
pub enum Event {
    CloseWindow,
    ApuTogglePulse1,
    ApuTogglePulse2,
    ApuToggleTriangle,
    ApuToggleNoise,
    ApuToggleDmc,
    LoadCartridge(Rc<Vec<u8>>),
    LoadFailed(String),
    MouseMove(i32, i32),
    MouseClick(i32, i32),
    MouseRelease,
    MemoryViewerNextPage,
    MemoryViewerPreviousPage,
    MemoryViewerNextBus,
    NesRunFrame,
    RequestFrame,
    ShowApuWindow,
    ShowCpuWindow,
    ShowMemoryWindow,
    ShowPpuWindow,
    ShowTestWindow,
    Update,
}