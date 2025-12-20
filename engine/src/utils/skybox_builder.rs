use crate::{
    RR_new,
    assets::{
        INVALID_TEXTURE_HANDLE, TextureDimension, TextureFormat, TextureHandle, TextureUsages,
        environment::skybox::Skybox, materials::material::Material,
        meshes::{mesh::Mesh, primitives::cuboid::Cuboid},
        sampler::Sampler
    },
    components::mesh_renderer::MeshRenderer,
    core::Engine,
    graphics::render_states::{CullMode, RenderQueue},
};

pub struct SkyboxBuilderOptions {
    pub is_flip_y: bool,
    pub is_bake_irradiance: bool,
    pub is_bake_reflection: bool,
    pub is_generate_brdf_lut: bool,
}

impl Default for SkyboxBuilderOptions {
    fn default() -> Self {
        Self {
            is_flip_y: false,
            is_bake_irradiance: true,
            is_bake_reflection: true,
            is_generate_brdf_lut: true,
        }
    }
}

pub struct SkyboxBuilder {}

impl SkyboxBuilder {
    /// Create a skybox node with given skybox texture data.
    pub fn create_skybox(
        engine: &mut Engine,
        cube_map_data: Vec<Vec<u8>>,
        texture_format: TextureFormat,
        skybox_builder_options: SkyboxBuilderOptions,
    ) ->crate::prelude::NodeId {
        // create cubemap texture and sampler.
        let skybox_texture_handle: TextureHandle;
        let input_texture_handle: TextureHandle;
        let mut is_inpunt_cube_map = false;
        let skybox_sampler = Sampler::default_sampler();
        {
            let cube_map_data_length = cube_map_data.len();
            let is_cube_map = cube_map_data_length == 6;
            input_texture_handle = engine.texture_sampler_manager.create_texture_from_image_with_usages(
                cube_map_data,
                TextureDimension::D2,
                texture_format,
                skybox_builder_options.is_flip_y,
                is_cube_map,
                TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC | TextureUsages::COPY_DST,
            );
            if is_cube_map {
                skybox_texture_handle = input_texture_handle;
                is_inpunt_cube_map = true;
            } else {
                // cube_map_data_length == 1，more steps are needed when initialization.
                skybox_texture_handle = INVALID_TEXTURE_HANDLE;
            };
        }

        // create skybox material
        let shader_skybox = engine.shader_manager.get_builtin_skybox_shader();
        let material_skybox = Material::new(shader_skybox);
        {
            let mut material_skybox_mut_ref = material_skybox.borrow_mut();
            if is_inpunt_cube_map {
                material_skybox_mut_ref.set_texture("skybox_cube_texture", skybox_texture_handle);
            }
            material_skybox_mut_ref.set_sampler("skybox_cube_sampler", skybox_sampler);
            // note here.
            material_skybox_mut_ref.render_state.cull_mode = CullMode::Front;
            material_skybox_mut_ref.render_state.render_queue = RenderQueue::Skybox;
        }

        // create cuboid that represents Skybox geometry.
        let scene = engine.world.current_scene_mut();
        let cuboid_node = scene.create_node("Skybox");
        {
            let mesh: Mesh = Cuboid::default().into();
            let mesh = RR_new!(mesh);
            let mesh_renderer = MeshRenderer::new(mesh, vec![material_skybox]);
            scene.add_component(&cuboid_node, mesh_renderer);
            let skybox_component = Skybox::new(input_texture_handle, is_inpunt_cube_map);
            scene.add_component(&cuboid_node, skybox_component);
        }
        scene.cached_skybox_ = cuboid_node;
        scene.add(cuboid_node.clone());
        cuboid_node
    }
}
