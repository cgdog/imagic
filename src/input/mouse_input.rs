//! Mouse input processor.

use std::any::Any;

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

/// Mouse event.
/// 
/// It includes mouse position and event type.
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
    
    pub fn new(logical_pos: Vec2, event_type: MouseEventType) -> Self {
        Self {
            logical_pos,
            event_type
        }
    }
}

/// Mouse input listener trait.
/// 
/// Structs implemented this trait can listen mouse input.
pub trait MouseInputListener : Any{
    fn on_mouse_input(&mut self, mouse_event: MouseEvent);
    fn on_update(&mut self) {}
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}