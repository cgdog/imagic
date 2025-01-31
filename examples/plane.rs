use std::f32::consts;

use changeable::Changeable;
use imagic::{prelude::*, window::WindowSize};
use log::info;

pub struct App {
    plane: Plane,
    camera: usize,
    window_size: WindowSize,
    is_show_image: Changeable<bool>,
    show_brdf_lut: Changeable<bool>,
    // TODO: optimize to use only one unlit material to show both texture.
    lena_unlit: ID,
    brdf_lut_unlit: ID,
}

impl Default for App {
    fn default() -> Self {
        Self {
            plane: Plane::default(),
            camera: INVALID_ID,
            window_size: WindowSize::new(500.0, 500.0),
            is_show_image: Changeable::new(true),
            show_brdf_lut: Changeable::new(true),
            lena_unlit: INVALID_ID,
            brdf_lut_unlit: INVALID_ID,
        }
    }
}

impl App {
    fn prepare_albedo(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let albedo_texture = Texture::create_from_bytes(
            imagic_context.graphics_context(),
            include_bytes!("./assets/lena.png"),
            wgpu::TextureFormat::Rgba8UnormSrgb,
            true,
            1,
        );
        let albedo_texture_index = imagic_context
            .texture_manager_mut()
            .add_texture(albedo_texture);
        albedo_texture_index
    }

    fn prepare_material(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let mut lena_unlit_material = Box::new(UnlitMaterial::new());
        let albedo_index = self.prepare_albedo(imagic_context);
        lena_unlit_material.set_albedo_map(albedo_index);
        self.lena_unlit = imagic_context.add_material(lena_unlit_material);

        let brdf_lut = self.generate_brdf_lut(imagic_context);
        let mut brdf_lut_material = Box::new(UnlitMaterial::new());
        brdf_lut_material.set_albedo_map(brdf_lut);
        self.brdf_lut_unlit = imagic_context.add_material(brdf_lut_material);
        self.brdf_lut_unlit
    }

    fn generate_brdf_lut(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let mut ibl_baker = IBLBaker::new(IBLBakerOptions {
            brdf_lut_size: 512,
            ..Default::default()
        });
        let brdf_lut = ibl_baker.generate_brdf_lut(imagic_context);
        brdf_lut
        // debug brdflut integration
        // let unlit_material = Box::new(BRDFIntegralMaterial::new());
    }
}

impl ImagicAppTrait for App {
    fn init(&mut self, imagic_context: &mut ImagicContext) {
        self.camera = Camera::new(
            Vec3::new(0.0, 0.0, 5.0),
            consts::FRAC_PI_4,
            self.window_size.get_aspect(),
            1.0,
            100.0,
            None,
            imagic_context,
        );

        let material_index = self.prepare_material(imagic_context);
        self.plane.init(imagic_context, material_index);
    }

    fn on_update(&mut self, imagic_context: &mut ImagicContext) {
        if self.is_show_image.is_changed() {
            self.is_show_image.reset();
            let render_item = imagic_context
                .render_item_manager_mut()
                .get_render_item_mut(self.plane.render_item_id());
            render_item.is_visible = *self.is_show_image;
        }

        if self.show_brdf_lut.is_changed() {
            self.show_brdf_lut.reset();
            // TODO: optimize to use only one material.
            let unlit_material_index = if *self.show_brdf_lut {
                self.brdf_lut_unlit
            } else {
                self.lena_unlit
            };

            imagic_context
                .pipeline_manager()
                .borrow_mut()
                .remove_render_pipeline(self.plane.render_item_id());

            imagic_context
                .render_item_manager_mut()
                .get_render_item_mut(self.plane.render_item_id())
                .set_material_id(unlit_material_index);
        }
    }

    fn on_render_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Imagic - plane")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .default_size([200.0, 10.0])
            .show(&ctx, |ui| {
                if *self.show_brdf_lut {
                    ui.label("Below is the brdf integration map, a HDR texture and upside down.");
                } else {
                    ui.label("Below is lena image.");
                }

                if *self.show_brdf_lut {
                    if ui.button("Show lena").clicked() {
                        *self.show_brdf_lut = !*self.show_brdf_lut;
                        self.show_brdf_lut.set();
                    }
                } else {
                    if ui.button("Show brdf lut").clicked() {
                        *self.show_brdf_lut = !*self.show_brdf_lut;
                        self.show_brdf_lut.set();
                    }
                }

                if *self.is_show_image {
                    if ui.button("Hide image").clicked() {
                        info!("Hide image.");
                        *self.is_show_image = !*self.is_show_image;
                        self.is_show_image.set();
                    }
                } else {
                    if ui.button("Show image").clicked() {
                        info!("Show image.");
                        *self.is_show_image = !*self.is_show_image;
                        self.is_show_image.set();
                    }
                }

                // ui.separator();
                // ui.label("This simple demo powered by wgpu renders full screen with a big triangle and a texture without Vertex buffer");
            });
    }

    fn get_imagic_option(&self) -> ImagicOption {
        ImagicOption::new(self.window_size, "plane Demo")
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("plane example main.");
    let mut imagic = Imagic::new(Box::new(App::default()));
    imagic.run();
}
