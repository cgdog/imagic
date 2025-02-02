use crate::{
    camera::Camera,
    math::Vec4,
    model::Cube,
    prelude::{
        create_cube_color_attachment_views, create_cube_depth_views, CubeRenderTexture,
        EnvironmentPrefilterMaterial, ImagicContext, MaterialTrait, RenderTexture,
    },
    types::ID,
};

pub struct CubeTexturePrefilter {
    input_cube_texture: ID,
    mipmap_level_count: u32,
    face_size: u32,
}

impl CubeTexturePrefilter {
    pub fn new(input_cube_texture: ID, mipmap_level_count: u32, face_size: u32) -> Self {
        Self {
            input_cube_texture,
            mipmap_level_count,
            face_size,
        }
    }

    pub fn prefilter(
        &mut self,
        imagic_context: &mut ImagicContext,
        camera: &mut Camera,
        cube: &mut Cube,
        rt_format: wgpu::TextureFormat,
    ) -> ID {
        let prefilter_material_id = self.get_prefilter_material(imagic_context);
        let box_item_id = cube.render_item_id();

        imagic_context
            .pipeline_manager()
            .borrow_mut()
            .remove_render_pipeline(box_item_id);

        imagic_context
            .render_item_manager_mut()
            .get_render_item_mut(box_item_id)
            .set_material_id(prefilter_material_id);

        let total_mipmaps_count = self.mipmap_level_count; //self.face_size.ilog2() + 1;
        let cube_rt = CubeRenderTexture::new(
            imagic_context,
            rt_format,
            self.face_size,
            self.face_size,
            total_mipmaps_count,
        );

        let rt_color_attachment = cube_rt.get_color_attachment_id();

        camera.set_render_texture(Box::new(cube_rt));

        for mip in 0..self.mipmap_level_count {
            let mip_size = self.face_size >> mip;
            let viewport_size = mip_size as f32;
            camera.set_viewport(Vec4::new(0.0, 0.0, 1.0, 1.0));
            camera.set_logical_viewport(Vec4::new(0.0, 0.0, viewport_size, viewport_size));
            camera.set_physical_viewport(Vec4::new(0.0, 0.0, viewport_size, viewport_size));

            if let Some(rt) = camera.get_render_texture() {
                if mip != 0 {
                    let depth_texture_id = rt.get_depth_attachment_id();
                    let depth_texture = imagic_context
                        .texture_manager_mut()
                        .get_texture(depth_texture_id);
                    let new_depth_views = create_cube_depth_views(depth_texture, mip);
                    rt.set_depth_attachment_views(new_depth_views);

                    let color_attachment_id = rt.get_color_attachment_id();
                    let color_texture = imagic_context
                        .texture_manager_mut()
                        .get_texture(color_attachment_id);
                    let new_color_attachment_views =
                        create_cube_color_attachment_views(color_texture, mip, rt_format);
                    rt.set_color_attachment_views(new_color_attachment_views);
                }
            }

            let roughness = (mip as f32) / (self.mipmap_level_count as f32);
            let material = imagic_context
                .material_manager_mut()
                .get_material_mut(prefilter_material_id);
            if let Some(prefilter_material) = material
                .as_any_mut()
                .downcast_mut::<EnvironmentPrefilterMaterial>()
            {
                prefilter_material.set_roughness(roughness);
                imagic_context.update_material(prefilter_material_id);
            }
            camera.render(imagic_context, None);
            // imagic_context
            // .graphics_context()
            // .get_device()
            // .poll(wgpu::Maintain::Wait);
        }

        rt_color_attachment
    }

    fn get_prefilter_material(&self, imagic_context: &mut ImagicContext) -> ID {
        let mut prefilter_material =
            EnvironmentPrefilterMaterial::new(self.input_cube_texture, self.mipmap_level_count);
        prefilter_material.set_cull_mode(wgpu::Face::Front);
        imagic_context.add_material(Box::new(prefilter_material))
    }
}
