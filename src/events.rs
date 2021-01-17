#[derive(Copy, Clone)]
pub enum Event {
    CloseWindow,
    RequestFrame,
    ShowVramWindow,
    ShowTestWindow,
    Update,
}