use crate::{
    camera::{Camera, Layer, LayerMask},
    math::{Vec3, Vec4},
    model::{Cube, Plane},
    prelude::{BRDFIntegralMaterial, ImagicContext, RenderTexture, RenderTexture2D, INVALID_ID},
    scene::SceneObject,
    types::{ID, RR},
};

use super::{EquirectToCubeConverter, IrradianceMapGenerator};

pub enum InputEquirect {
    Path(&'static str),
    Bytes(&'static [u8]),
    None,
}

pub struct IBLBakerOptions {
    pub input_equirect_image: InputEquirect,
    pub is_flip_y: bool,
    pub is_bake_irradiance: bool,
    pub is_bake_reflection: bool,
    pub is_generate_brdf_lut: bool,

    pub background_cube_map_size: u32,
    pub irradiance_cube_map_size: u32,
    pub reflection_cube_map_size: u32,
    pub brdf_lut_size: u32,

    pub rt_format: wgpu::TextureFormat,
}

impl Default for IBLBakerOptions {
    fn default() -> Self {
        Self {
            input_equirect_image: InputEquirect::None,
            is_flip_y: false,
            is_bake_irradiance: true,
            is_bake_reflection: true,
            is_generate_brdf_lut: true,

            background_cube_map_size: 512,
            irradiance_cube_map_size: 32,
            reflection_cube_map_size: 128,
            brdf_lut_size: 512,

            rt_format: wgpu::TextureFormat::Rgba32Float,
        }
    }
}

pub struct IBLData {
    pub background_cube_texture: ID,
    pub irradiance_cube_texture: ID,
    pub refelction_cube_texture: ID,
    pub brdf_lut: ID,
}

impl Default for IBLData {
    fn default() -> Self {
        Self {
            background_cube_texture: INVALID_ID,
            irradiance_cube_texture: INVALID_ID,
            refelction_cube_texture: INVALID_ID,
            brdf_lut: INVALID_ID,
        }
    }
}

pub struct IBLBaker {
    options: IBLBakerOptions,
    background_cube_texture: ID,
    irradiance_cube_texture: ID,
    refelction_cube_texture: ID,
    brdf_lut: ID,
}

impl Default for IBLBaker {
    fn default() -> Self {
        Self {
            options: Default::default(),
            background_cube_texture: INVALID_ID,
            irradiance_cube_texture: INVALID_ID,
            refelction_cube_texture: INVALID_ID,
            brdf_lut: INVALID_ID,
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
        self.generate_irradiance_cube_texture(
            imagic_context,
            &mut camera.borrow_mut(),
            &mut cube,
            // &sync_buffer2,
        );
        // sync_buffer2.receive(imagic_context.graphics_context());

        if self.options.is_generate_brdf_lut {
            self.generate_brdf_lut(imagic_context);
        }
        IBLData {
            background_cube_texture: self.background_cube_texture,
            irradiance_cube_texture: self.irradiance_cube_texture,
            refelction_cube_texture: self.refelction_cube_texture,
            brdf_lut: self.brdf_lut,
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
        match self.options.input_equirect_image {
            InputEquirect::Path(image_path) => {
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
            InputEquirect::Bytes(image_bytes) => {
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
            self.background_cube_texture,
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

    pub fn get_background_cube_texture(&self) -> ID {
        self.background_cube_texture
    }

    pub fn get_irradiance_cube_texture(&self) -> ID {
        self.irradiance_cube_texture
    }

    pub fn get_reflection_cube_texture(&self) -> ID {
        self.refelction_cube_texture
    }

    /// Generate BRDF LUT texture.
    pub fn generate_brdf_lut(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let camera = Self::create_camera(imagic_context);
        let material = Box::new(BRDFIntegralMaterial::new());
        let material_index = imagic_context.add_material(material);
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
        self.brdf_lut = rt.get_color_attachment_id();
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
        self.brdf_lut
    }
}
