use std::{cell::RefCell, rc::Rc};

use imagic::{prelude::*, window::WindowSize};
use log::info;

pub struct App {
    pub full_screen_item_index: usize,
    pub is_show_image: bool,
    window_size: WindowSize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            full_screen_item_index: INVALID_ID,
            is_show_image: true,
            window_size: WindowSize::new(500.0, 500.0),
        }
    }
}

impl ImagicAppTrait for App {
    fn on_update(&mut self, imagic_context: &mut ImagicContext) {
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
    
    fn init(&mut self, imagic: &mut Imagic) {
        let graphics_context = imagic.context().graphics_context();

        let bind_group_layout = bind_group_layout::create_default_bind_group_layout(graphics_context);
        let render_pipeline = render_pipeline::create_default_render_pipeline(graphics_context, &bind_group_layout);
        
        let texture = Texture::create_from_bytes(graphics_context,
            include_bytes!("./assets/lena.png"), wgpu::TextureFormat::Rgba8UnormSrgb);
        let texture_view = texture.get_texture_view();

        let texture_sampler = graphics_context.get_device().create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
    
        let bind_group = bind_group::create_default_bind_group(graphics_context,
            &bind_group_layout, &texture_view, &texture_sampler);
        
        {
            let context = imagic.context_mut();
            context.bind_group_layout_manager_mut().add_bind_group_layout(bind_group_layout);
            let pipeline_id = context.pipeline_manager_mut().add_render_pipeline(render_pipeline);
            let bind_group_id = context.bind_group_manager_mut().add_bind_group(bind_group);
    
            let mut render_item = RenderItem::new_thinly(pipeline_id, VertexOrIndexCount::VertexCount { vertex_count: 3, instance_count: 1 });
            render_item.set_item_bind_group_id(bind_group_id);
            let render_item_index = context.render_item_manager_mut().add_render_item(render_item);
            self.full_screen_item_index = render_item_index;
        }
    }
    
    fn get_imagic_option(& self) -> ImagicOption {
        ImagicOption::new(self.window_size, "Imagic Full Screen Demo")
    }
}

#[allow(dead_code)]
fn prepare_hdr_texture(graphics_context: &GraphicsContext) -> Texture {
    let mut hdr_loader = HDRLoader{};
    let cwd = std::env::current_dir().unwrap();
    let hdr_path = cwd.join("examples/assets/pbr/hdr/newport_loft.hdr");
    let hdr_texture = hdr_loader.load(hdr_path.to_str().unwrap(), graphics_context);
    hdr_texture
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut app: App = Default::default();

    let img = image::load_from_memory(include_bytes!("./assets/lena.png")).unwrap();
    let width = img.width() as f32;
    let height = img.height() as f32;
    app.window_size.set_width(width);
    app.window_size.set_height(height);

    let mut imagic = Imagic::new();
    let app: Rc<RefCell<Box<dyn ImagicAppTrait>>> = Rc::new(RefCell::new(Box::new(app)));
    imagic.run(app);
}