#[derive(Copy, Clone)]
pub enum Event {
    CloseWindow,
    RequestFrame,
    ShowCpuWindow,
    ShowPpuWindow,
    ShowTestWindow,
    Update,
}