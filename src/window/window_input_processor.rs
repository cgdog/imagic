use log::info;
use winit::{
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent},
    // event_loop::EventLoopWindowTarget,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::input::{InputManager, MouseEvent, MouseEventType};


/// Window input proccessor.
/// It processes keyboard or mouse inupts at present.
pub struct WindowInputProcessor {}

impl Default for WindowInputProcessor {
    fn default() -> Self {
        Self {}
    }
}

impl WindowInputProcessor {
    /// Process window input events.
    pub(crate) fn process_window_input(
        event: WindowEvent,
        event_loop: &winit::event_loop::ActiveEventLoop,
        dpi: f64,
        input_manager: &mut InputManager,
    ) {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                Self::process_keyboard_event(event, event_loop, input_manager);
            }
            // TODO: process mouse input
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                Self::process_mouse_button_event(button, state, input_manager);
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase,
            } => {
                Self::process_mouse_scroll_event(delta, phase, dpi, input_manager);
            }
            WindowEvent::CursorMoved { device_id: _, position } => {
                // info!("cursor position: {:?}", position);
                Self::process_mouse_move_event(dpi, position.x, position.y, input_manager);
            }
            WindowEvent::PinchGesture {
                device_id: _,
                delta,
                phase,
            } => {
                Self::process_pinch_gesture(delta, phase, dpi, input_manager);
            }
            _ => (),
        }
    }

    /// Process keyboard inputs.
    fn process_keyboard_event(
        event: KeyEvent,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _input_manager: &InputManager,
    ) {
        match event {
            KeyEvent {
                physical_key,
                logical_key: _,
                text: _,
                location: _,
                state,
                repeat: _,
                ..
            } => match physical_key {
                PhysicalKey::Code(key_code) => {
                    if key_code == KeyCode::Escape && state == ElementState::Released {
                        info!("Press Escape");
                        event_loop.exit();
                    }
                }
                _ => (),
            },
        }
    }

    fn process_mouse_button_event(
        button: MouseButton,
        state: ElementState,
        input_manager: &mut InputManager,
    ) {
        match button {
            MouseButton::Left => match state {
                ElementState::Pressed => {
                    // info!("Left mouse button is pressed");
                    input_manager.trigger_mouse_input_event(MouseEvent::click(MouseEventType::LeftPressed));
                }
                ElementState::Released => {
                    // info!("Left mouse button is released");
                    input_manager.trigger_mouse_input_event(MouseEvent::click(MouseEventType::LeftReleased));
                }
            },
            MouseButton::Right => match state {
                ElementState::Pressed => {
                    info!("Right mouse button is pressed");
                }
                ElementState::Released => {
                    info!("Right mouse button is released");
                }
            },
            MouseButton::Middle => match state {
                ElementState::Pressed => {
                    info!("Middle mouse button is pressed");
                }
                ElementState::Released => {
                    info!("Middle mouse button is released");
                }
            },
            _ => (),
        }
    }

    fn process_mouse_move_event(
        dpi: f64,
        pos_x: f64,
        pos_y: f64,
        input_manager: &mut InputManager,
    ) {
        let logical_x = pos_x / dpi;
        let logical_y = pos_y / dpi;
        input_manager.trigger_mouse_input_event(MouseEvent::new(logical_x, logical_y, MouseEventType::Move));
    }

    fn process_mouse_scroll_event(
        delta: MouseScrollDelta,
        phase: TouchPhase,
        dpi: f64,
        input_manager: &mut InputManager,
    ) {
        // info!("Mouse scroll delta: {:?}, phase: {:?}", delta, phase);
        match phase {
            TouchPhase::Moved => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    info!("LineDelta: {x}, {y}");
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    let x = pos.x / dpi;
                    let y = pos.y / dpi;
                    // info!("PixelDelta: {x}, {y}");
                    input_manager.trigger_mouse_input_event(MouseEvent::new(x, y, MouseEventType::Scroll));
                }
            },
            _ => (),
        }
    }

    /// only supported on macos and iOS
    fn process_pinch_gesture(
        delta: f64,
        _phase: TouchPhase,
        _dpi: f64,
        input_manager: &mut InputManager,
    ) {
        // info!("pinch: delta: {}, phase: {:?}", delta, phase);
        input_manager.trigger_mouse_input_event(MouseEvent::new(delta, 0.0, MouseEventType::Pinch));
    }
}
