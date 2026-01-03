use crate::{
    assets::{
        TextureDimension, TextureFormat, TextureHandle, TextureUsages,
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
    ) ->crate::prelude::NodeHandle {
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
                skybox_texture_handle = TextureHandle::INVALID;
            };
        }

        // create skybox material
        let (_, shader_skybox_handle) = engine.shader_manager.get_builtin_skybox_shader();
        let mut material_skybox = Material::new(*shader_skybox_handle, &mut engine.shader_manager);
        {
            if is_inpunt_cube_map {
                material_skybox.set_texture("skybox_cube_texture", skybox_texture_handle);
            }
            material_skybox.set_sampler("skybox_cube_sampler", skybox_sampler);
            // note here.
            material_skybox.render_state.cull_mode = CullMode::Front;
            material_skybox.render_state.render_queue = RenderQueue::Skybox;
        }

        let material_skybox_handle = engine.material_manager.add_material(material_skybox);

        // create cuboid that represents Skybox geometry.
        let scene = engine.world.current_scene_mut();
        let cuboid_node = scene.create_node("Skybox");
        {
            let mesh: Mesh = Cuboid::default().into();
            let mesh_handle = engine.mesh_manager.add_mesh(mesh);
            let mesh_renderer = MeshRenderer::new(mesh_handle, vec![material_skybox_handle]);
            scene.add_component(&cuboid_node, mesh_renderer);
            let skybox_component = Skybox::new(input_texture_handle, is_inpunt_cube_map);
            scene.add_component(&cuboid_node, skybox_component);
        }
        scene.cached_skybox_ = cuboid_node;
        scene.add(cuboid_node.clone());
        cuboid_node
    }
}
