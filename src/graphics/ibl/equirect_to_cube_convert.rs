use core::f32;

use crate::{
    asset::{asset::Handle, loaders::hdr_loader::{HDRLoader, HDRLoaderOptions}}, camera::{Camera, Layer}, math::Vec4, model::Cube, prelude::{
        CubeRenderTexture, EquirectangularToCubeMaterial,
        ImagicContext, MaterialTrait, RenderTexture, Texture,
    }, scene::SceneObject, types::ID
};

pub struct EquirectToCubeConverter {
    is_flip_y: bool,
}

impl Default for EquirectToCubeConverter {
    fn default() -> Self {
        Self { is_flip_y: false }
    }
}

/// Convert an equirectangular image to cube texture.
impl EquirectToCubeConverter {
    pub(crate) fn new(is_flip_y: bool) -> Self {
        Self { is_flip_y }
    }

    /// Convert an equirectangular map to a CubeMap.
    pub(crate) fn convert(
        &self,
        equirect_path: &str,
        imagic_context: &mut ImagicContext,
        face_size: u32,
        format: wgpu::TextureFormat,
        camera: &mut Camera,
        cube: &mut Cube,
        // sync_buffer: &SyncBuffer,
    ) -> Handle<Texture> {
        let material_index = Self::get_equirect_to_cube_material(equirect_path, imagic_context);
        self.convert_core(
            material_index,
            imagic_context,
            face_size,
            format,
            camera,
            cube,
            // sync_buffer,
        )
    }

    pub(crate) fn convert_by_bytes(
        &self,
        equirect_img_data: &[u8],
        imagic_context: &mut ImagicContext,
        face_size: u32,
        format: wgpu::TextureFormat,
        camera: &mut Camera,
        cube: &mut Cube,
        // sync_buffer: &SyncBuffer,
    ) -> Handle<Texture> {
        let material_index =
            self.get_equirect_to_cube_material_by_bytes(equirect_img_data, imagic_context);
        self.convert_core(
            material_index,
            imagic_context,
            face_size,
            format,
            camera,
            cube,
            // sync_buffer,
        )
    }

    fn convert_core(
        &self,
        material_index: usize,
        imagic_context: &mut ImagicContext,
        face_size: u32,
        format: wgpu::TextureFormat,
        camera: &mut Camera,
        cube: &mut Cube,
        // sync_buffer: &SyncBuffer,
    ) -> Handle<Texture> {
        cube.init(imagic_context, material_index);
        cube.set_layer(
            Layer::RenderTarget,
            imagic_context.render_item_manager_mut(),
        );

        let cube_rt = CubeRenderTexture::new(imagic_context, format, face_size, face_size, 1);
        let rt_texture_id = cube_rt.get_color_attachment_handle();
        camera.set_viewport(Vec4::new(0.0, 0.0, 1.0, 1.0));

        let viewport_size = face_size as f32;
        camera.set_logical_viewport(Vec4::new(0.0, 0.0, viewport_size, viewport_size));
        camera.set_physical_viewport(Vec4::new(0.0, 0.0, viewport_size, viewport_size));
        camera.set_render_texture(Box::new(cube_rt));
        camera.render(imagic_context, None);
        rt_texture_id
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
        let mut hdr_loader = HDRLoader::new(HDRLoaderOptions {
            is_flip_y: self.is_flip_y,
        });
        let hdr_texture =
            hdr_loader.load_by_bytes(equirect_img_data, imagic_context.graphics_context());
        Self::get_equirect_to_cube_material_core(hdr_texture, imagic_context)
    }

    fn get_equirect_to_cube_material_core(
        hdr_texture: Texture,
        imagic_context: &mut ImagicContext,
    ) -> ID {
        let hdr_texture_handle = imagic_context
            .asset_manager_mut()
            .add(hdr_texture);

        let mut equirectangular_to_cube_material = Box::new(EquirectangularToCubeMaterial::new());
        equirectangular_to_cube_material.set_equirectangular_map(hdr_texture_handle);
        // Note here: camera is inside the box, we should render the back face.
        equirectangular_to_cube_material.set_cull_mode(wgpu::Face::Front);
        let material_index = imagic_context.add_material(equirectangular_to_cube_material);
        material_index
    }
}
