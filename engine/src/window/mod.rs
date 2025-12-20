use std::sync::Arc;
use winit::{dpi::{LogicalSize, Size}, event_loop::ActiveEventLoop, window::Window as WindowWinit};

pub mod window_size;
pub(crate) mod window_input_processor;

pub use window_size::*;


pub struct Window {
    window: Arc<WindowWinit>,
    logical_size: WindowSize,
    physical_size: WindowSize,
    /// see https://docs.rs/dpi/0.1.1/dpi/index.html
    pub dpi: f64,
}

impl Window {
    pub fn new(
        event_loop: &ActiveEventLoop,
        window_size: WindowSize,
        window_title: &'static str,
    ) -> Self {
        let (logical_width, logical_height) = window_size.get();
        let logical_size = LogicalSize::new(logical_width as f64, logical_height as f64);
        let window_attributes = WindowWinit::default_attributes()
            .with_title(window_title)
            .with_inner_size(Size::Logical(logical_size));

        let window = event_loop.create_window(window_attributes).unwrap();
        let dpi = window.scale_factor();
        // info!("window dpi: {0}", self.dpi);
        let window = Arc::new(window);

        let physical_size = window.inner_size();
        Self {
            window,
            logical_size: window_size,
            physical_size: WindowSize::new(physical_size.width as f32, physical_size.height as f32),
            dpi,
        }
    }

    pub(crate) fn get(&self) -> Arc<WindowWinit> {
        self.window.clone()
    }

    pub(crate) fn get_ref(&self) -> &WindowWinit {
        &self.window
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn force_get_scale_factor(&mut self) -> f64 {
        self.dpi = self.window.scale_factor();
        self.dpi
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
}