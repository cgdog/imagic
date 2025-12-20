use glam::Vec2;
use winit::{
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent},
    // event_loop::EventLoopWindowTarget,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::input::input_manager::InputManager;

// use crate::input::{InputManager, MouseEvent, MouseEventType};

/// Window input proccessor.
/// It processes keyboard or mouse inupts at present.
pub(crate) struct WindowInputProcessor {
    _cursor_logical_pos: Vec2,
    // _event_dispatcher: RR<EventDispatcher>,
}


impl WindowInputProcessor {

    pub(crate) fn new() -> Self {
        Self {
            _cursor_logical_pos: Vec2::ZERO,
        }
    }

    /// Process window input events.
    pub(crate) fn process_window_input(
        &mut self,
        event: WindowEvent,
        event_loop: &winit::event_loop::ActiveEventLoop,
        dpi: f64,
        input_manager: &mut InputManager,
        is_ui_interacting: bool,
    ) {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                self.process_keyboard_event(event, event_loop, input_manager);
            }
            // TODO: process mouse input
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                if !is_ui_interacting {
                    self.process_mouse_button_event(button, state, input_manager);
                }
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase,
            } => {
                if !is_ui_interacting {
                    self.process_mouse_scroll_event(delta, phase, dpi, input_manager);
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                // CursorMoved event always occurs before MouseInput event(e.g., left or right mouse click).
                // info!("cursor position: {:?}", position);
                if !is_ui_interacting {
                    self.process_mouse_move_event(dpi, position.x, position.y, input_manager);
                }
            }
            // WindowEvent::PinchGesture {
            //     device_id: _,
            //     delta,
            //     phase,
            // } => {
            //     self.process_pinch_gesture(delta, phase, dpi, input_manager);
            // }
            _ => (),
        }
    }

    /// Process keyboard inputs.
    fn process_keyboard_event(
        &mut self,
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
                        log::info!("Press Escape");
                        // self._event_dispatcher.borrow_mut().emit(Events::EVENT_WINDOW_CLOSED);
                        event_loop.exit();
                    }
                }
                _ => (),
            },
        }
    }

    fn process_mouse_button_event(
        &mut self,
        button: MouseButton,
        state: ElementState,
        input_manager: &mut InputManager,
    ) {
        match button {
            MouseButton::Left => match state {
                ElementState::Pressed => {
                    // self._event_dispatcher.borrow_mut().emit(
                    //     GeneralMouseEventData::new_left_button_down_event(
                    //         self._cursor_logical_pos,
                    //     ),
                    // );

                    input_manager.on_mouse_left_button_down(&self._cursor_logical_pos);
                    // info!("Left mouse button is pressed at: ({}, {})", self._cursor_logical_pos.x, self._cursor_logical_pos.y);
                    // input_manager
                    //     .trigger_mouse_input_event(MouseEvent::new(self.cursor_logical_pos, MouseEventType::LeftPressed));
                }
                ElementState::Released => {
                    input_manager.on_mouse_left_button_up(&self._cursor_logical_pos);
                    // self._event_dispatcher.borrow_mut().emit(
                    //     GeneralMouseEventData::new_left_button_up_event(
                    //         self._cursor_logical_pos,
                    //     ),
                    // );
                    // info!("Left mouse button is released at: ({}, {})", self._cursor_logical_pos.x, self._cursor_logical_pos.y);
                    // input_manager
                    //     .trigger_mouse_input_event(MouseEvent::new(self.cursor_logical_pos, MouseEventType::LeftReleased));
                }
            },
            MouseButton::Right => match state {
                ElementState::Pressed => {
                    input_manager.on_mouse_right_button_down(&self._cursor_logical_pos);
                    // self._event_dispatcher.borrow_mut().emit(
                    //     GeneralMouseEventData::new_right_button_down_event(
                    //         self._cursor_logical_pos,
                    //     ),
                    // );
                    // info!("Right mouse button is pressed at: ({}, {})", self._cursor_logical_pos.x, self._cursor_logical_pos.y);
                }
                ElementState::Released => {
                    input_manager.on_mouse_right_button_up(&self._cursor_logical_pos);
                    // self._event_dispatcher.borrow_mut().emit(
                    //     GeneralMouseEventData::new_right_button_up_event(
                    //         self._cursor_logical_pos,
                    //     ),
                    // );
                    // info!("Right mouse button is released at: ({}, {})", self._cursor_logical_pos.x, self._cursor_logical_pos.y);
                }
            },
            MouseButton::Middle => match state {
                ElementState::Pressed => {
                    input_manager.on_mouse_middle_button_down(&self._cursor_logical_pos);
                    log::info!("Middle mouse button is pressed at: ({}, {})", self._cursor_logical_pos.x, self._cursor_logical_pos.y);
                }
                ElementState::Released => {
                    input_manager.on_mouse_middle_button_up(&self._cursor_logical_pos);
                    log::info!("Middle mouse button is released at: ({}, {})", self._cursor_logical_pos.x, self._cursor_logical_pos.y);
                }
            },
            _ => (),
        }
    }

    fn process_mouse_move_event(
        &mut self,
        dpi: f64,
        pos_x: f64,
        pos_y: f64,
        input_manager: &mut InputManager,
    ) {
        let logical_x = pos_x / dpi;
        let logical_y = pos_y / dpi;
        self._cursor_logical_pos = Vec2::new(logical_x as f32, logical_y as f32);
        input_manager.on_mouse_move(&self._cursor_logical_pos);

        // log::info!("window input processor, mouse pos: {}", self._cursor_logical_pos);
        // input_manager.trigger_mouse_input_event(MouseEvent::new(
        //     self.cursor_logical_pos,
        //     MouseEventType::Move,
        // ));
    }

    fn process_mouse_scroll_event(
        &mut self,
        delta: MouseScrollDelta,
        phase: TouchPhase,
        _dpi: f64,
        _input_manager: &mut InputManager,
    ) {
        match phase {
            TouchPhase::Started => {
                // log::info!("Scroll started.");// no output on Windows.
            }
            TouchPhase::Moved => match delta {
                #[allow(unused)]
                MouseScrollDelta::LineDelta(x, y) => {
                    // log::info!("LineDelta: {x}, {y}");
                    // TODO: Windows execute the branch. Select a proper speed.
                    // (x, y) is (0, 1) or (0, -1), or other intergers on Windows.
                    // let speed = 10.0;
                    // input_manager.trigger_mouse_input_event(MouseEvent::new(
                    //     self.cursor_logical_pos,
                    //     MouseEventType::Scroll(Vec2::new(x as f32 * speed, y as f32 * speed)),
                    // ));
                }
                MouseScrollDelta::PixelDelta(_pos) => {
                    // let x = pos.x / dpi;
                    // let y = pos.y / dpi;
                    // info!("PixelDelta: {x}, {y}");
                    // input_manager.trigger_mouse_input_event(MouseEvent::new(
                    //     self.cursor_logical_pos,
                    //     MouseEventType::Scroll(Vec2::new(x as f32, y as f32)),
                    // ));
                }
            },
            TouchPhase::Ended | TouchPhase::Cancelled => {
                // log::info!("Scroll ended");// no output on Windows.
            }
        }
    }

    /// only supported on macos and iOS
    #[allow(unused)]
    fn process_pinch_gesture(
        &mut self,
        delta: f64,
        _phase: TouchPhase,
        _dpi: f64,
        // input_manager: &mut InputManager,
    ) {
        // info!("pinch: delta: {}, phase: {:?}", delta, phase);
        // input_manager.trigger_mouse_input_event(MouseEvent::new(
        //     self.cursor_logical_pos,
        //     MouseEventType::Pinch(delta as f32),
        // ));
    }
}
