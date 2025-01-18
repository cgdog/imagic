use std::{cell::RefCell, f32::consts, rc::Rc};

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

    fn _prepare_hdr_texture(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let mut hdr_loader = HDRLoader{};
        let cwd = std::env::current_dir().unwrap();
        let hdr_path = cwd.join("examples/assets/pbr/hdr/newport_loft.hdr");
        let hdr_texture = hdr_loader.load(hdr_path.to_str().unwrap(), imagic_context.graphics_context());
        let hdr_texture_index = imagic_context.texture_manager_mut().add_texture(hdr_texture);
        hdr_texture_index
    }

    fn _prepare_albedo(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let albedo_texture = Texture::create_from_bytes(imagic_context.graphics_context(),
            include_bytes!("./assets/lena.png"), wgpu::TextureFormat::Rgba8UnormSrgb);
        let albedo_texture_index = imagic_context.texture_manager_mut().add_texture(albedo_texture);
        albedo_texture_index
    }

    fn prepare_material(&mut self, imagic: &mut Imagic) -> ID {
        let mut unlit_material = Box::new(UnlitMaterial::new());
        let albedo_index = self._prepare_albedo(imagic.context_mut());
        unlit_material.set_albedo_map(albedo_index);

        let material_index = imagic.context_mut().material_manager_mut().add_material(unlit_material);
        material_index
    }
    
    pub fn run(self) {
        let mut imagic = Imagic::new();
        let app: Rc<RefCell<Box<dyn ImagicAppTrait>>> = Rc::new(RefCell::new(Box::new(self)));
        imagic.run(app);
    }
}

impl ImagicAppTrait for App {
    fn init(&mut self, imagic: &mut Imagic) {
        let imagic_context = imagic.context_mut();
        // self.prepare_lights(imagic_context);
        self.camera = Camera::new(Vec3::new(0.0, 0.0, 5.0), consts::FRAC_PI_4
            , self.window_size.get_aspect(), 1.0, 100.0, None, imagic_context);

        let material_index = self.prepare_material(imagic);
        self.plane.init(imagic, material_index);
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
    let app: App = Default::default();
    app.run();
}