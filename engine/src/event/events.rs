use crate::event::{Event, EventID};

/// Builtin events

pub struct Events {
}

impl Events {
    pub const BUILTIN_EVENT_ID_START: EventID = 0;
    pub const EVENT_ID_WINDOW_CLOSED: EventID = Self::BUILTIN_EVENT_ID_START + 0;
    /// General mouse event.
    pub const EVENT_ID_GENERAL_MOUSE_EVENT: EventID = Self::BUILTIN_EVENT_ID_START + 1;
    /// Mouse left button pressed event.
    pub const EVENT_ID_MOUSE_LEFT_BUTTON_PRESSED: EventID = Self::BUILTIN_EVENT_ID_START + 2;
    pub const EVENT_ID_MOUSE_MOVE: EventID = Self::BUILTIN_EVENT_ID_START + 1;


    pub const BUILTIN_EVENT_ID_END: EventID = 511;

    /// User custom event id start from `512`.
    /// Events with id less then 512 are builtin events.
    pub const CUSTOM_EVENT_ID_START: EventID = Self::BUILTIN_EVENT_ID_END + 1;

    /// Builtin window closed event.
    pub const EVENT_WINDOW_CLOSED: Event = Event {
        id: Self::EVENT_ID_WINDOW_CLOSED,
        data: None,
    };
}