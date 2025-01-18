//! Mouse input processor.

/// Mouse event type.
#[derive(Debug, Copy, Clone)]
pub enum MouseEventType {
    None,
    LeftPressed,
    RightPressed,
    MiddlePressed,
    LeftReleased,
    RightReleased,
    MiddleReleased,
    Scroll,
    Pinch,
    Move,
}

#[derive(Debug, Copy, Clone)]
pub struct MouseEvent {
    pub x: f32,
    pub y: f32,
    pub event_type: MouseEventType,
}

impl Default for MouseEvent {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            event_type: MouseEventType::None,
        }
    }
}

impl MouseEvent {
    pub fn click (event_type: MouseEventType) -> Self {
        Self {
            event_type,
            ..Default::default()
        }
    }
    
    pub fn new(x: f32, y: f32, event_type: MouseEventType) -> Self {
        Self {
            x,
            y, 
            event_type
        }
    }
}

pub trait MouseInputListener {
    fn on_mouse_input(&mut self, mouse_event: MouseEvent);
}