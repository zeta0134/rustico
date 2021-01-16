use events::Event;
use drawing::SimpleBuffer;

pub trait Panel {
    fn title(&self) -> &str;
    fn handle_event(&mut self, event: Event) -> Vec<Event>;
    fn active_canvas(&self) -> &SimpleBuffer;
}