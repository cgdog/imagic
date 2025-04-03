use crate::{
    asset::asset::Handle, model::Cube, prelude::{ImagicContext, Material, MaterialTrait, SkyboxMaterial, Texture}
};

/// Skybox representation. It includes a background cube texture and an environment refelction cube texture.
pub struct Skybox {
    skybox_material_id: Handle<Material>,
    cube: Cube,
}

impl Default for Skybox {
    fn default() -> Self {
        Self {
            // background_texture_id: INVALID_ID,
            skybox_material_id: Handle::INVALID,
            cube: Cube::default(),
        }
    }
}

impl Skybox {
    /// Initialize teh skybox background cubetexture with ldr texture bytes
    pub fn init_ldr_bytes(&mut self, imagic_context: &mut ImagicContext, buffers: [&[u8]; 6]) {
        let cube_texture = Texture::create_cube_texture_from_bytes(
            imagic_context.graphics_context(),
            buffers,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            1,
        );
        let cube_texture_handle = imagic_context
            .asset_manager_mut()
            .add(cube_texture);

        let mut skybox_material = Box::new(SkyboxMaterial::new());
        skybox_material.set_skybox_map(cube_texture_handle);
        skybox_material.set_cull_mode(wgpu::Face::Front);
        let skybox_material_id = imagic_context.add_material(skybox_material as Material);

        self.cube.init(imagic_context, skybox_material_id.clone());

        self.skybox_material_id = skybox_material_id;
    }

    /// Initialize the skybox with a background cube texture.
    pub fn init_with_cube_texture(
        &mut self,
        imagic_context: &mut ImagicContext,
        background_cube_texture: Handle<Texture>,
    ) {
        let mut skybox_material = Box::new(SkyboxMaterial::new());
        skybox_material.set_skybox_map(background_cube_texture);
        skybox_material.set_cull_mode(wgpu::Face::Front);
        let skybox_material_id = imagic_context.add_material(skybox_material as Material);

        self.cube.init(imagic_context, skybox_material_id.clone());

        self.skybox_material_id = skybox_material_id;
    }

    pub fn init_with_custom_material(&mut self, imagic_context: &mut ImagicContext, material: Handle<Material>) {
        self.cube.init(imagic_context, material);
    }
}
