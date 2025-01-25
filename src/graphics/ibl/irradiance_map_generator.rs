use crate::{
    camera::Camera,
    math::Vec4,
    prelude::{CubeRenderTexture, ImagicContext, IrradianceMapGenMaterial, MaterialTrait, RenderTexture},
    types::ID,
};

/// Generate a irradiance map for a CubeTexture
pub struct IrradianceMapGenerator {}

impl IrradianceMapGenerator {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn generate(
        &self,
        imagic_context: &mut ImagicContext,
        input_cube_texture: ID,
        face_size: u32,
        box_item_id: ID,
        format: wgpu::TextureFormat,
        camera: &mut Camera,
        // sync_buffer: &SyncBuffer,
    ) -> ID {
        let irradiance_map_gen_material =
            self.create_irradiance_map_gen_map(imagic_context, input_cube_texture);
        imagic_context
            .pipeline_manager()
            .borrow_mut()
            .remove_render_pipeline(box_item_id);

        imagic_context
            .render_item_manager_mut()
            .get_render_item_mut(box_item_id)
            .set_material_id(irradiance_map_gen_material);

        let cube_rt = CubeRenderTexture::create(imagic_context, format, face_size, face_size);
        // camera change viewport
        let viewport_size = face_size as f32;
        camera.set_logical_viewport(Vec4::new(0.0, 0.0, viewport_size, viewport_size));
        camera.set_physical_viewport(Vec4::new(
            0.0,
            0.0,
            viewport_size,
            viewport_size,
        ));
        let rt_texture_id = cube_rt.get_color_attachment_id();
        camera.set_render_texture(Box::new(cube_rt));
        camera.render(imagic_context, None);
        rt_texture_id
    }

    fn create_irradiance_map_gen_map(
        &self,
        imagic_context: &mut ImagicContext,
        cube_texture: ID,
    ) -> ID {
        let mut material = IrradianceMapGenMaterial::new();
        material.set_input_cube_map(cube_texture);
        material.set_cull_mode(wgpu::Face::Front);
        imagic_context.add_material(Box::new(material))
    }
}
