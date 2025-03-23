use wgpu::TextureView;
use winit::window::Window as WindowWinit;

use crate::{ecs::world::World, prelude::{ImagicAppTrait, ImagicContext, UIRenderer}};

pub struct Renderer {
    ui_renderer: Option<UIRenderer>,
}

impl Default for Renderer {
    fn default() -> Self {
        Self { ui_renderer: None }
    }
}

impl Renderer {
    pub fn set_ui_renderer(&mut self, ui_renderer: Option<UIRenderer>) {
        self.ui_renderer = ui_renderer;
    }

    pub fn ui_renderer(&mut self) -> &mut UIRenderer {
        self.ui_renderer.as_mut().expect("ui_renderer is None.")
    }

    pub fn render(
        &mut self,
        world: &mut World,
        window: &WindowWinit,
        app: &mut Box<dyn ImagicAppTrait>,
    ) {
        let surface_texture = world.context_mut()
            .graphics_context()
            .get_surface()
            .get_current_texture();
        let surface_texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let cameras = world.context().camera_manager().get_cameras();
        for (index, camera) in cameras.iter().enumerate() {
            if camera.borrow().draw_manually {
                continue;
            }
            // self.render_with_camera(context, camera, index, &surface_texture_view);
            camera.borrow().render_to_attachments(world.context(), &surface_texture_view, index, None, None, None);
        }

        self.render_ui(world.context_mut(), window, &surface_texture_view, app);
        surface_texture.present();
    }

    pub fn render_ui(
        &mut self,
        context: &mut ImagicContext,
        window: &WindowWinit,
        surface_texture_view: &TextureView,
        app: &mut Box<dyn ImagicAppTrait>,
    ) {
        self.ui_renderer().draw(
            context.graphics_context(),
            &window,
            surface_texture_view,
            app,
        );
    }
}
