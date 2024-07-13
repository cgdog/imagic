use std::{cell::RefCell, rc::Rc};

use log::info;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::{graphics, ui::ui_renderer::UIRenderer, window::Window};
use super::{imagic_app::ImagicAppTrait, imagic_context::ImagicContext};

pub struct ImagicOption {
    pub window_width: f64,
    pub window_height: f64,
    pub window_title: &'static str,
}

impl ImagicOption {
    pub fn new(window_width: f64, window_height: f64, window_title: &'static str) -> Self {
        Self {
            window_width,
            window_height,
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

    pub fn init(&mut self, imagic_option: ImagicOption) -> EventLoop<()> {
        info!("Imagic init() begin.");
        let event_loop = EventLoop::new().unwrap();
        self.window.init(&event_loop, imagic_option.window_width, imagic_option.window_height, imagic_option.window_title);
        pollster::block_on(self.context.graphics_context_mut().init(&self.window));

        self.context.init();

        let ui_renderer = UIRenderer::new(self.context.graphics_context().get_device(), 
            self.context.graphics_context().get_swapchian_format(), None, 1, &self.window.get());
        self.renderer.set_ui_renderer(Some(ui_renderer));

        self.is_inited = true;
        info!("Imagic init() finished.");

        event_loop
    }

    fn init_after_app(&mut self) {
        self.context.init_after_app();
    }

    pub fn run(&mut self, event_loop: EventLoop<()>, app: Rc<RefCell<Box<dyn ImagicAppTrait>>>) {

        self.init_after_app();

        let app_for_ui = app.clone();
        self.app = Some(app);
        self.renderer.ui_renderer().set_ui_drawer(Some(Box::new(move |ctx|{
            app_for_ui.borrow_mut().on_render_ui(ctx);
        })));

        let _ = event_loop.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);
            self.window.process_window_event(
                event,
                elwt,
                &mut self.renderer,
                &mut self.context,
                &self.app,
            );
        });
    }
}
