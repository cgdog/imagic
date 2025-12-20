use std::collections::HashMap;

use crate::{event::{Event, EventHandler, EventID}, types::RR, RR_new};

pub type EventHandle = usize;
pub const INVALID_EVENT_HANDLE: EventHandle = EventHandle::MAX;

pub struct EventDispatcher {
    _handlers: HashMap<EventID, Vec<(EventHandle, RR<dyn EventHandler>)>>,
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self {
            _handlers: HashMap::new(),
        }
    }
}


impl EventDispatcher {
    pub fn new() -> RR<Self> {
        RR_new!(Self::default())
    }

    fn next_handle() -> EventHandle {
        static mut _HANDLE: EventHandle = 0;
        unsafe {
            let handle = _HANDLE;
            _HANDLE += 1;
            handle
        }
    }

    pub fn register(&mut self, event_id: EventID, handler: RR<dyn EventHandler>) -> EventHandle{
        let event_handle = Self::next_handle();
        self._handlers.entry(event_id).or_insert(Vec::new()).push((event_handle, handler));
        event_handle
    }

    pub fn unregister(&mut self, event_handle: EventHandle) {
        for (_, handlers) in &mut self._handlers {
            handlers.retain(|(handle, _handler)| {
                *handle != event_handle
            });
        }
    }

    pub fn emit(&mut self, event: Event) {
        if let Some(handlers) = self._handlers.get_mut(&event.id) {
            for (_handle, handler) in handlers {
                handler.borrow_mut().handle(&event);
            }
        }
    }
}