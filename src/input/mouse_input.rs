//! Mouse input processor.

use crate::math::Vec2;

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
    Scroll(Vec2),
    Pinch(f32),
    Move,
}

#[derive(Debug, Copy, Clone)]
pub struct MouseEvent {
    /// Logical position.
    pub logical_pos: Vec2,
    pub event_type: MouseEventType,
}

impl Default for MouseEvent {
    fn default() -> Self {
        Self {
            logical_pos: Vec2::ZERO,
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
    
    pub fn new(logical_pos: Vec2, event_type: MouseEventType) -> Self {
        Self {
            logical_pos,
            event_type
        }
    }
}

pub trait MouseInputListener {
    fn on_mouse_input(&mut self, mouse_event: MouseEvent);
    fn on_update(&mut self) {}
}