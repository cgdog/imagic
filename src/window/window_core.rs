use log::info;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use winit::{dpi::LogicalSize, event_loop::EventLoopWindowTarget};
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::event::WindowEvent;

use winit::window::Window as WindowWinit;

use crate::graphics;
use crate::imagic_core::imagic_app::ImagicAppTrait;
use crate::imagic_core::imagic_context::ImagicContext;

use super::WindowSize;

pub struct Window {
    window: Option<Arc<WindowWinit>>,
    logical_size: WindowSize,
    physical_size: WindowSize,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            window: None,
            logical_size: WindowSize::default(),
            physical_size: WindowSize::default(),
        }
    }
}

impl Window {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&self) -> Arc<WindowWinit> {
        match &self.window {
            Some(window) => window.clone(),
            None => panic!("Now window is None.")
        }
    }

    pub fn get_physical_size(&self) -> &WindowSize {
        &self.physical_size
    }

    pub fn set_physical_size(&mut self, width: f32, height: f32) {
        self.physical_size.set(width, height);
    }

    pub fn get_logical_size(&self) -> &WindowSize {
        &self.logical_size
    }

    pub fn set_logical_size(&mut self, width: f32, height: f32) {
        self.logical_size.set(width, height);
    }

    pub fn init(&mut self, event_loop: &EventLoop<()>, window_size: WindowSize, window_title: &'static str) {
        let builder = winit::window::WindowBuilder::new()
            .with_title(window_title);
        let window = builder.build(&event_loop).unwrap();
        let window = Arc::new(window);

        let _ = window.request_inner_size(LogicalSize::new(window_size.get_width(), window_size.get_height()));
        // let scale_factor = window.scale_factor();
        let physical_size = window.inner_size();
        self.set_physical_size(physical_size.width as f32, physical_size.height as f32);
        self.window = Some(window);

        // info!("Window is inited. scale_factor: {}", scale_factor);
    }

    pub fn process_window_event(&mut self, event: Event<()>, elwt: &EventLoopWindowTarget<()>,
        renderer: &mut graphics::Renderer, context: &mut ImagicContext, app: &Option<Rc<RefCell<Box<dyn ImagicAppTrait>>>>) {
        
        match event {
            Event::WindowEvent { event , ..} => {
                renderer.ui_renderer().handle_input(&self.get(), &event);
                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::Resized(new_size) => {
                        context.graphics_context_mut().on_resize(new_size);
                        self.get().request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        match app {
                            Some(app) => app.borrow_mut().on_update(context, renderer.ui_renderer()),
                            None => info!("No app supplied."),
                        }
                        renderer.render(context, &self.get());
                        self.get().request_redraw();
                    }
                    _ => (),
                }
                
            }
            
            _ => (),
        }
    }

}