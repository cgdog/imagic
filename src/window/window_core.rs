use log::info;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use winit::dpi::Size;

use winit::event::WindowEvent;
use winit::{dpi::LogicalSize, event_loop::ActiveEventLoop};

use winit::window::Window as WindowWinit;

use crate::graphics;
use crate::imagic_core::imagic_app::ImagicAppTrait;
use crate::imagic_core::imagic_context::ImagicContext;
use crate::input::InputManager;

use super::{WindowInputProcessor, WindowSize};

pub struct Window {
    window: Option<Arc<WindowWinit>>,
    logical_size: WindowSize,
    physical_size: WindowSize,
    /// see https://docs.rs/dpi/0.1.1/dpi/index.html
    dpi: f64,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            window: None,
            logical_size: WindowSize::default(),
            physical_size: WindowSize::default(),
            dpi: 1.0,
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
            None => panic!("Now window is None."),
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

    pub fn init(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_size: WindowSize,
        window_title: &'static str,
    ) {
        // create the window.
        let logical_size = LogicalSize::new(
            window_size.get_width() as f64,
            window_size.get_height() as f64,
        );
        let window_attributes = WindowWinit::default_attributes()
            .with_title(window_title)
            .with_inner_size(Size::Logical(logical_size));

        let window = event_loop.create_window(window_attributes).unwrap();
        self.dpi = window.scale_factor();
        info!("window dpi: {0}", self.dpi);
        let window = Arc::new(window);

        // let scale_factor = window.scale_factor();
        let physical_size = window.inner_size();
        self.set_physical_size(physical_size.width as f32, physical_size.height as f32);
        self.window = Some(window);
    }

    pub fn process_window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: winit::event::WindowEvent,
        renderer: &mut graphics::Renderer,
        context: &mut ImagicContext,
        app: &Option<Rc<RefCell<Box<dyn ImagicAppTrait>>>>,
        input_manager: &mut InputManager,
    ) {
        renderer.ui_renderer().handle_input(&self.get(), &event);
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                context.on_resize(new_size);
                self.get().request_redraw();
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer: _,
            } => {
                self.dpi = scale_factor;
            }
            WindowEvent::RedrawRequested => {
                match app {
                    Some(app) => app.borrow_mut().on_update(context),
                    None => info!("No app supplied."),
                }
                renderer.render(context, &self.get());
                self.get().request_redraw();
            }
            others => {
                WindowInputProcessor::process_window_input(others, event_loop, self.dpi, input_manager);
            }
        }
    }
}
