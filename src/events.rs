#[derive(Copy, Clone)]
pub enum Event {
    CloseWindow,
    RequestFrame,
    ShowPpuWindow,
    ShowTestWindow,
    Update,
}