#[derive(Copy, Clone)]
pub enum Event {
    CloseWindow,
    ApuTogglePulse1,
    ApuTogglePulse2,
    ApuToggleTriangle,
    ApuToggleNoise,
    ApuToggleDmc,
    RequestFrame,
    ShowApuWindow,
    ShowCpuWindow,
    ShowPpuWindow,
    ShowTestWindow,
    Update,
}