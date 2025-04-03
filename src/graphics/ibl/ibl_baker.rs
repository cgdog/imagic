use glam::Vec4;

use crate::{
    asset::asset::Handle, camera::{Camera, Layer, LayerMask}, math::Vec3, model::{Cube, Plane}, prelude::{
        BRDFIntegralMaterial, ComputeShader, CubeRenderTexture, ImagicContext, Material, RenderTexture, RenderTexture2D, SrgbCubeToLinearMaterial, Texture
    }, scene::SceneObject, types::RR
};

use super::{
    CubeMipmapsGenerator, CubeTexturePrefilter, EquirectToCubeConverter, IrradianceMapGenerator,
    MipmapGeneratorType,
};

pub enum InputBackgroundType {
    Path(&'static str),
    HDRBytes(&'static [u8]),
    LDRBytes([&'static [u8]; 6]),
    None,
}

pub struct IBLBakerOptions {
    pub input_background_type: InputBackgroundType,
    pub is_flip_y: bool,
    pub is_bake_irradiance: bool,
    pub is_bake_reflection: bool,
    pub is_generate_brdf_lut: bool,

    pub background_cube_map_size: u32,
    pub irradiance_cube_map_size: u32,
    pub reflection_cube_map_size: u32,
    pub reflection_cube_map_mipmap_level_count: u32,
    pub brdf_lut_size: u32,

    pub rt_format: wgpu::TextureFormat,
    pub mipmap_generator_type: MipmapGeneratorType,
}

impl Default for IBLBakerOptions {
    fn default() -> Self {
        Self {
            input_background_type: InputBackgroundType::None,
            is_flip_y: false,
            is_bake_irradiance: true,
            is_bake_reflection: true,
            is_generate_brdf_lut: true,

            background_cube_map_size: 512,
            irradiance_cube_map_size: 32,
            reflection_cube_map_size: 128,
            reflection_cube_map_mipmap_level_count: 5,
            brdf_lut_size: 512,

            rt_format: wgpu::TextureFormat::Rgba32Float,
            mipmap_generator_type: MipmapGeneratorType::GaussianFilter4x4,
        }
    }
}

pub struct IBLData {
    pub background_cube_texture: Handle<Texture>,
    pub irradiance_cube_texture: Handle<Texture>,
    pub refelction_cube_texture: Handle<Texture>,
    pub brdf_lut: Handle<Texture>,
}

impl Default for IBLData {
    fn default() -> Self {
        Self {
            background_cube_texture: Handle::INVALID,
            irradiance_cube_texture: Handle::INVALID,
            refelction_cube_texture: Handle::INVALID,
            brdf_lut: Handle::INVALID,
        }
    }
}

pub struct IBLBaker {
    options: IBLBakerOptions,
    background_cube_texture: Handle<Texture>,
    irradiance_cube_texture: Handle<Texture>,
    refelction_cube_texture: Handle<Texture>,
    brdf_lut: Handle<Texture>,
}

impl Default for IBLBaker {
    fn default() -> Self {
        Self {
            options: Default::default(),
            background_cube_texture: Handle::INVALID,
            irradiance_cube_texture: Handle::INVALID,
            refelction_cube_texture: Handle::INVALID,
            brdf_lut: Handle::INVALID,
        }
    }
}

impl IBLBaker {
    pub fn new(options: IBLBakerOptions) -> Self {
        Self {
            options,
            ..Default::default()
        }
    }

    pub fn bake(&mut self, imagic_context: &mut ImagicContext) -> IBLData {
        let mut cube = Cube::new(1.0, 1.0, 1.0, 1, 1, 1);
        let camera = Self::create_camera(imagic_context);
        // let sync_buffer = SyncBuffer::new(imagic_context.graphics_context());
        self.generate_background_cube_texture(
            imagic_context,
            &mut camera.borrow_mut(),
            &mut cube,
            // &sync_buffer,
        );
        // sync_buffer.receive(imagic_context.graphics_context());

        // let sync_buffer2 = SyncBuffer::new(imagic_context.graphics_context());
        if self.options.is_bake_irradiance {
            self.generate_irradiance_cube_texture(
                imagic_context,
                &mut camera.borrow_mut(),
                &mut cube,
                // &sync_buffer2,
            );
        }
        // sync_buffer2.receive(imagic_context.graphics_context());

        if self.options.is_bake_reflection {
            self.generate_prefiltered_cube_texture(
                imagic_context,
                &mut camera.borrow_mut(),
                &mut cube,
            );
        }

        if self.options.is_generate_brdf_lut {
            // Note: this function generate a full screen plane with layer of RenderTarget.
            self.generate_brdf_lut(imagic_context);
        }

        // TODO: Delete this cube when baking finished.
        imagic_context
            .render_item_manager_mut()
            .get_render_item_mut(cube.render_item_id())
            .is_visible = false;

        IBLData {
            background_cube_texture: self.background_cube_texture.clone(),
            irradiance_cube_texture: self.irradiance_cube_texture.clone(),
            refelction_cube_texture: self.refelction_cube_texture.clone(),
            brdf_lut: self.brdf_lut.clone(),
        }
    }

    fn generate_background_cube_texture(
        &mut self,
        imagic_context: &mut ImagicContext,
        camera: &mut Camera,
        cube: &mut Cube,
        // sync_buffer: &SyncBuffer,
    ) {
        let background_cube_texture_generator =
            EquirectToCubeConverter::new(self.options.is_flip_y);
        match self.options.input_background_type {
            InputBackgroundType::Path(image_path) => {
                self.background_cube_texture = background_cube_texture_generator.convert(
                    image_path,
                    imagic_context,
                    self.options.background_cube_map_size,
                    self.options.rt_format,
                    camera,
                    cube,
                    // sync_buffer
                );
            }
            InputBackgroundType::HDRBytes(image_bytes) => {
                self.background_cube_texture = background_cube_texture_generator.convert_by_bytes(
                    &image_bytes,
                    imagic_context,
                    self.options.background_cube_map_size,
                    self.options.rt_format,
                    camera,
                    cube,
                    // sync_buffer
                );
            }
            InputBackgroundType::LDRBytes(ldr_bytes) => {
                let cube_texture = Texture::create_cube_texture_from_bytes(
                    imagic_context.graphics_context(),
                    ldr_bytes,
                    wgpu::TextureFormat::Rgba8UnormSrgb,
                    1,
                );
                let size = cube_texture.get_size();
                let ldr_background_cube_texture = imagic_context
                    .asset_manager_mut()
                    .add(cube_texture);
                let srgb_cube_to_linear_material =
                    SrgbCubeToLinearMaterial::new(ldr_background_cube_texture);
                let srgb_cube_to_linear_material_id =
                    imagic_context.add_material(Box::new(srgb_cube_to_linear_material) as Material);
                cube.init(imagic_context, srgb_cube_to_linear_material_id);
                cube.set_layer(
                    Layer::RenderTarget,
                    imagic_context.render_item_manager_mut(),
                );

                let hdr_background_rt = CubeRenderTexture::new(
                    imagic_context,
                    wgpu::TextureFormat::Rgba32Float,
                    size.width,
                    size.height,
                    1,
                );
                // info!("wgpu::TextureFormat::Rgba32Float: {:?}", wgpu::TextureFormat::Rgba32Float);
                self.background_cube_texture = hdr_background_rt.get_color_attachment_handle();
                camera.set_render_texture(Box::new(hdr_background_rt));
                camera.set_viewport(Vec4::new(0.0, 0.0, 1.0, 0.0));
                camera.set_logical_viewport(Vec4::new(
                    0.0,
                    0.0,
                    size.width as f32,
                    size.height as f32,
                ));
                camera.set_physical_viewport(Vec4::new(
                    0.0,
                    0.0,
                    size.width as f32,
                    size.height as f32,
                ));
                camera.render(imagic_context, None);
            }
            _ => (),
        }
    }

    fn generate_irradiance_cube_texture(
        &mut self,
        imagic_context: &mut ImagicContext,
        camera: &mut Camera,
        cube: &mut Cube,
        // sync_buffer: &SyncBuffer,
    ) {
        let irradiance_map_generator = IrradianceMapGenerator::new();
        self.irradiance_cube_texture = irradiance_map_generator.generate(
            imagic_context,
            self.background_cube_texture.clone(),
            self.options.irradiance_cube_map_size,
            cube.render_item_id(),
            self.options.rt_format,
            camera,
            // sync_buffer
        );
    }

    fn create_camera(imagic_context: &mut ImagicContext) -> RR<Camera> {
        let camera_id = Camera::new(
            Vec3::ZERO,
            std::f32::consts::FRAC_PI_2,
            1.0,
            0.1,
            10.0,
            None,
            imagic_context,
        );

        let camera = imagic_context.camera_manager_mut().get_camera(camera_id);
        camera.borrow_mut().set_clear_color(Vec4::ZERO);
        camera.borrow_mut().layer_mask = LayerMask::new(Layer::RenderTarget.into());
        camera.borrow_mut().draw_manually = true;
        // Note here: avoid viewport is automatically resized when window resized.
        camera.borrow_mut().is_viewport_auto_resizeable = false;
        camera
    }

    pub fn get_background_cube_texture(&self) -> Handle<Texture> {
        self.background_cube_texture.clone()
    }

    pub fn get_irradiance_cube_texture(&self) -> Handle<Texture> {
        self.irradiance_cube_texture.clone()
    }

    pub fn get_reflection_cube_texture(&self) -> Handle<Texture> {
        self.refelction_cube_texture.clone()
    }

    /// Generate BRDF LUT texture.
    pub fn generate_brdf_lut(&mut self, imagic_context: &mut ImagicContext) -> Handle<Texture> {
        let camera = Self::create_camera(imagic_context);
        let material = Box::new(BRDFIntegralMaterial::new());
        let material_index = imagic_context.add_material(material as Material);
        let mut plane = Plane::default();
        let rt_size = self.options.brdf_lut_size;
        plane.init(imagic_context, material_index);
        plane.set_layer(
            Layer::RenderTarget,
            imagic_context.render_item_manager_mut(),
        );
        let rt = RenderTexture2D::new(
            imagic_context,
            // TODO: try to use Rgba16Float.
            wgpu::TextureFormat::Rgba32Float,
            rt_size,
            rt_size,
        );
        self.brdf_lut = rt.get_color_attachment_handle();
        let rt_size = rt_size as f32;
        camera
            .borrow_mut()
            .set_viewport(Vec4::new(0.0, 0.0, 1.0, 1.0));
        camera
            .borrow_mut()
            .set_logical_viewport(Vec4::new(0.0, 0.0, rt_size, rt_size));
        camera
            .borrow_mut()
            .set_physical_viewport(Vec4::new(0.0, 0.0, rt_size, rt_size));
        camera.borrow_mut().set_render_texture(Box::new(rt));
        camera.borrow_mut().render(imagic_context, None);

        // TODO: Delete this plane when baking finished.
        imagic_context
            .render_item_manager_mut()
            .get_render_item_mut(plane.render_item_id())
            .is_visible = false;
        self.brdf_lut.clone()
    }

    fn generate_prefiltered_cube_texture(
        &mut self,
        imagic_context: &mut ImagicContext,
        camera: &mut Camera,
        cube: &mut Cube,
        // sync_buffer: &SyncBuffer,
    ) {
        // generate mipmaps with compute shader, like openGL's glGenerateMipmap.
        let mut cube_mipmaps_generator = CubeMipmapsGenerator::new(
            self.background_cube_texture.clone(),
            self.options.background_cube_map_size,
            // note: Rgba8UnormSrgb format does not support StorageBinding
            wgpu::TextureFormat::Rgba32Float,
            // MipmapGeneratorType::GaussianFilter4x4,
            // MipmapGeneratorType::BilinearFilter,
            self.options.mipmap_generator_type,
        );
        cube_mipmaps_generator.execute(imagic_context);
        let cube_map_with_mipmaps = cube_mipmaps_generator.get_cube_with_mipmap();
        // self.refelction_cube_texture = cube_map_with_mipmaps;

        let mut cube_texture_prefilter = CubeTexturePrefilter::new(
            cube_map_with_mipmaps,
            self.options.reflection_cube_map_mipmap_level_count,
            self.options.reflection_cube_map_size,
        );
        self.refelction_cube_texture =
            cube_texture_prefilter.prefilter(imagic_context, camera, cube, self.options.rt_format);
    }
}
