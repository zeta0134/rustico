use events::Event;
use drawing::SimpleBuffer;

pub trait Panel {
    fn title(&self) -> String;
    fn handle_event(Event) -> [Event];
    fn active_canvas<'a>(&'a self) -> &'a SimpleBuffer;
}