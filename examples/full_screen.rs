use std::{cell::RefCell, rc::Rc};

use imagic::prelude::*;
use log::info;

pub struct App {
    pub full_screen_item_index: usize,
    pub is_show_image: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            full_screen_item_index: usize::MAX,
            is_show_image: true,
        }
    }
}

impl ImagicAppTrait for App {
    fn on_update(&mut self, imagic_context: &mut ImagicContext, _ui_renderer: &mut UIRenderer) {
        let render_item = imagic_context.render_item_manager_mut().get_render_item_mut(self.full_screen_item_index);
        render_item.is_visible = self.is_show_image;
        // You can set parameters about UIRenderer, for example, scale the UIs.
        // _ui_renderer.set_ui_scale_factor(2.0);
    }

    fn on_render_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Imagic - full screen")
        .resizable(true)
        .vscroll(true)
        .default_open(false)
        .show(&ctx, |ui| {
            ui.label("You can drag the UI window title to move the UI window!");
            if self.is_show_image {
                if ui.button("Hide image").clicked() {
                    info!("Hide image.");
                    self.is_show_image = !self.is_show_image;
                }
            } else {
                if ui.button("Show image").clicked() {
                    info!("Show image.");
                    self.is_show_image = !self.is_show_image;
                }
            }

            ui.separator();
            ui.label("This simple demo powered by wgpu renders full screen with a big triangle and a texture without Vertex buffer");
        });
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut app: App = Default::default();

    let img = image::load_from_memory(include_bytes!("./assets/lena.png")).unwrap();
    // let img = image::io::Reader::open("./assets/lena.png").expect("failed to load img").decode().unwrap();
    let width = img.width() as f64;
    let height = img.height() as f64;
    let img_rgba = img.to_rgba8();

    let mut imagic = Imagic::new();
    let event_loop = imagic.init(ImagicOption::new(width, height, "Imagic Full Screen Demo"));

    let graphics_context = imagic.context().graphics_context();

    let bind_group_layout = bind_group_layout::create_default_bind_group_layout(graphics_context);
    let render_pipeline = render_pipeline::create_default_render_pipeline(graphics_context, &bind_group_layout);
    
    use image::GenericImageView;
    let dimensions = img.dimensions();
    let texture = Texture::create(graphics_context, dimensions.0, dimensions.1, wgpu::TextureFormat::Rgba8UnormSrgb);
    texture.fill_content(graphics_context, &img_rgba);

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let texture_sampler = graphics_context.get_device().create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let bind_group = bind_group::create_default_bind_group(imagic.context().graphics_context(),
        &bind_group_layout, &texture_view, &texture_sampler);
    
    {
        let context = imagic.context_mut();
        context.bind_group_layout_manager_mut().add_bind_group_layout(bind_group_layout);
        let pipeline_id = context.pipeline_manager_mut().add_render_pipeline(render_pipeline);
        let bind_group_id = context.bind_group_manager_mut().add_bind_group(bind_group);

        let mut render_item = RenderItem::new_thinly(pipeline_id, VertexOrIndexCount::VertexCount { vertex_count: 3, instance_count: 1 });
        render_item.set_bind_groups(vec![bind_group_id]);
        let render_item_index = context.render_item_manager_mut().add_render_item(render_item);
        app.full_screen_item_index = render_item_index;
    }
    
    let app: Rc<RefCell<Box<dyn ImagicAppTrait>>> = Rc::new(RefCell::new(Box::new(app)));
    imagic.run(event_loop, app);
}