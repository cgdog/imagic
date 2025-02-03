use log::info;
use winit::{application::ApplicationHandler, event_loop::{ControlFlow, EventLoop}};

use crate::{graphics, ui::ui_renderer::UIRenderer, window::{Window, WindowSize}};
use super::{imagic_app::ImagicAppTrait, imagic_context::ImagicContext};

pub struct ImagicOption {
    // window logical size.
    pub window_size: WindowSize,
    pub window_title: &'static str,
}

impl Default for ImagicOption {
    fn default() -> Self {
        Self {
            window_size: Default::default(),
            window_title: "Imagic window"
        }
    }
}

impl ImagicOption {
    pub fn new(window_size: WindowSize, window_title: &'static str) -> Self {
        Self {
            window_size,
            window_title,
        }
    }
}

// #[derive(Default)]
pub struct Imagic {
    app: Box<dyn ImagicAppTrait>,
    option: ImagicOption,
    window: Window,
    context: ImagicContext,
    renderer: graphics::Renderer,
    is_inited: bool,
}

impl ApplicationHandler for Imagic {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if !self.is_inited {
            // When first resumed, we create window.
            self.window.init(&event_loop, self.option.window_size, self.option.window_title);
            pollster::block_on(self.context.graphics_context_mut().init(&self.window));

            self.context.init(*self.window.get_logical_size(), *self.window.get_physical_size());

            let ui_renderer = UIRenderer::new(self.context.graphics_context().get_device(), 
                self.context.graphics_context().get_swapchain_format(), None, 1, &self.window.get());
            self.renderer.set_ui_renderer(Some(ui_renderer));

            self.app.init(&mut self.context);

            // TODO: optimize lightmanager logic
            self.context.init_after_app();

            self.is_inited = true;
            info!("Imagic init() finished.");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        self.window.process_window_event(
            event_loop,
            event,
            &mut self.renderer,
            &mut self.context,
            &mut self.app,
        );
    }
}

impl Imagic {
    pub fn new(app: Box<dyn ImagicAppTrait>) -> Self {
        let option = app.get_imagic_option();
        Self {
            app,
            option: option,
            window: Default::default(),
            context: Default::default(),
            renderer: Default::default(),
            is_inited: false,
        }
    }

    pub fn context(&self) -> &ImagicContext {
        &self.context
    }
    pub fn context_mut(&mut self) -> &mut ImagicContext {
        &mut self.context
    }

    pub fn run(&mut self) {
        info!("Imagic init() begin.");
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let _ = event_loop.run_app(self);
    }
}
