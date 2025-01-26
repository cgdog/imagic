use std::f32::consts;

use log::info;
use imagic::{prelude::*, window::WindowSize};

pub struct App {
    plane: Plane,
    camera: usize,
    window_size: WindowSize,
    pub is_show_image: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            plane: Plane::default(),
            camera: INVALID_ID,
            window_size: WindowSize::new(500.0, 500.0),
            is_show_image: true,
        }
    }
}

impl App {

    fn prepare_albedo(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let albedo_texture = Texture::create_from_bytes(imagic_context.graphics_context(),
            include_bytes!("./assets/lena.png"), wgpu::TextureFormat::Rgba8UnormSrgb, true);
        let albedo_texture_index = imagic_context.texture_manager_mut().add_texture(albedo_texture);
        albedo_texture_index
    }

    fn prepare_material(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let mut unlit_material = Box::new(UnlitMaterial::new());
        let albedo_index = self.prepare_albedo(imagic_context);
        unlit_material.set_albedo_map(albedo_index);

        let material_index = imagic_context.add_material(unlit_material);
        material_index
    }
}

impl ImagicAppTrait for App {
    fn init(&mut self, imagic_context: &mut ImagicContext) {
        self.camera = Camera::new(Vec3::new(0.0, 0.0, 5.0), consts::FRAC_PI_4
            , self.window_size.get_aspect(), 1.0, 100.0, None, imagic_context);

        let material_index = self.prepare_material(imagic_context);
        self.plane.init(imagic_context, material_index);
    }

    fn on_update(&mut self, _imagic_context: &mut ImagicContext) {
        let render_item = _imagic_context.render_item_manager_mut().get_render_item_mut(self.plane.render_item_id());
        render_item.is_visible = self.is_show_image;
    }

    fn on_render_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Imagic - plane")
        .resizable(true)
        .vscroll(true)
        .default_open(false)
        .default_size([200.0, 10.0])
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

    fn get_imagic_option(& self) -> ImagicOption {
        ImagicOption::new(self.window_size, "plane Demo")
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("plane example main.");
    let mut imagic = Imagic::new(Box::new(App::default()));
    imagic.run();
}