use crate::{
    assets::{
        Quad, Sampler, TextureAspect, TextureDimension, TextureFormat,
        TextureHandle, TextureSamplerManager, TextureUsages, TextureViewDescriptor, TextureViewDimension,
        materials::material::Material,
        meshes::{mesh::Mesh, primitives::cuboid::Cuboid},
        sampler::{AddressMode, FilterMode},
        shaders::shader::Shader, texture_view::TextureView
    },
    components::camera::Camera, graphics::{
        bind_group::BindGroupID, graphics_context::GraphicsContext, render_api::RenderAPI,
        render_states::CullMode,
    }, impl_component, math::{Mat4, Vec3, Vec4, color::Color},
    prelude::render_pipeline::INVALID_PIPELINE_HASH, renderer::frame_data::ItemRenderData, types::RR
};

/// Skybox component.
pub struct Skybox {
    pub background_cube_map: TextureHandle,
    pub input_texture: TextureHandle,
    pub is_inpunt_cube_map: bool,
    pub reflection_cube_map: TextureHandle,
    pub reflection_cube_face_resolution: u32,
    pub brdf_lut: TextureHandle,
    pub sh: [Color; 9] 
}

impl_component!(Skybox);

impl Skybox {
    const DEFAULT_DEPTH_ATTACHMENT_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub(crate) fn new(input_texture: TextureHandle, is_inpunt_cube_map: bool) -> Self {
        let cube_map = if is_inpunt_cube_map {
            input_texture
        } else {
            TextureHandle::INVALID
        };
        Self {
            background_cube_map: cube_map,
            input_texture,
            is_inpunt_cube_map,
            reflection_cube_map: TextureHandle::INVALID,
            reflection_cube_face_resolution: 128,
            brdf_lut: TextureHandle::INVALID,
            sh: [Color::BLACK; 9],
        }
    }

    pub(crate) fn should_init(&self) -> bool{
        !self.is_inpunt_cube_map || self.reflection_cube_map == TextureHandle::INVALID
    }

    pub(crate) fn on_init(&mut self, graphics_context: &mut GraphicsContext, texture_sampler_manager: &mut TextureSamplerManager) {
        log::info!("Skybox componnet on_init");

        if !self.is_inpunt_cube_map {
            self.convert_equirect_to_cube(graphics_context, texture_sampler_manager);
            self.is_inpunt_cube_map = true;
        }
        else {
            // self._convolute(graphics_context);
            self.prefilter_reflection_map(graphics_context, texture_sampler_manager);
        }
        
        let sh_tools = crate::utils::sh_tools::CubeMapToSHTools::new();
        let sh = sh_tools.generate_sh_from_cube_texture_handle(&self.background_cube_map, graphics_context, texture_sampler_manager);
        // log::info!("sh coefs: {},{},{},{},{},{},{},{},{}", sh[0], sh[1], sh[2], sh[3], sh[4], sh[5], sh[6], sh[7], sh[8]);
        self.sh.iter_mut().zip(sh.iter()).for_each(|(sh4, sh3)| {
            sh4.r = sh3.x;
            sh4.g = sh3.y;
            sh4.b = sh3.z;
        });

        if self.brdf_lut == TextureHandle::INVALID {
            self.generate_brdf_lut(graphics_context, texture_sampler_manager);
        }
    }

    fn generate_brdf_lut(&mut self, graphics_context: &mut GraphicsContext, texture_sampler_manager: &mut TextureSamplerManager) {
        let mut quad_mesh: Mesh = Quad::default().into();
        quad_mesh.upload(graphics_context);
        let mesh_mut_ref = &mut quad_mesh;

        let color_attachment_format = TextureFormat::Rgba32Float;
        let depth_attachment_format = TextureFormat::Depth32Float;
        let rt_size = 512;
        let (color_attachment, _depth_attachment, color_attachment_view, depth_attachment_view) =
            Self::create_2d_rt(texture_sampler_manager, rt_size, rt_size, color_attachment_format, depth_attachment_format);
        self.brdf_lut = color_attachment;

        let brdf_lut_shader = Shader::new(
            include_str!("../shaders/wgsl/brdf_integration.wgsl"),
            "brdf_integration".to_owned(),
        );
        let item_render_data: ItemRenderData;
        let brdf_lut_material = Material::new(brdf_lut_shader);
        {
            let mut brdf_lut_material_mut_ref = brdf_lut_material.borrow_mut();
            brdf_lut_material_mut_ref.on_update(graphics_context, texture_sampler_manager);
            let render_pipeline_hash = brdf_lut_material_mut_ref.hash_value();
            if !graphics_context
                .render_pipelines
                .contains(render_pipeline_hash)
            {
                let depth_format = Self::DEFAULT_DEPTH_ATTACHMENT_FORMAT;
                let vertex_buffer_layout = mesh_mut_ref
                    .vertex_attributes
                    .compute_vertex_buffer_layout();
                graphics_context.render_pipelines.create_render_pipeline(
                    render_pipeline_hash,
                    &brdf_lut_material_mut_ref,
                    &[vertex_buffer_layout],
                    &[Some(color_attachment_format.into())],
                    depth_format,
                );
            }
            let sub_mesh = &mesh_mut_ref.sub_meshes[0];
            item_render_data = ItemRenderData::new(
                vec![],
                render_pipeline_hash,
                mesh_mut_ref.vertex_buffer,
                Some(mesh_mut_ref.index_buffer),
                mesh_mut_ref.index_data.index_format(),
                sub_mesh.index_start,
                sub_mesh.index_count,
                sub_mesh.base_vertex,
            );
        }
        let physical_view_port = Vec4::new(0.0, 0.0, rt_size as f32, rt_size as f32);
        RenderAPI::render_item_to_rt_directly(
            graphics_context,
            Some(Color::BLACK),
            physical_view_port,
            &color_attachment_view,
            &depth_attachment_view,
            &item_render_data,
        );
    }

    fn convert_equirect_to_cube(&mut self, graphics_context: &mut GraphicsContext, texture_sampler_manager: &mut TextureSamplerManager) {
        let mut cube_mesh: Mesh = Cuboid::default().into();
        // The cube mesh used to rendering (convert equirect to cubemap).
        cube_mesh.upload(graphics_context);
        let mesh_mut_ref = &mut cube_mesh;

        let mut hdri_width = 1;
        let mut hdri_height = 1;
        texture_sampler_manager.ensure_gpu_texture_valid(&self.input_texture);
        let textuer_format: TextureFormat;
        if let Some(intput_texture) = texture_sampler_manager.get_texture(&self.input_texture) {
            hdri_width = intput_texture.size.width;
            hdri_height = intput_texture.size.height;
            textuer_format = intput_texture.format;
        } else {
            textuer_format = TextureFormat::Rgba32Float;
        }
        log::info!("hdri width: {}", hdri_width);
        log::info!("hdri height: {}", hdri_height);
        let mut skybox_face_resolution = hdri_height / 2;
        // make cube_map_face_resolution be power of two or next power of two.
        skybox_face_resolution = skybox_face_resolution.next_power_of_two();
        log::info!("cube map face resolution: {}", skybox_face_resolution);

        let mip_level_count = skybox_face_resolution.ilog2() + 1;
        let (cube_map, depth_attachment, cube_color_attachment_views, cube_depth_attachment_views) =
            Self::create_cube_rt(
                texture_sampler_manager,
                skybox_face_resolution,
                textuer_format,
                Self::DEFAULT_DEPTH_ATTACHMENT_FORMAT,
                mip_level_count,
            );
        self.background_cube_map = cube_map;

        let sampler_2d = texture_sampler_manager.create_sampler(
            AddressMode::ClampToEdge,
            AddressMode::ClampToEdge,
            AddressMode::ClampToEdge,
            FilterMode::Linear,
            FilterMode::Linear,
            FilterMode::Linear,
        );
        texture_sampler_manager.create_gpu_sampler(&sampler_2d);

        let equirect_to_cube_shader = Shader::new(
            include_str!("../shaders/wgsl/equirect_to_cube.wgsl"),
            "equirect_to_cube".to_owned(),
        );
        let mut bind_group_ids: Vec<BindGroupID> = Vec::new();
        let mut item_render_data: ItemRenderData;
        let euqirect_to_cube_material = Material::new(equirect_to_cube_shader);
        {
            let mut euqirect_to_cube_material_mut_ref = euqirect_to_cube_material.borrow_mut();
            euqirect_to_cube_material_mut_ref.set_texture("skybox_2d_texture", self.input_texture);
            euqirect_to_cube_material_mut_ref.set_sampler("skybox_2d_sampler", sampler_2d);
            euqirect_to_cube_material_mut_ref.render_state.cull_mode = CullMode::Front;
            euqirect_to_cube_material_mut_ref.on_update(graphics_context, texture_sampler_manager);

            let render_pipeline_hash = euqirect_to_cube_material_mut_ref.hash_value();
            if !graphics_context
                .render_pipelines
                .contains(render_pipeline_hash)
            {
                let depth_format = Self::DEFAULT_DEPTH_ATTACHMENT_FORMAT;
                let vertex_buffer_layout = mesh_mut_ref
                    .vertex_attributes
                    .compute_vertex_buffer_layout();
                graphics_context.render_pipelines.create_render_pipeline(
                    render_pipeline_hash,
                    &euqirect_to_cube_material_mut_ref,
                    &[vertex_buffer_layout],
                    &[Some(textuer_format.into())],
                    depth_format,
                );
            }

            bind_group_ids.push(euqirect_to_cube_material_mut_ref.get_bind_group());
            let sub_mesh = &mesh_mut_ref.sub_meshes[0];

            item_render_data = ItemRenderData::new(
                bind_group_ids,
                render_pipeline_hash,
                mesh_mut_ref.vertex_buffer,
                Some(mesh_mut_ref.index_buffer),
                mesh_mut_ref.index_data.index_format(),
                sub_mesh.index_start,
                sub_mesh.index_count,
                sub_mesh.base_vertex,
            );
        }

        let (mut camera, vp_matrices) = Self::create_cube_camera_info(
            self.background_cube_map,
            depth_attachment,
            skybox_face_resolution,
        );

        Self::render_cube_rt(
            texture_sampler_manager,
            &mut camera,
            &vp_matrices,
            &cube_color_attachment_views,
            &cube_depth_attachment_views,
            &item_render_data,
            &euqirect_to_cube_material,
            graphics_context,
            -1.0,
            skybox_face_resolution as f32,
        );

        // self._convolute_core(graphics_context, textuer_format, mesh_mut_ref, &mut item_render_data);
        self.prefilter_reflection_map_core(graphics_context, texture_sampler_manager, textuer_format, mesh_mut_ref, &mut item_render_data);
    }

    fn prefilter_reflection_map(&mut self, graphics_context: &mut GraphicsContext, texture_sampler_manager: &mut TextureSamplerManager) {
        let texture_format: TextureFormat;
        if let Some(texture) = texture_sampler_manager.get_texture(&self.background_cube_map) {
            texture_format = texture.format;
        } else {
            panic!("Failed to get background cube map texture when prefilter reflection map.");
        }
        let mut cube_mesh: Mesh = Cuboid::default().into();
        // The cube mesh used to rendering (convert equirect to cubemap).
        cube_mesh.upload(graphics_context);
        let mesh_mut_ref = &mut cube_mesh;
        let sub_mesh = &mesh_mut_ref.sub_meshes[0];
        let mut item_render_data = ItemRenderData::new(
            vec![],
            INVALID_PIPELINE_HASH,
            mesh_mut_ref.vertex_buffer,
            Some(mesh_mut_ref.index_buffer),
            mesh_mut_ref.index_data.index_format(),
            sub_mesh.index_start,
            sub_mesh.index_count,
            sub_mesh.base_vertex,
        );

        self.prefilter_reflection_map_core(graphics_context, texture_sampler_manager, texture_format, mesh_mut_ref, &mut item_render_data);
    }
    
    fn prefilter_reflection_map_core(&mut self, graphics_context: &mut GraphicsContext, texture_sampler_manager: &mut TextureSamplerManager, textuer_format: TextureFormat,
        mesh_mut_ref: &mut Mesh, item_render_data: &mut ItemRenderData) {

        let mip_level_count = self.reflection_cube_face_resolution.ilog2() + 1;

        let (reflection_cube_map, depth_attachment, cube_color_attachment_views, cube_depth_attachment_views)
            = Self::create_cube_rt(texture_sampler_manager, self.reflection_cube_face_resolution,
                textuer_format, Self::DEFAULT_DEPTH_ATTACHMENT_FORMAT, mip_level_count);
        // let input_cube_map_sampler = texture_sampler_manager.create_sampler(
        //     AddressMode::ClampToEdge,
        //     AddressMode::ClampToEdge,
        //     AddressMode::ClampToEdge,
        //     FilterMode::Linear,
        //     FilterMode::Linear,
        //     FilterMode::Linear, // reserved for debug. Change this line to Nearest to see the different mip levels.
        // );
        let input_cube_map_sampler = Sampler::default_sampler();
        texture_sampler_manager.generate_mipmaps(&self.background_cube_map);

        self.reflection_cube_map = reflection_cube_map;
        let environmetn_prefilter_shader = Shader::new(
            include_str!("../shaders/wgsl/environmetn_prefilter.wgsl"),
            "environmetn_prefilter".to_owned(),
        );

        let mut bind_group_ids: Vec<BindGroupID> = Vec::new();
        let prefilter_material = Material::new(environmetn_prefilter_shader);
        {
            let mut material_mut_ref = prefilter_material.borrow_mut();
            material_mut_ref.set_texture("input_cube_texture", self.background_cube_map);
            material_mut_ref.set_sampler("cube_sampler", input_cube_map_sampler);
            material_mut_ref.set_float("roughness", 0.0);
            material_mut_ref.render_state.cull_mode = CullMode::Front;
            material_mut_ref.on_update(graphics_context, texture_sampler_manager);

            let render_pipeline_hash = material_mut_ref.hash_value();
            if !graphics_context
                .render_pipelines
                .contains(render_pipeline_hash)
            {
                let depth_format = Self::DEFAULT_DEPTH_ATTACHMENT_FORMAT;
                let vertex_buffer_layout = mesh_mut_ref
                    .vertex_attributes
                    .compute_vertex_buffer_layout();
                graphics_context.render_pipelines.create_render_pipeline(
                    render_pipeline_hash,
                    &material_mut_ref,
                    &[vertex_buffer_layout],
                    &[Some(textuer_format.into())],
                    depth_format,
                );
            }

            bind_group_ids.push(material_mut_ref.get_bind_group());

            item_render_data.bind_group = bind_group_ids;
            item_render_data.render_pipeline = render_pipeline_hash;
        }

        let (mut camera, vp_matrices) = Self::create_cube_camera_info(
            self.reflection_cube_map,
            depth_attachment,
            self.reflection_cube_face_resolution,
        );

        Self::render_cube_rt(
            texture_sampler_manager,
            &mut camera,
            &vp_matrices,
            &cube_color_attachment_views,
            &cube_depth_attachment_views,
            &item_render_data,
            &prefilter_material,
            graphics_context,
            mip_level_count as f32,
            self.reflection_cube_face_resolution as f32,
        );
    }

    fn create_2d_rt(
        texture_sampler_manager: &mut TextureSamplerManager,
        rt_width: u32,
        rt_height: u32,
        color_attachment_format: TextureFormat,
        depth_attachment_format: TextureFormat,
    ) -> (TextureHandle, TextureHandle, TextureView, TextureView) {
        let color_attachment = texture_sampler_manager.create_attachment(
            rt_width,
            rt_height,
            1,
            TextureDimension::D2,
            1,
            color_attachment_format,
        );
        let color_attachment_view = Self::create_2d_texture_view(texture_sampler_manager, &color_attachment, color_attachment_format);
        let depth_attachment = texture_sampler_manager.create_attachment(
            rt_width,
            rt_height,
            1,
            TextureDimension::D2,
            1,
            depth_attachment_format,
        );
        let depth_attachment_view = Self::create_2d_texture_view(texture_sampler_manager, &depth_attachment, depth_attachment_format);
        (color_attachment, depth_attachment, color_attachment_view, depth_attachment_view)
    }

    fn create_2d_texture_view(
        texture_sampler_manager: &mut TextureSamplerManager,
        texture_handle: &TextureHandle,
        texture_format: TextureFormat,
    ) -> TextureView {
        const TEXTURE_USAGE: TextureUsages =
            TextureUsages::RENDER_ATTACHMENT.union(TextureUsages::TEXTURE_BINDING);
        
        if let Some(cube_texture) = texture_sampler_manager.get_texture(texture_handle)
            && let Some(gpu_texture) = &cube_texture.gpu_texture {
            let wgpu_texture_view = gpu_texture.create_view(&TextureViewDescriptor {
                label: Some("Create 2d teture view"),
                dimension: Some(TextureViewDimension::D2),
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
                format: Some(texture_format),
                usage: Some(TEXTURE_USAGE),
            });
            let texture_view = TextureView::from(wgpu_texture_view);
            texture_view
        } else {
            // log::error!("Failed to get 2d texture when create 2d texture views.");
            unreachable!("Failed to get 2d texture when create 2d texture views.");
        }
    }

    fn create_cube_rt(
        texture_sampler_manager: &mut TextureSamplerManager,
        face_size: u32,
        color_attachment_format: TextureFormat,
        depth_attachment_format: TextureFormat,
        mip_level_count: u32,
    ) -> (
        TextureHandle,
        TextureHandle,
        Vec<Vec<TextureView>>,
        Vec<Vec<TextureView>>,
    ) {
        let cube_map = texture_sampler_manager.create_attachment(
            face_size,
            face_size,
            6,
            TextureDimension::D2,
            mip_level_count,
            color_attachment_format,
        );
        let depth_attachment = texture_sampler_manager.create_attachment(
            face_size,
            face_size,
            6,
            TextureDimension::D2,
            mip_level_count,
            depth_attachment_format,
        );

        let cube_color_attachment_views = Self::create_cube_texture_views(
            texture_sampler_manager,
            &cube_map,
            color_attachment_format,
            mip_level_count,
        );
        let cube_depth_attachment_views = Self::create_cube_texture_views(
            texture_sampler_manager,
            &depth_attachment,
            depth_attachment_format,
            mip_level_count,
        );
        (
            cube_map,
            depth_attachment,
            cube_color_attachment_views,
            cube_depth_attachment_views,
        )
    }

    fn create_cube_texture_views(
        texture_sampler_manager: &mut TextureSamplerManager,
        texture_handle: &TextureHandle,
        texture_format: TextureFormat,
        mip_level_count: u32,
    ) -> Vec<Vec<TextureView>> {
        const TEXTURE_USAGE: TextureUsages =
            TextureUsages::RENDER_ATTACHMENT.union(TextureUsages::TEXTURE_BINDING);
        if let Some(cube_texture) = texture_sampler_manager.get_texture(texture_handle)
            && let Some(gpu_texture) = &cube_texture.gpu_texture {
            let views = (0..mip_level_count).map(|base_mip_level| {
                let cube_texture_views = (0..6)
                    .map(|i| {
                        let wgpu_texture_view = gpu_texture.create_view(&TextureViewDescriptor {
                            label: Some(&format!("Create cube teture view {}", i)),
                            dimension: Some(TextureViewDimension::D2),
                            aspect: TextureAspect::All,
                            base_mip_level,
                            mip_level_count: Some(1),
                            base_array_layer: i,
                            array_layer_count: Some(1),
                            format: Some(texture_format),
                            usage: Some(TEXTURE_USAGE),
                        });
                        TextureView::from(wgpu_texture_view)
                    })
                    .collect::<Vec<_>>();
                cube_texture_views
            }).collect::<Vec<_>>();
            views
        } else {
            log::error!("Failed to get cube texture when create cube texture views.");
            vec![vec![]]
        }
    }

    fn create_cube_camera_info(
        color_attachment: TextureHandle,
        depth_attachment: TextureHandle,
        cube_map_face_resolution: u32,
    ) -> (Camera, [Mat4; 6]) {
        let mut camera = Camera::default();
        camera.fov = std::f32::consts::FRAC_PI_2;
        camera.aspect = 1.0;
        camera.near = 0.1;
        camera.far = 10.0;
        camera.clear_color = Some(Color::BLACK);
        camera.color_attachment = color_attachment;
        camera.depth_attachment = depth_attachment;
        camera.physical_view_port = Vec4::new(
            0.0,
            0.0,
            cube_map_face_resolution as f32,
            cube_map_face_resolution as f32,
        );
        camera.logical_view_port = Vec4::new(
            0.0,
            0.0,
            cube_map_face_resolution as f32,
            cube_map_face_resolution as f32,
        );

        let projection_matrix = camera.get_projection_matrix();
        let vp_matrices: [Mat4; 6] = [
            projection_matrix
                * Mat4::look_at_rh(
                    Vec3::ZERO,
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec3::new(0.0, -1.0, 0.0),
                ),
            projection_matrix
                * Mat4::look_at_rh(
                    Vec3::ZERO,
                    Vec3::new(-1.0, 0.0, 0.0),
                    Vec3::new(0.0, -1.0, 0.0),
                ),
            projection_matrix
                * Mat4::look_at_rh(
                    Vec3::ZERO,
                    Vec3::new(0.0, -1.0, 0.0),
                    Vec3::new(0.0, 0.0, -1.0),
                ),
            projection_matrix
                * Mat4::look_at_rh(
                    Vec3::ZERO,
                    Vec3::new(0.0, 1.0, 0.0),
                    Vec3::new(0.0, 0.0, 1.0),
                ),
            projection_matrix
                * Mat4::look_at_rh(
                    Vec3::ZERO,
                    Vec3::new(0.0, 0.0, 1.0),
                    Vec3::new(0.0, -1.0, 0.0),
                ),
            projection_matrix
                * Mat4::look_at_rh(
                    Vec3::ZERO,
                    Vec3::new(0.0, 0.0, -1.0),
                    Vec3::new(0.0, -1.0, 0.0),
                ),
        ];

        (camera, vp_matrices)
    }

    fn render_cube_rt(
        texture_sampler_manager: &mut TextureSamplerManager,
        camera: &mut Camera,
        vp_matrices: &[Mat4; 6],
        cube_color_attachment_views: &Vec<Vec<TextureView>>,
        cube_depth_attachment_views: &Vec<Vec<TextureView>>,
        item_render_data: &ItemRenderData,
        material: &RR<Material>,
        graphics_context: &mut GraphicsContext,
        max_mip_level_count: f32,
        face_resolution: f32,
    ) {
        let mut cur_mip_level = 0.0;
        let mut cur_face_res = face_resolution;
        let is_prefilter_environment_map = max_mip_level_count > 0.0;
        let real_max_mip_level_count = max_mip_level_count.min(5.0);
        cube_color_attachment_views
            .iter()
            .zip(cube_depth_attachment_views.iter())
            .for_each(|(cube_color_attachment_views_per_lod, cube_depth_attachment_views_per_lod)| {
                let roughness = if is_prefilter_environment_map {
                    let roughness = cur_mip_level / (real_max_mip_level_count - 1.0);
                    cur_mip_level += 1.0;
                    camera.physical_view_port.z = cur_face_res;
                    camera.physical_view_port.w = cur_face_res;
                    roughness
                } else {
                    0.0
                };
                if !is_prefilter_environment_map || is_prefilter_environment_map && cur_mip_level < max_mip_level_count {
                    cube_color_attachment_views_per_lod
                        .iter()
                        .zip(cube_depth_attachment_views_per_lod.iter())
                        .zip(vp_matrices.iter())
                        .for_each(
                            |((color_attachment_view, depth_attachment_view), vp_matrix)| {
                                let mut material = material.borrow_mut();
                                material.set_matrix4x4("vp", *vp_matrix);
                                if is_prefilter_environment_map {
                                    material.set_float("roughness", roughness);
                                }
                                material.on_update(graphics_context, texture_sampler_manager);
            
                                RenderAPI::render_item_to_rt_directly(
                                    graphics_context,
                                    camera.clear_color,
                                    camera.physical_view_port,
                                    color_attachment_view,
                                    depth_attachment_view,
                                    &item_render_data,
                                );
                            },
                        );
                }
                if is_prefilter_environment_map {
                    cur_face_res = cur_face_res * 0.5;
                }
        });
    }
}
