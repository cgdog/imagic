use std::any::Any;

pub mod event_dispatcher;
pub mod events;
pub mod mouse_events;

pub type EventID = u32;

pub trait EventData : Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct Event {
    pub id: EventID,
    pub data: Option<Box<dyn EventData>>,
}

pub trait EventHandler {
    fn handle(&mut self, event: &Event);
}