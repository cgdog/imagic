use std::cell::RefCell;

use crate::{
    assets::{
        BuiltinGlobalShaderFeatures, MaterialManager, MeshManager, Sampler, ShaderManager, Texture, TextureFormat, TextureHandle,
        TextureSamplerManager, shaders::shader_property::BuiltinShaderUniformNames
    }, components::{camera::Camera, mesh_renderer::MeshRenderer},
    core::{LayerMask, NodeHandle, SH, scene::Scene}, graphics::{
        bind_group::BindGroupID, graphics_context::GraphicsContext, render_states::RenderQueue,
        uniform::{BuiltinUniforms, CameraUniformSyncFlags, GlobalUniformSyncFlags}
    }, math::Vec4, renderer::{
        frame_data::{CameraRenderData, ItemRenderData}, frame_renderer::FrameRenderer,
    }, time::Time
};

/// A world in the scene. It is a container for scenes.
pub struct World {
    /// The scenes in the world. At least one scene is required.
    /// 
    /// (Now only one scene is supported. Two or more scenes are not tested.)
    pub scenes: Vec<Scene>,
    /// The index of the current scene.
    pub current_scene_index: usize,
}

impl World {
    
    /// Creates a new world with a default scene.
    /// 
    /// # Returns
    /// 
    /// * `World` - The created world.
    pub(crate) fn new() -> World {
        let current_scene = Scene::new();
        Self::new_with_current_scene(current_scene)
    }

    /// Creates a new world with the given scene.
    /// 
    /// # Arguments
    /// 
    /// * `current_scene` - The scene to add to the world.
    /// 
    /// # Returns
    /// 
    /// * `World` - The created world.
    pub(crate) fn new_with_current_scene(current_scene: Scene) -> World {
        let world: World = World {
            scenes: vec![current_scene],
            current_scene_index: 0,
        };
        world
    }

    /// Returns a mutable reference to the current scene.
    /// 
    /// # Returns
    /// 
    /// * `&mut Scene` - The current scene.
    pub fn current_scene_mut(&mut self) -> &mut Scene {
        &mut self.scenes[self.current_scene_index]
    }

    /// Returns a reference to the current scene.
    /// 
    /// # Returns
    /// 
    /// * `&Scene` - The current scene.
    pub fn current_scene(&mut self) -> & Scene {
        &self.scenes[self.current_scene_index]
    }

    /// Stops the world.
    /// 
    /// # Arguments
    /// 
    /// * `time` - The time object.
    pub(crate) fn stop(&mut self, time: &mut Time) {
        self.scenes[self.current_scene_index].on_stop(time);
    }

    /// Initializes the world.
    /// 
    /// # Arguments
    /// 
    /// * `graphics_context` - The graphics context.
    /// * `texture_sampler_manager` - The texture sampler manager.
    /// * `shader_manager` - The shader manager.
    /// * `material_manager` - The material manager.
    /// * `time` - The time object.
    pub(crate) fn on_init(&mut self, graphics_context: &mut GraphicsContext, texture_sampler_manager: &mut TextureSamplerManager,
        shader_manager: &mut ShaderManager, material_manager: &mut MaterialManager, time: &mut Time) {
        log::info!("world on_init");
        self.scenes[self.current_scene_index].try_init_skybox(graphics_context, texture_sampler_manager, shader_manager, material_manager);
        self.on_resize(&graphics_context, texture_sampler_manager);
        self.scenes[self.current_scene_index].on_init(time);
    }

    /// Resizes the world.
    /// 
    /// # Arguments
    /// 
    /// * `graphics_context` - The graphics context.
    /// * `texture_sampler_manager` - The texture sampler manager.
    pub(crate) fn on_resize(&mut self, graphics_context: &GraphicsContext, texture_sampler_manager: &mut TextureSamplerManager,) {
        let physical_size = graphics_context.main_window().get_physical_size();
        let logical_size = graphics_context.main_window().get_logical_size();

        let current_scene = self.current_scene_mut();
        let cached_cameras = std::mem::take(&mut current_scene.cached_cameras);
        for camera_node in &cached_cameras {
            if let Some(camera) = &mut current_scene.get_component_mut::<Camera>(camera_node) {
                camera.on_resize(texture_sampler_manager, physical_size, logical_size);
                log::info!(
                    "World on_resize, physical size: ({}, {})",
                    physical_size.width, physical_size.height
                );
            }
        }
        current_scene.cached_cameras = cached_cameras;
    }

    /// Updates the world.
    /// 
    /// # Arguments
    /// 
    /// * `time` - The time object.
    pub(crate) fn updpate(&mut self, time: &mut Time) {
        self.current_scene_mut().on_update(time);
    }

    /// Generates a render frame.
    /// 
    /// # Arguments
    /// 
    /// * `graphics_context` - The graphics context.
    /// * `texture_sampler_manager` - The texture sampler manager.
    /// * `shader_manager` - The shader manager.
    /// * `material_manager` - The material manager.
    /// * `time` - The time object.
    /// * `frame_renderer` - The frame renderer which contains the render data.
    /// * `global_uniforms` - The global uniforms.
    pub(crate) fn generate_render_frame(&mut self, graphics_context: &mut GraphicsContext,
        texture_sampler_manager: &mut TextureSamplerManager, shader_manager: &mut ShaderManager,
        material_manager: &mut MaterialManager, mesh_manager: &mut MeshManager, time: &mut Time,
        frame_renderer: &mut FrameRenderer, global_uniforms: &mut BuiltinUniforms) {
        frame_renderer.frame_render_data.reset();
        let cur_scene = &mut self.scenes[self.current_scene_index];
        let (reflection_map, brdf_lut) = cur_scene.get_environment_reflection_info();
        let sh = cur_scene.sh;
        frame_renderer.frame_render_data.time_data = time.time_data;
        let mut global_uniform_sync_flags = GlobalUniformSyncFlags::new();
        let cached_cameras = std::mem::take(&mut cur_scene.cached_cameras);
        let cached_renderables = std::mem::take(&mut cur_scene.cached_renderables);
        for camera_node_id in &cached_cameras {
            let camera_node_ref = cur_scene.node_arena.get_forcely(camera_node_id);
            let camera_position = camera_node_ref.transform.position;
            let camera_render_data = if let Some(camera) = cur_scene.get_component::<Camera>(camera_node_id) {
                if TextureHandle::INVALID != camera.depth_attachment {
                    let view_matrix =
                        camera.get_view_matrix(&camera_position);
                    let projection_matrix = camera.get_projection_matrix();
                    let camera_render_data = CameraRenderData::new(
                        camera_node_ref.id,
                        camera.priority,
                        view_matrix,
                        projection_matrix,
                        camera.depth_attachment,
                        camera.color_attachment,
                        camera.physical_view_port,
                        camera.clear_color,
                        camera_position,
                    );
                    Some((camera_render_data, camera.visible_layers, camera.depth_format, camera.per_camera_uniforms.clone()))
                } else {
                    log::warn!(
                        "Camera {}, {} has no depth attachment!",
                        camera_node_ref.id, camera_node_ref.name
                    );
                    None
                }
            } else {
                None
            };
            if let Some((mut camera_render_data, visible_layers, depth_format, per_camera_uniforms)) = camera_render_data {
                Self::_generate_frame_per_camera(
                        cur_scene,
                        visible_layers,
                        depth_format,
                        per_camera_uniforms,
                        global_uniforms,
                        graphics_context,
                        texture_sampler_manager,
                        shader_manager,
                        material_manager,
                        mesh_manager,
                        time,
                        &mut camera_render_data,
                        &cached_renderables,
                        &mut global_uniform_sync_flags,
                        reflection_map,
                        brdf_lut,
                        &sh,
                    );
                    frame_renderer
                        .frame_render_data
                        .camera_data
                        .push(camera_render_data);
            }
        }
        cur_scene.cached_cameras = cached_cameras;
        cur_scene.cached_renderables = cached_renderables;
    }

    fn _generate_frame_per_camera(
        current_scene: &mut Scene,
        visible_layers: LayerMask,
        depth_format: TextureFormat,
        per_camera_uniforms: RefCell<BuiltinUniforms>,
        global_uniforms: &mut BuiltinUniforms,
        graphics_context: &mut GraphicsContext,
        texture_sampler_manager: &mut TextureSamplerManager,
        shader_manager: &mut ShaderManager,
        material_manager: &mut MaterialManager,
        mesh_manager: &mut MeshManager,
        time: &mut Time,
        camera_render_data: &mut CameraRenderData,
        cached_renderables: &Vec<NodeHandle>,
        global_uniform_sync_flags: &mut GlobalUniformSyncFlags,
        reflection_map: TextureHandle,
        brdf_lut: TextureHandle,
        sh: &SH,
    ) {
        let mut camera_unifom_sync_flags = CameraUniformSyncFlags::new();
        // let current_scene = self.current_scene_mut();
        for renderable_node in cached_renderables {
            let node_mut_ref = current_scene.node_arena.get_mut_forcely(renderable_node);
            if !visible_layers.contains(node_mut_ref.layer) {
                continue;
            }
            let model_matrix = node_mut_ref.transform.model_matrix;
            let normal_matrix = node_mut_ref.transform.normal_matrix;
            if let Some(mesh_renderer) = &mut current_scene.get_component_mut::<MeshRenderer>(renderable_node) 
                && let Some(mesh_mut_ref) = mesh_manager.get_mesh_mut(&mesh_renderer.mesh){
                if mesh_mut_ref.is_dirty {
                    mesh_mut_ref.upload(graphics_context);
                }
                for (sub_mesh_index, sub_mesh) in mesh_mut_ref.sub_meshes.iter().enumerate() {
                    if let Some(material) = mesh_renderer.materials.get(sub_mesh_index) {
                        let material_mut_ref = material_manager.get_material_mut_forcely(material);
                        material_mut_ref.on_update(graphics_context, texture_sampler_manager, shader_manager);
                        let render_pipeline_hash = material_mut_ref.hash_value();
                        let shader_ref = shader_manager.get_shader_forcely(&material_mut_ref.shader_handle);
                        if !graphics_context
                            .render_pipelines
                            .contains(render_pipeline_hash)
                        {
                            let target = graphics_context.get_swapchain_format().into();
                            let vertex_buffer_layout = mesh_mut_ref
                                .vertex_attributes
                                .compute_vertex_buffer_layout();
                            graphics_context.render_pipelines.create_render_pipeline(
                                render_pipeline_hash,
                                material_mut_ref,
                                shader_ref,
                                &[vertex_buffer_layout],
                                &[Some(target)],
                                depth_format,
                            );
                        }

                        let mut bind_group_pairs = Vec::<(u32, BindGroupID)>::new();
                        if material_mut_ref.uniforms.is_valid() {
                            bind_group_pairs.push((
                                material_mut_ref.uniforms.bind_group_index,
                                material_mut_ref.uniforms.bind_group_id,
                            ));
                        }
                        
                        let builtin_uniform_flags = &shader_ref.builtin_uniform_flags;
                        // per object uniforms
                        if shader_ref.shader_properties.per_object_properties.is_valid() {
                            if builtin_uniform_flags.has_model_matrix {
                                mesh_renderer.per_object_uniforms.set_matrix4x4(
                                    BuiltinShaderUniformNames::_MODEL_MATRIX,
                                    model_matrix,
                                );
                            }
                            if builtin_uniform_flags.has_normal_matrix {
                                mesh_renderer.per_object_uniforms.set_matrix3x3(
                                    BuiltinShaderUniformNames::_NORMAL_MATRIX,
                                    normal_matrix,
                                );
                            }
                            if builtin_uniform_flags.has_mvp_matrix {
                                let mvp_matrix = camera_render_data.projection_matrix
                                    * camera_render_data.view_matrix
                                    * model_matrix;
                                mesh_renderer.per_object_uniforms.set_matrix4x4(
                                    BuiltinShaderUniformNames::_MVP_MATRIX,
                                    mvp_matrix,
                                );
                            }
                            if builtin_uniform_flags.has_mv_matrix {
                                let mv_matrix = camera_render_data.view_matrix * model_matrix;
                                mesh_renderer.per_object_uniforms.set_matrix4x4(
                                    BuiltinShaderUniformNames::_MV_MATRIX,
                                    mv_matrix,
                                );
                            }
                            if builtin_uniform_flags.has_m_v_p_matrices {
                                let matrices = [
                                    model_matrix.to_cols_array(),
                                    camera_render_data.view_matrix.to_cols_array(),
                                    camera_render_data.projection_matrix.to_cols_array(),
                                ];
                                mesh_renderer.per_object_uniforms.set_struct(
                                    BuiltinShaderUniformNames::_M_V_P_MATRICES,
                                    bytemuck::cast_slice(&matrices).to_vec(),
                                );
                            }
                            if builtin_uniform_flags.has_m_v_p_n_matrices {
                                let mut bytes = Vec::with_capacity(3 * 16 * 4 + 9 * 4);
                                bytes.extend_from_slice(bytemuck::cast_slice(
                                    &model_matrix.to_cols_array(),
                                ));
                                bytes.extend_from_slice(bytemuck::cast_slice(
                                    &camera_render_data.view_matrix.to_cols_array(),
                                ));
                                bytes.extend_from_slice(bytemuck::cast_slice(
                                    &camera_render_data.projection_matrix.to_cols_array(),
                                ));
                                bytes.extend_from_slice(bytemuck::cast_slice(
                                    &normal_matrix.to_cols_array(),
                                ));
                                mesh_renderer
                                    .per_object_uniforms
                                    .set_struct(BuiltinShaderUniformNames::_M_V_P_N_MATRICES, bytes);
                            }
                            mesh_renderer
                                .per_object_uniforms
                                .sync_properties(graphics_context, texture_sampler_manager);
                            let per_object_bind_group_index = shader_ref
                                .shader_properties
                                .per_object_properties
                                .bind_group_index;
                            let per_obejct_bind_group_id =
                                mesh_renderer.per_object_uniforms.get_bind_group(
                                    graphics_context,
                                    texture_sampler_manager,
                                    &shader_ref.shader_properties.per_object_properties,
                                );
                            bind_group_pairs
                                .push((per_object_bind_group_index, per_obejct_bind_group_id));
                        }

                        // per camera uniforms
                        if shader_ref.shader_properties.per_camera_properties.is_valid() {
                            let mut need_sync_camera_uniforms = false;
                            let per_camera_uniforms_mut_ref =
                                &mut per_camera_uniforms.borrow_mut();
                            if builtin_uniform_flags.has_view_matrix && !camera_unifom_sync_flags.has_view_matrix_synced {
                                per_camera_uniforms_mut_ref.set_matrix4x4(
                                    BuiltinShaderUniformNames::_VIEW_MATRIX,
                                    camera_render_data.view_matrix,
                                );
                                camera_unifom_sync_flags.has_view_matrix_synced = true;
                                need_sync_camera_uniforms = true;
                            }
                            if builtin_uniform_flags.has_projection_matrix && !camera_unifom_sync_flags.has_projection_matrix_synced {
                                per_camera_uniforms_mut_ref.set_matrix4x4(
                                    BuiltinShaderUniformNames::_PROJECTION_MATRIX,
                                    camera_render_data.projection_matrix,
                                );
                                camera_unifom_sync_flags.has_projection_matrix_synced = true;
                                need_sync_camera_uniforms = true;
                            }
                            if builtin_uniform_flags.has_vp_matrix && !camera_unifom_sync_flags.has_vp_matrix_synced {
                                let vp_matrix = camera_render_data.projection_matrix * camera_render_data.view_matrix;
                                per_camera_uniforms_mut_ref.set_matrix4x4(
                                    BuiltinShaderUniformNames::_VP_MATRIX,
                                    vp_matrix,
                                );
                                camera_unifom_sync_flags.has_vp_matrix_synced = true;
                                need_sync_camera_uniforms = true;
                            }
                            if builtin_uniform_flags.has_v_p_matrices && !camera_unifom_sync_flags.has_v_p_matrices_synced {
                                let mut bytes = Vec::with_capacity(2 * 16 * 4);
                                bytes.extend_from_slice(bytemuck::cast_slice(
                                    &camera_render_data.view_matrix.to_cols_array(),
                                ));
                                bytes.extend_from_slice(bytemuck::cast_slice(
                                    &camera_render_data.projection_matrix.to_cols_array(),
                                ));
                                per_camera_uniforms_mut_ref.set_struct(BuiltinShaderUniformNames::_V_P_MATRICES, bytes);
                                camera_unifom_sync_flags.has_v_p_matrices_synced = true;
                                need_sync_camera_uniforms = true;
                            }
                            if builtin_uniform_flags.has_camera_position && !camera_unifom_sync_flags.has_camera_position_synced {
                                per_camera_uniforms_mut_ref.set_vec4f(
                                    BuiltinShaderUniformNames::_CAMERA_POSITION,
                                    Vec4::from((camera_render_data.camera_position, 1.0)),
                                );
                                camera_unifom_sync_flags.has_camera_position_synced = true;
                                need_sync_camera_uniforms = true;
                            }
                            if need_sync_camera_uniforms {
                                per_camera_uniforms_mut_ref.sync_properties(graphics_context, texture_sampler_manager);
                            }
                            let per_camera_bind_group_id = per_camera_uniforms_mut_ref
                                .get_bind_group(
                                    graphics_context,
                                    texture_sampler_manager,
                                    &shader_ref.shader_properties.per_camera_properties,
                                );
                            bind_group_pairs.push((
                                shader_ref
                                    .shader_properties
                                    .per_camera_properties
                                    .bind_group_index,
                                per_camera_bind_group_id,
                            ));
                        }

                        // global uniforms
                        if shader_ref.shader_properties.per_scene_properties.is_valid() {
                            let mut need_sync_global_uniforms = false;
                            if builtin_uniform_flags.has_time && !global_uniform_sync_flags.has_time_synced {
                                global_uniforms.set_vec4f(
                                    BuiltinShaderUniformNames::_TIME,
                                    time.time_data,
                                );
                                global_uniform_sync_flags.has_time_synced = true;
                                need_sync_global_uniforms = true;
                            }
                            if builtin_uniform_flags.has_environment_reflection_info() && !global_uniform_sync_flags.has_reflection_maps_synced {
                                assert_ne!(reflection_map, TextureHandle::INVALID, "Reflection cube map is invalid!");
                                global_uniforms.set_struct(BuiltinShaderUniformNames::_SH, bytemuck::bytes_of(sh).to_vec());
                                global_uniforms.set_texture(BuiltinShaderUniformNames::_REFLECTION_CUBE_MAP, reflection_map);
                                global_uniforms.set_sampler(BuiltinShaderUniformNames::_REFLECTION_CUBE_SAMPLER, Sampler::default_sampler());
                                global_uniforms.set_texture(BuiltinShaderUniformNames::_BRDF_LUT, brdf_lut);
                                if reflection_map != Texture::default_cube_texture() {
                                    global_uniforms.enable_global_feature(BuiltinGlobalShaderFeatures::FEATURE_FLAG_IBL);
                                } else {
                                    global_uniforms.disable_global_feature(BuiltinGlobalShaderFeatures::FEATURE_FLAG_IBL,
                                        !global_uniform_sync_flags.has_reflection_maps_synced);
                                }
                                global_uniform_sync_flags.has_reflection_maps_synced = true;
                                need_sync_global_uniforms = true;
                            }
                            if need_sync_global_uniforms {
                                global_uniforms.sync_properties(graphics_context, texture_sampler_manager);
                            }
                            let global_bind_group_id = global_uniforms.get_bind_group(
                                graphics_context,
                                texture_sampler_manager,
                                &shader_ref.shader_properties.per_scene_properties,
                            );
                            bind_group_pairs.push((
                                shader_ref
                                    .shader_properties
                                    .per_scene_properties
                                    .bind_group_index,
                                global_bind_group_id,
                            ));
                        }

                        bind_group_pairs.sort_by_key(|&(key, _)| key);
                        let bind_group_ids: Vec<BindGroupID> =
                            bind_group_pairs.iter().map(|&(_, id)| id).collect();

                        let item_render_data = ItemRenderData::new(
                            bind_group_ids,
                            render_pipeline_hash,
                            mesh_mut_ref.vertex_buffer,
                            Some(mesh_mut_ref.index_buffer),
                            mesh_mut_ref.index_data.index_format(),
                            sub_mesh.index_start,
                            sub_mesh.index_count,
                            sub_mesh.base_vertex,
                        );
                        match material_mut_ref.render_state.render_queue {
                            RenderQueue::Opaque => {
                                camera_render_data.opaque_item_data.push(item_render_data);
                            }
                            RenderQueue::Skybox => {
                                camera_render_data.skybox_item_data = Some(item_render_data);
                            }
                            RenderQueue::Transparent => {
                                camera_render_data.transparent_item_data.push(item_render_data);
                            }
                        } 
                    }
                }
            }
        }
    }
}
