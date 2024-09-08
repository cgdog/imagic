use log::info;
use winit::{event::{ElementState, KeyEvent, WindowEvent}, event_loop::EventLoopWindowTarget, keyboard::{KeyCode, PhysicalKey}};
/// Window input proccessor.
/// It processes keyboard or mouse inupts at present.
pub struct WindowInputProcessor {

}

impl Default for WindowInputProcessor {
    fn default() -> Self {
        Self {

        }
    }
}

impl WindowInputProcessor {

    /// Process window input events.
    pub fn process_window_input(event: WindowEvent, elwt: &EventLoopWindowTarget<()>) {
        match event {
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _} => {
                WindowInputProcessor::process_keyboard_event(event, elwt);
            }
            _ => (),
        }
    }

    /// Process keyboard inputs.
    pub fn process_keyboard_event(event: KeyEvent, elwt: &EventLoopWindowTarget<()>) {
        match event {
            KeyEvent { physical_key, logical_key: _, text: _, location: _, state, repeat: _, .. } => {
                match physical_key {
                    PhysicalKey::Code(key_code) => {
                        if key_code == KeyCode::Escape && state == ElementState::Released {
                            info!("Press Escape");
                            elwt.exit();
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}