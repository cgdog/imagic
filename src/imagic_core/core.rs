use std::{cell::RefCell, rc::Rc};

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

#[derive(Default)]
pub struct Imagic {
    window: Window,
    context: ImagicContext,
    renderer: graphics::Renderer,
    pub app: Option<Rc<RefCell<Box<dyn ImagicAppTrait>>>>,
    is_inited: bool,
    option: Option<ImagicOption>,
}

impl ApplicationHandler for Imagic {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if !self.is_inited {
            // When first resumed, we create window.
            let option = self.option.get_or_insert(ImagicOption::default());
            self.window.init(&event_loop, option.window_size, option.window_title);
            pollster::block_on(self.context.graphics_context_mut().init(&self.window));

            self.context.init();

            let ui_renderer = UIRenderer::new(self.context.graphics_context().get_device(), 
                self.context.graphics_context().get_swapchian_format(), None, 1, &self.window.get());
            self.renderer.set_ui_renderer(Some(ui_renderer));

            let app_for_ui = self.app.as_mut().unwrap().clone();
            self.renderer.ui_renderer().set_ui_drawer(Some(Box::new(move |ctx|{
                app_for_ui.borrow_mut().on_render_ui(ctx);
            })));

            // TODO: study clone here
            self.app.clone().unwrap().borrow_mut().init(self);

            self.context.init_after_app(&self.window);

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
            &self.app,
        );
    }
}

impl Imagic {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn context(&self) -> &ImagicContext {
        &self.context
    }
    pub fn context_mut(&mut self) -> &mut ImagicContext {
        &mut self.context
    }

    pub fn run(&mut self, app: Rc<RefCell<Box<dyn ImagicAppTrait>>>) {
        info!("Imagic init() begin.");
        self.option = Some(app.borrow().get_imagic_option());
        self.app = Some(app);
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let _ = event_loop.run_app(self);
    }
}
