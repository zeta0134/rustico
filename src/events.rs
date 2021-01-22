#[derive(Copy, Clone)]
pub enum Event {
    CloseWindow,
    ApuTogglePulse1,
    ApuTogglePulse2,
    ApuToggleTriangle,
    ApuToggleNoise,
    ApuToggleDmc,
    MouseMove(i32, i32),
    MouseClick(i32, i32),
    MouseRelease,
    MemoryViewerNextPage,
    MemoryViewerPreviousPage,
    MemoryViewerNextBus,
    RequestFrame,
    ShowApuWindow,
    ShowCpuWindow,
    ShowMemoryWindow,
    ShowPpuWindow,
    ShowTestWindow,
    Update,

}