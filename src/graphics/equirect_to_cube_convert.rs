use core::f32;

use crate::{
    camera::{Camera, Layer, LayerMask},
    math::{Vec3, Vec4},
    model::Cube,
    prelude::{EquirectangularToCubeMaterial, HDRLoader, HDRLoaderOptions, ImagicContext, MaterialTrait},
    scene::SceneObject,
    types::{ID, RR},
};

use super::{texture::Texture, CubeRenderTexture, RenderTexture};


pub struct EquirectToCubeConverter {
    is_flip_y: bool,
}

impl Default for EquirectToCubeConverter {
    fn default() -> Self {
        Self {
            is_flip_y: false,
        }
    }
}

impl EquirectToCubeConverter {
    pub fn new(is_flip_y: bool) -> Self {
        Self {
            is_flip_y,
        }
    }

    /// Convert an equirectangular map to a CubeMap.
    pub fn convert(
        &self,
        equirect_path: &str,
        imagic_context: &mut ImagicContext,
        face_size: u32,
        format: wgpu::TextureFormat,
    ) -> ID {
        let material_index = Self::get_equirect_to_cube_material(equirect_path, imagic_context);
        self.convert_core(material_index, imagic_context, face_size, format)
    }

    pub fn convert_by_bytes(
        &self,
        equirect_img_data: &[u8],
        imagic_context: &mut ImagicContext,
        face_size: u32,
        format: wgpu::TextureFormat,
    ) -> ID {
        let material_index =
            self.get_equirect_to_cube_material_by_bytes(equirect_img_data, imagic_context);
        self.convert_core(material_index, imagic_context, face_size, format)
    }

    fn convert_core(
        &self,
        material_index: usize,
        imagic_context: &mut ImagicContext,
        face_size: u32,
        format: wgpu::TextureFormat,
    ) -> ID {
        let mut cube = Cube::new(1.0, 1.0, 1.0, 1, 1, 1);
        cube.init(imagic_context, material_index);
        cube.set_layer(
            Layer::RenderTarget,
            imagic_context.render_item_manager_mut(),
        );

        let camera = Self::create_camera(imagic_context, face_size as f32);
        let cube_rt = CubeRenderTexture::create(imagic_context, format, face_size, face_size);
        let rt_texture_id = cube_rt.get_color_attachment_id();
        camera.borrow_mut().set_render_texture(Box::new(cube_rt));
        camera.borrow_mut().render(imagic_context);
        rt_texture_id
    }

    fn create_camera(imagic_context: &mut ImagicContext, viewport_size: f32) -> RR<Camera> {
        let camera_id = Camera::new(
            Vec3::ZERO,
            f32::consts::FRAC_PI_2,
            1.0,
            0.1,
            10.0,
            None,
            imagic_context,
        );

        let camera = imagic_context.camera_manager_mut().get_camera(camera_id);
        camera
            .borrow_mut()
            .set_viewport(Vec4::new(0.0, 0.0, 1.0, 1.0));
        camera
            .borrow_mut()
            .set_logical_viewport(Vec4::new(0.0, 0.0, viewport_size, viewport_size));
        camera.borrow_mut().set_physical_viewport(Vec4::new(
            0.0,
            0.0,
            viewport_size,
            viewport_size,
        ));
        camera.borrow_mut().set_clear_color(Vec4::ZERO);
        camera.borrow_mut().layer_mask = LayerMask::new(Layer::RenderTarget.into());
        camera.borrow_mut().draw_manually = true;
        // Note here: avoid viewport is automatically resized when window resized.
        camera.borrow_mut().is_viewport_auto_resizeable = false;
        camera
    }

    fn get_equirect_to_cube_material(
        equirect_path: &str,
        imagic_context: &mut ImagicContext,
    ) -> ID {
        let mut hdr_loader = HDRLoader::default();
        let hdr_texture = hdr_loader.load(equirect_path, imagic_context.graphics_context());
        Self::get_equirect_to_cube_material_core(hdr_texture, imagic_context)
    }

    fn get_equirect_to_cube_material_by_bytes(
        &self,
        equirect_img_data: &[u8],
        imagic_context: &mut ImagicContext,
    ) -> ID {
        let mut hdr_loader = HDRLoader::new(HDRLoaderOptions{is_flip_y: self.is_flip_y});
        let hdr_texture =
            hdr_loader.load_by_bytes(equirect_img_data, imagic_context.graphics_context());
        Self::get_equirect_to_cube_material_core(hdr_texture, imagic_context)
    }

    fn get_equirect_to_cube_material_core(
        hdr_texture: Texture,
        imagic_context: &mut ImagicContext,
    ) -> ID {
        let hdr_texture_index = imagic_context
            .texture_manager_mut()
            .add_texture(hdr_texture);

        let mut equirectangular_to_cube_material = Box::new(EquirectangularToCubeMaterial::new());
        equirectangular_to_cube_material.set_equirectangular_map(hdr_texture_index);
        // Note here: camera is inside the box, we should render the back face.
        equirectangular_to_cube_material.set_cull_mode(wgpu::Face::Front);
        let material_index = imagic_context.add_material(equirectangular_to_cube_material);
        material_index
    }
}
