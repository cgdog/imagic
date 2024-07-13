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

pub struct Window {
    window: Option<Arc<WindowWinit>>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            window: None,
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

    pub fn init(&mut self, event_loop: &EventLoop<()>, width: f64, height: f64, window_title: &'static str) {
        let builder = winit::window::WindowBuilder::new()
            .with_title(window_title);
        let window = builder.build(&event_loop).unwrap();
        let window = Arc::new(window);

        #[cfg(debug)]
        {
            if width > 2048.0 {
                info!("Window width: {width} is large than 2048.");
            }
        }
        let width = f64::min(width, 2048.0);
        #[cfg(debug)]
        {
            if height > 2048.0 {
                info!("Window width width: {height} is large than 2048.");
            }
        }
        let height = f64::min(height, 2048.0);
        let _ = window.request_inner_size(LogicalSize::new(width, height));
        self.window = Some(window);

        info!("Window is inited.");
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