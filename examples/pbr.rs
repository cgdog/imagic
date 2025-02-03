use std::f32::consts;

use glam::Vec3;
use imagic::{prelude::*, window::WindowSize};
use log::info;

pub struct App {
    ibl_data: IBLData,
    skybox: Skybox,
    camera: usize,
    window_size: WindowSize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            ibl_data: IBLData::default(),
            skybox: Skybox::default(),
            camera: INVALID_ID,
            window_size: WindowSize::new(1280.0, 720.0),
        }
    }
}

impl App {
    fn set_pbr_ibl(&self, pbr_material: &mut Box<PBRMaterial>) {
        pbr_material.set_irradiance_cube_texture(self.ibl_data.irradiance_cube_texture);
        pbr_material.set_prefiltered_cube_texture(self.ibl_data.refelction_cube_texture);
        pbr_material.set_brdf_lut(self.ibl_data.brdf_lut);
    }

    fn init_ibl(&mut self, imagic_context: &mut ImagicContext) {
        let mut ibl_baker = IBLBaker::new(IBLBakerOptions {
            input_background_type: InputBackgroundType::HDRBytes(include_bytes!(
                "./assets/pbr/hdr/newport_loft.hdr"
            )),
            background_cube_map_size: 512,
            irradiance_cube_map_size: 32,
            // is_generate_brdf_lut: false,
            ..Default::default()
        });
        self.ibl_data = ibl_baker.bake(imagic_context);
        self.skybox
            // .init_with_cube_texture(imagic_context, self.ibl_data.refelction_cube_texture);
            .init_with_cube_texture(imagic_context, self.ibl_data.background_cube_texture);
        // .init_with_cube_texture(imagic_context, self.ibl_data.irradiance_cube_texture);
    }

    fn prepare_lights(&mut self, imagic_context: &mut ImagicContext) {
        let transform_manager = imagic_context.transform_manager();
        let point_light_0 = PointLight::new(
            Vec3::new(-10.0, 10.0, 10.0),
            Vec3::new(300.0, 300.0, 300.0),
            &mut transform_manager.borrow_mut(),
        );
        let point_light_1 = PointLight::new(
            Vec3::new(10.0, 10.0, 10.0),
            Vec3::new(300.0, 300.0, 300.0),
            &mut transform_manager.borrow_mut(),
        );
        let point_light_2 = PointLight::new(
            Vec3::new(-10.0, -10.0, 10.0),
            Vec3::new(300.0, 300.0, 300.0),
            &mut transform_manager.borrow_mut(),
        );
        let point_light_3 = PointLight::new(
            Vec3::new(10.0, -10.0, 10.0),
            Vec3::new(300.0, 300.0, 300.0),
            &mut transform_manager.borrow_mut(),
        );

        let light_manager = imagic_context.light_manager_mut();
        light_manager.add_point_light(point_light_0);
        light_manager.add_point_light(point_light_1);
        light_manager.add_point_light(point_light_2);
        light_manager.add_point_light(point_light_3);
    }

    fn prepare_material(
        &mut self,
        imagic_context: &mut ImagicContext,
        albedo_map_buffer: &[u8],
        normal_map_buffer: &[u8],
        metallic_map_buffer: &[u8],
        roughness_map_buffer: &[u8],
        ao_map_buffer: &[u8],
    ) -> ID {
        let graphics_context = imagic_context.graphics_context();
        let mut pbr_material = Box::new(PBRMaterial::new(
            Vec4::new(1.0, 1.0, 1.0, 1.0),
            1.0,
            1.0,
            1.0,
        ));
        let albedo_texture = Texture::create_from_bytes(
            graphics_context,
            albedo_map_buffer,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            false,
            1,
        );
        let normal_texture = Texture::create_from_bytes(
            graphics_context,
            normal_map_buffer,
            wgpu::TextureFormat::Rgba8Unorm,
            false,
            1,
        );
        let metallic_texture = Texture::create_from_bytes(
            graphics_context,
            metallic_map_buffer,
            wgpu::TextureFormat::Rgba8Unorm,
            false,
            1,
        );
        let roughness_texture = Texture::create_from_bytes(
            graphics_context,
            roughness_map_buffer,
            wgpu::TextureFormat::Rgba8Unorm,
            false,
            1,
        );
        let ao_texture = Texture::create_from_bytes(
            graphics_context,
            ao_map_buffer,
            wgpu::TextureFormat::Rgba8Unorm,
            false,
            1,
        );

        let texture_manager = imagic_context.texture_manager_mut();

        let albedo_texture = texture_manager.add_texture(albedo_texture);
        pbr_material.set_albedo_texture(albedo_texture);
        let normal_texture = texture_manager.add_texture(normal_texture);
        pbr_material.set_normal_texture(normal_texture);
        let metallic_texture = texture_manager.add_texture(metallic_texture);
        pbr_material.set_metallic_texture(metallic_texture);
        let roughness_texture = texture_manager.add_texture(roughness_texture);
        pbr_material.set_roughness_texture(roughness_texture);
        let ao_texture = texture_manager.add_texture(ao_texture);
        pbr_material.set_ao_texture(ao_texture);

        self.set_pbr_ibl(&mut pbr_material);
        let pbr_material_index = imagic_context.add_material(pbr_material);
        pbr_material_index
    }

    fn create_sphere(
        &mut self,
        imagic_context: &mut ImagicContext,
        albedo_map_buffer: &[u8],
        normal_map_buffer: &[u8],
        metallic_map_buffer: &[u8],
        roughness_map_buffer: &[u8],
        ao_map_buffer: &[u8],
        x_pos: f32,
    ) {
        let pbr_material_index = self.prepare_material(
            imagic_context,
            albedo_map_buffer,
            normal_map_buffer,
            metallic_map_buffer,
            roughness_map_buffer,
            ao_map_buffer,
        );
        let mut sphere = Sphere::new(1.0, 256, 256);
        sphere.init_with_transform(imagic_context, pbr_material_index, Transform {
            position: Vec3::new(x_pos, 0.0, 0.0),
            ..Default::default()
        });
    }

    fn create_spheres(&mut self, imagic_context: &mut ImagicContext) {
        self.create_sphere(
            imagic_context,
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/rustediron2_basecolor.png"),
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/rustediron2_normal.png"),
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/rustediron2_metallic.png"),
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/rustediron2_roughness.png"),
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/ao.png"),
            -4.0,
        );

        self.create_sphere(
            imagic_context,
            include_bytes!("./assets/pbr/gold/albedo.png"),
            include_bytes!("./assets/pbr/gold/normal.png"),
            include_bytes!("./assets/pbr/gold/metallic.png"),
            include_bytes!("./assets/pbr/gold/roughness.png"),
            include_bytes!("./assets/pbr/gold/ao.png"),
            -2.0,
        );

        self.create_sphere(
            imagic_context,
            include_bytes!("./assets/pbr/grass/albedo.png"),
            include_bytes!("./assets/pbr/grass/normal.png"),
            include_bytes!("./assets/pbr/grass/metallic.png"),
            include_bytes!("./assets/pbr/grass/roughness.png"),
            include_bytes!("./assets/pbr/grass/ao.png"),
            0.0,
        );

        self.create_sphere(
            imagic_context,
            include_bytes!("./assets/pbr/plastic/albedo.png"),
            include_bytes!("./assets/pbr/plastic/normal.png"),
            include_bytes!("./assets/pbr/plastic/metallic.png"),
            include_bytes!("./assets/pbr/plastic/roughness.png"),
            include_bytes!("./assets/pbr/plastic/ao.png"),
            2.0,
        );

        self.create_sphere(
            imagic_context,
            include_bytes!("./assets/pbr/wall/albedo.png"),
            include_bytes!("./assets/pbr/wall/normal.png"),
            include_bytes!("./assets/pbr/wall/metallic.png"),
            include_bytes!("./assets/pbr/wall/roughness.png"),
            include_bytes!("./assets/pbr/wall/ao.png"),
            4.0,
        );
    }
}

impl ImagicAppTrait for App {
    fn init(&mut self, imagic_context: &mut ImagicContext) {
        self.prepare_lights(imagic_context);
        self.init_ibl(imagic_context);

        self.camera = Camera::new(
            Vec3::new(0.0, 1.0, 6.0),
            consts::FRAC_PI_4,
            self.window_size.get_aspect() as f32,
            0.1,
            100.0,
            Some(CameraControllerOptions::default()),
            imagic_context,
        );

        self.create_spheres(imagic_context);
    }

    fn on_update(&mut self, _imagic_context: &mut ImagicContext) {}

    fn get_imagic_option(&self) -> ImagicOption {
        ImagicOption::new(self.window_size, "pbr Demo")
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("pbr main.");
    let mut imagic = Imagic::new(Box::new(App::default()));
    imagic.run();
}
