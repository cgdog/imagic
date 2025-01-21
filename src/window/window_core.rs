use log::info;
use std::sync::Arc;
use winit::dpi::Size;

use winit::event::WindowEvent;
use winit::{dpi::LogicalSize, event_loop::ActiveEventLoop};

use winit::window::Window as WindowWinit;

use crate::graphics;
use crate::imagic_core::imagic_app::ImagicAppTrait;
use crate::imagic_core::imagic_context::ImagicContext;

use super::{WindowInputProcessor, WindowSize};

pub struct Window {
    window: Option<Arc<WindowWinit>>,
    logical_size: WindowSize,
    physical_size: WindowSize,
    /// see https://docs.rs/dpi/0.1.1/dpi/index.html
    dpi: f64,
    window_input_processor: WindowInputProcessor,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            window: None,
            logical_size: WindowSize::default(),
            physical_size: WindowSize::default(),
            dpi: 1.0,
            window_input_processor: Default::default()
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
        let (logical_width, logical_height) = window_size.get();
        self.set_logical_size(logical_width, logical_height);
        // create the window.
        let logical_size = LogicalSize::new(
            logical_width as f64,
            logical_height as f64,
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
        app: &mut Box<dyn ImagicAppTrait>,
    ) {
        renderer.ui_renderer().handle_input(&self.get(), &event);
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_physical_size) => {
                info!("on resize...");
                self.dpi = self.get().scale_factor();
                let new_logical_size: LogicalSize<u32> = new_physical_size.to_logical(self.dpi);

                context.on_resize(new_physical_size, new_logical_size);
                self.get().request_redraw();
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer: _,
            } => {
                self.dpi = scale_factor;
            }
            WindowEvent::RedrawRequested => {
                app.on_update(context);
                context.on_update();
                renderer.render(context, &self.get(), app);
                self.get().request_redraw();
            }
            others => {
                self.window_input_processor.process_window_input(others, event_loop, self.dpi, context.input_manager_mut());
            }
        }
    }
}
