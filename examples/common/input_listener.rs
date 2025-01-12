use imagic::input::mouse_input::MouseInputListener;
use imagic::input::{MouseEvent, MouseEventType};
use log::info;

pub struct InputListener {
    is_left_button_pressed: bool,
    is_right_button_pressed: bool,
}

impl Default for InputListener {
    fn default() -> Self {
        Self {
            is_left_button_pressed: false,
            is_right_button_pressed: false,
        }
    }
}

impl MouseInputListener for InputListener {
    fn on_mouse_input(&mut self, event: MouseEvent) {
        let mut is_cursor_move = false;
        match event.event_type {
            MouseEventType::LeftPressed => {
                self.is_left_button_pressed = true;
            }
            MouseEventType::LeftReleased => {
                self.is_left_button_pressed = false;
            }
            MouseEventType::RightPressed => {
                self.is_right_button_pressed = true;
            }
            MouseEventType::RightReleased => {
                self.is_right_button_pressed = false;
            }
            MouseEventType::Move => {
                is_cursor_move = true;
            }
            _ => {}
        }
        if !is_cursor_move || self.is_left_button_pressed || self.is_right_button_pressed {
            info!("mouse event: {:?}", event);
        }
    }
}