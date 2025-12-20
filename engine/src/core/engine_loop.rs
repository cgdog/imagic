use crate::{
    core::Engine, graphics::graphics_context::GraphicsContext,
    window::Window
};

impl winit::application::ApplicationHandler for Engine {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if !self._is_inited {
            let engine_option = &self.options;
            let main_window =
                Window::new(event_loop, engine_option.window_size, engine_option.app_name);
            let graphics_context = GraphicsContext::new(
                main_window,
            );
            self.init(graphics_context);
            self._is_inited = true;
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        self.process_window_events(event_loop, event);
    }
}

impl Engine {

    pub fn process_window_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: winit::event::WindowEvent,
    ) {
        {
            if let Some(ui_renderer) = &mut self.frame_renderer.ui_renderer {
                let graphics_context = self._graphics_context.as_mut().unwrap();
                ui_renderer.handle_input(graphics_context.main_window().get_ref(), &event);
            }
        }
        match event {
            winit::event::WindowEvent::CloseRequested => {
                self.stop();
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(new_physical_size) => {
                self.on_resize(new_physical_size);
            }
            winit::event::WindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer: _,
            } => {
                self.graphics_context().on_dpi_changed(scale_factor);
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.on_update();
            }
            others => {
                let is_ui_interacting = {
                    if let Some(ui_renderer) = &self.frame_renderer.ui_renderer {
                        ui_renderer.state().egui_ctx().wants_pointer_input()
                    } else {
                        false
                    }
                };
                self._window_input_processor.process_window_input(
                    others,
                    event_loop,
                    self._graphics_context.as_mut().unwrap().dpi(),
                    &mut self.input_manager,
                    is_ui_interacting,
                );
            }
        }
    }
}
