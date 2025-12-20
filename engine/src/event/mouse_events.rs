use crate::{event::{events::Events, Event, EventData}, impl_as_any, math::Vec2};

/// General mouse event data.
#[derive(Debug, Clone, Copy)]
pub enum GeneralMouseEventData {
    LeftButtonDown(Vec2),
    LeftButtonUp(Vec2),
    RightButtonDown(Vec2),
    RightButtonUp(Vec2),
    MiddleButtonDown(Vec2),
    MiddleButtonUp(Vec2),
    Move(Vec2),
    /// delta, position
    Wheel(f32, Vec2),
}

impl EventData for GeneralMouseEventData {
    impl_as_any!();
}

impl GeneralMouseEventData {
    
    pub fn new_left_button_down_event(position: Vec2) -> Event {
        Event {
            id: Events::EVENT_ID_GENERAL_MOUSE_EVENT,
            data: Some(Box::new(Self::LeftButtonDown(position))),
        }
    }

    pub fn new_left_button_up_event(position: Vec2) -> Event {
        Event {
            id: Events::EVENT_ID_GENERAL_MOUSE_EVENT,
            data: Some(Box::new(Self::LeftButtonUp(position))),
        }
    }

    pub fn new_right_button_down_event(position: Vec2) -> Event {
        Event {
            id: Events::EVENT_ID_GENERAL_MOUSE_EVENT,
            data: Some(Box::new(Self::RightButtonDown(position))),
        }
    }

    pub fn new_right_button_up_event(position: Vec2) -> Event {
        Event {
            id: Events::EVENT_ID_GENERAL_MOUSE_EVENT,
            data: Some(Box::new(Self::RightButtonUp(position))),
        }
    }

    pub fn new_middle_button_down_event(position: Vec2) -> Event {
        Event {
            id: Events::EVENT_ID_GENERAL_MOUSE_EVENT,
            data: Some(Box::new(Self::MiddleButtonDown(position))),
        }
    }

    pub fn new_middle_button_up_event(position: Vec2) -> Event {
        Event {
            id: Events::EVENT_ID_GENERAL_MOUSE_EVENT,
            data: Some(Box::new(Self::MiddleButtonUp(position))),
        }
    }

    pub fn new_move_event(position: Vec2) -> Event {
        Event {
            id: Events::EVENT_ID_GENERAL_MOUSE_EVENT,
            data: Some(Box::new(Self::Move(position))),
        }
    }

    pub fn new_wheel_event(delta: f32, position: Vec2) -> Event {
        Event {
            id: Events::EVENT_ID_GENERAL_MOUSE_EVENT,
            data: Some(Box::new(Self::Wheel(delta, position))),
        }
    }
}