use gltf::mesh::Mode;

use crate::{
    RR_new,
    assets::{
        AddressMode, BuiltinShaderUniformNames, FilterMode,
        Material, Mesh, SamplerHandle, TextureDimension, TextureFormat,
        TextureHandle, model_loader::ModelLoaderTrait, sub_mesh::SubMesh,
        vertex_attribute::VertexAttributes, vertex_index::IndexData,
    },
    core::{Engine, NodeHandle},
    math::{Color, Vec2, Vec3, Vec4},
    prelude::{MeshRenderer, PolygonMode},
    types::RR,
};

/// A loader for GLTF models.
/// It implements the ModelLoaderTrait to load GLTF models.
pub struct GLTFLoader {}

impl ModelLoaderTrait for GLTFLoader {
    /// Load a GLTF model from the specified path.
    /// Returns a Result containing the loaded Node or an error message.
    /// If sucess, the returned Node have already been added to Current Scene.
    /// # Errors
    /// Returns an error if the GLTF file cannot be opened or processed.
    fn load(
        &self,
        engine: &mut Engine,
        path: &str,
    ) -> Result<NodeHandle, Box<dyn std::error::Error>> {
        let (document, buffers, images) = gltf::import(path)?;
        for gltf_scene in document.scenes() {
            let scene_name = if let Some(scene_name) = gltf_scene.name() {
                scene_name
            } else {
                "UnknownSceneName"
            };
            log::info!("gltf scene name: {:?}", gltf_scene.name());
            if gltf_scene.nodes().len() > 0 {
                let root_node = engine.world.current_scene_mut().create_node(scene_name);
                engine
                    .world.current_scene_mut().add(root_node.clone());
                for node in gltf_scene.nodes() {
                    self.process_node(engine, &document, &node, &root_node, &buffers, &images);
                }
                return Ok(root_node);
            }
        }
        Err(format!("Failed to load model {}", path).to_string().into())
    }
}

impl GLTFLoader {
    /// Create a new GLTFLoader instance.
    pub fn new() -> Self {
        GLTFLoader {}
    }

    fn process_node(
        &self,
        engine: &mut Engine,
        document: &gltf::Document,
        gltf_node: &gltf::Node,
        parent: &NodeHandle,
        buffers: &Vec<gltf::buffer::Data>,
        images: &Vec<gltf::image::Data>,
    ) {
        let transform = gltf_node.transform();
        let (translation, rotation, scale) = transform.decomposed();
        let node_name = if let Some(node_name) = gltf_node.name() {
            node_name
        } else {
            "UnknownName"
        };
        log::info!("  node name: {:?}", node_name);
        let node = engine.world.current_scene_mut().create_node(node_name);
        engine.world.current_scene_mut().attach_to_parent(&node, *parent);
        engine.world.current_scene_mut().get_node_mut_forcely(&node)
            .transform
            .set_position_rotation_scale_from_arrays(translation, rotation, scale);

        if let Some(mesh) = gltf_node.mesh() {
            self.process_mesh(engine, document, &node, mesh, &buffers, &images);
        } else if let Some(camera) = gltf_node.camera() {
            self.process_camera(&node, camera);
        }
        for child_node in gltf_node.children() {
            self.process_node(engine, document, &child_node, &node, buffers, images);
        }
    }

    fn process_mesh(
        &self,
        engine: &mut Engine,
        document: &gltf::Document,
        node: &NodeHandle,
        gltf_mesh: gltf::Mesh,
        buffers: &Vec<gltf::buffer::Data>,
        images: &Vec<gltf::image::Data>,
    ) {
        let mesh_index = gltf_mesh.index();
        let mesh_name = if let Some(mesh_name) = gltf_mesh.name() {
            mesh_name
        } else {
            "UnknowMesh"
        };
        log::info!("mesh name: {}, index: {}", mesh_name, mesh_index);

        let mut vertex_attributes = VertexAttributes::default();
        let mut indices = Vec::<u32>::new();
        let mut sub_meshes = vec![];
        let mut materials: Vec<RR<Material>> = vec![];

        let primitives_count = gltf_mesh.primitives().len();
        log::info!("primitives_count: {}", primitives_count);
        for (primitive_index, primitive) in gltf_mesh.primitives().enumerate() {
            // material info
            let gltf_material = primitive.material();
            gltf_material.index();
            if let Some(material_name) = gltf_material.name() {
                log::info!("material name: {}", material_name);
            }
            let pbr_metallic_roughness = gltf_material.pbr_metallic_roughness();
            let metallic_factor = pbr_metallic_roughness.metallic_factor();
            let roughness_factor = pbr_metallic_roughness.roughness_factor();
            let base_color_factor = pbr_metallic_roughness.base_color_factor();
            let (base_color_texture, albdeo_sampler) = self.get_pbr_texture_and_sampler(
                engine,
                document,
                pbr_metallic_roughness.base_color_texture(),
                images,
                false,
            );
            let metallic_roughness_texture = self.get_pbr_texture(
                engine,
                pbr_metallic_roughness.metallic_roughness_texture(),
                images,
                true,
            );
            let occlusion_texture =
                if let Some(occlusion_texture) = gltf_material.occlusion_texture() {
                    let texture_index = occlusion_texture.texture().index();
                    self.get_texture_by_index(engine, texture_index, images, true)
                } else {
                    TextureHandle::INVALID
                };
            let normal_texture = if let Some(normal_texture) = gltf_material.normal_texture() {
                let texture_index = normal_texture.texture().index();
                self.get_texture_by_index(engine, texture_index, images, true)
            } else {
                TextureHandle::INVALID
            };
            let emissive_factor = gltf_material.emissive_factor();
            let emissive_texture = if let Some(emissive_texture) = gltf_material.emissive_texture()
            {
                let texture_index = emissive_texture.texture().index();
                self.get_texture_by_index(engine, texture_index, images, false)
            } else {
                TextureHandle::INVALID
            };

            let mode = primitive.mode();
            let pbr_shader = engine.shader_manager.get_builtin_pbr_shader();
            let material = Material::new(pbr_shader);
            {
                let mut pbr_material_mut_ref = material.borrow_mut();
                pbr_material_mut_ref.set_albedo_color(Color::from_array(base_color_factor));
                pbr_material_mut_ref.set_vec4f(
                    "_metallic_roughness_ao",
                    Vec4::new(metallic_factor, roughness_factor, 1.0, 1.0),
                );

                pbr_material_mut_ref.render_state.polygon_mode = match mode {
                    Mode::Triangles | Mode::TriangleFan | Mode::TriangleStrip => PolygonMode::Fill,
                    Mode::Points => PolygonMode::Point,
                    Mode::Lines | Mode::LineLoop | Mode::LineStrip => PolygonMode::Line,
                };

                if base_color_texture != TextureHandle::INVALID {
                    pbr_material_mut_ref.set_albedo_map(base_color_texture);
                }
                if albdeo_sampler != SamplerHandle::INVALID {
                    pbr_material_mut_ref.set_sampler(
                        BuiltinShaderUniformNames::_ALBEDO_MAP_SAMPLER,
                        albdeo_sampler,
                    );
                }

                if metallic_roughness_texture != TextureHandle::INVALID {
                    pbr_material_mut_ref.set_metallic_roughness_map(metallic_roughness_texture);
                }

                if occlusion_texture != TextureHandle::INVALID {
                    pbr_material_mut_ref.set_ao_map(occlusion_texture);
                }

                if normal_texture != TextureHandle::INVALID {
                    pbr_material_mut_ref.set_normal_map(normal_texture);
                }

                pbr_material_mut_ref.set_emissive_color(Color::rgb(
                    emissive_factor[0],
                    emissive_factor[1],
                    emissive_factor[2],
                ));
                if emissive_texture != TextureHandle::INVALID {
                    pbr_material_mut_ref.set_emissive_map(emissive_texture);
                }
            }
            materials.push(material);

            if primitive_index == 0 {
                // at preset, only supports submeshes sharing one same vertex buffer.
                // TODO: support submesh with differernt vertex buffers.
                self.process_primitive_geometry(
                    primitive,
                    &mut vertex_attributes,
                    &mut indices,
                    &mut sub_meshes,
                    buffers,
                );
            }
        }

        let index_data = IndexData::new_u32(indices);

        let mesh = Mesh::new(vertex_attributes, index_data, sub_meshes);
        let mesh_render = MeshRenderer::new(RR_new!(mesh), materials);
        engine.world.current_scene_mut().add_component::<MeshRenderer>(node, mesh_render);
    }

    fn process_primitive_geometry(
        &self,
        primitive: gltf::Primitive,
        vertex_attributes: &mut VertexAttributes,
        indices: &mut Vec<u32>,
        sub_meshes: &mut Vec<SubMesh>,
        buffers: &Vec<gltf::buffer::Data>,
    ) {
        let sub_mesh_vertex_start = vertex_attributes.position.len() as u32;
        // geometry info
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        if let Some(positions) = reader.read_positions() {
            // process positions
            vertex_attributes.position.reserve_exact(positions.len());
            for position in positions {
                vertex_attributes.position.push(Vec3::from_array(position));
            }
        }

        if let Some(normals) = reader.read_normals() {
            // process normals
            vertex_attributes.normal.reserve_exact(normals.len());
            for normal in normals {
                vertex_attributes.normal.push(Vec3::from_array(normal));
            }
        }

        if let Some(tangents) = reader.read_tangents() {
            // process tangents
            vertex_attributes.tangent.reserve_exact(tangents.len());
            for tangent in tangents {
                vertex_attributes.tangent.push(Vec4::from_array(tangent));
            }
        }

        if let Some(colors) = reader.read_colors(0) {
            // process colors
            let colors = colors.into_rgba_f32();
            vertex_attributes.color.reserve_exact(colors.len());
            for color in colors {
                vertex_attributes.color.push(Color::from_array(color));
            }
        }

        if let Some(tex_coords) = reader.read_tex_coords(0) {
            // process texture coordinates
            let uv0s = tex_coords.into_f32();
            vertex_attributes.uv.reserve_exact(uv0s.len());
            for uv0 in uv0s {
                vertex_attributes.uv.push(Vec2::from_array(uv0));
            }
        }

        if let Some(joints) = reader.read_joints(0) {
            // process joints
            for joint in joints.into_u16() {
                log::info!("joint: {:?}", joint);
            }
        }

        if let Some(weights) = reader.read_weights(0) {
            // process weights
            for weight in weights.into_f32() {
                log::info!("weight: {:?}", weight);
            }
        }

        for (morph_taregt_index, (positions, normals, tangents)) in
            reader.read_morph_targets().enumerate()
        {
            log::info!("morph target {}", morph_taregt_index);
            if let Some(position_iter) = positions {
                for _pos in position_iter {
                    // TODO: process morph positions
                }
            }
            if let Some(normal_iter) = normals {
                for _normal in normal_iter {
                    // TODO: process morph normals
                }
            }
            if let Some(tanget_iter) = tangents {
                for _tangent in tanget_iter {
                    // TODO: process morph tangents
                }
            }
        }

        let sub_mesh_index_start = indices.len() as u32;
        if let Some(indices_data) = reader.read_indices() {
            // process indices
            let indices_iter = indices_data.into_u32();
            for index in indices_iter {
                indices.push(index);
            }
        }
        let sub_mesh_index_count = indices.len() as u32 - sub_mesh_index_start;

        let sub_mesh = SubMesh::new(
            sub_mesh_index_start,
            sub_mesh_index_count,
            sub_mesh_vertex_start,
        );
        sub_meshes.push(sub_mesh);
    }

    fn process_camera(&self, _node: &NodeHandle, camera: gltf::Camera) {
        let camera_index = camera.index();
        let camera_name = if let Some(camera_name) = camera.name() {
            camera_name
        } else {
            "UnknownCamera"
        };
        log::info!("Camera name: {}, index: {}", camera_name, camera_index);
    }

    fn convert_texture_data_format(
        format: gltf::image::Format,
        data: &Vec<u8>,
        is_linear: bool,
    ) -> (Vec<u8>, TextureFormat) {
        match format {
            gltf::image::Format::R8 => (data.clone(), TextureFormat::R8Unorm),
            gltf::image::Format::R8G8 => (data.clone(), TextureFormat::Rg8Unorm),
            gltf::image::Format::R8G8B8 => {
                let format = if is_linear {
                    TextureFormat::Rgba8Unorm
                } else {
                    TextureFormat::Rgba8UnormSrgb
                };
                (Self::convert_rgb8_to_rgba8(data), format) // need convert?
            }
            gltf::image::Format::R8G8B8A8 => {
                let format = if is_linear {
                    TextureFormat::Rgba8Unorm
                } else {
                    TextureFormat::Rgba8UnormSrgb
                };
                (data.clone(), format)
            }
            gltf::image::Format::R16 => (data.clone(), TextureFormat::R16Float),
            gltf::image::Format::R16G16 => (data.clone(), TextureFormat::Rg16Float),
            gltf::image::Format::R16G16B16 => (data.clone(), TextureFormat::Rgba16Float), // need convert?
            gltf::image::Format::R16G16B16A16 => (data.clone(), TextureFormat::Rgba16Float),
            gltf::image::Format::R32G32B32FLOAT => (data.clone(), TextureFormat::Rgba32Float), // need convert?
            gltf::image::Format::R32G32B32A32FLOAT => (data.clone(), TextureFormat::Rgba32Float),
        }
    }

    fn convert_rgb8_to_rgba8(rgb_data: &Vec<u8>) -> Vec<u8> {
        let mut rgba_data = Vec::with_capacity(rgb_data.len() / 3 * 4);

        for chunk in rgb_data.chunks(3) {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];

            rgba_data.push(r);
            rgba_data.push(g);
            rgba_data.push(b);
            rgba_data.push(255);
        }

        rgba_data
    }

    fn get_pbr_texture(
        &self,
        engine: &mut Engine,
        texture_info: Option<gltf::texture::Info>,
        images: &Vec<gltf::image::Data>,
        is_linear: bool,
    ) -> TextureHandle {
        if let Some(texture_info) = texture_info {
            let texture_index = texture_info.texture().index();
            self.get_texture_by_index(engine, texture_index, images, is_linear)
        } else {
            TextureHandle::INVALID
        }
    }

    fn get_pbr_texture_and_sampler(
        &self,
        engine: &mut Engine,
        document: &gltf::Document,
        texture_info: Option<gltf::texture::Info>,
        images: &Vec<gltf::image::Data>,
        is_linear: bool,
    ) -> (TextureHandle, SamplerHandle) {
        if let Some(texture_info) = texture_info {
            let texture_index = texture_info.texture().index();
            (
                self.get_texture_by_index(engine, texture_index, images, is_linear),
                self.get_sampler(engine, document, texture_index),
            )
        } else {
            (TextureHandle::INVALID, SamplerHandle::INVALID)
        }
    }

    fn get_texture_by_index(
        &self,
        engine: &mut Engine,
        texture_index: usize,
        images: &Vec<gltf::image::Data>,
        is_linear: bool,
    ) -> TextureHandle {
        let texture_data = &images[texture_index];
        let (pixel_data, format) =
            Self::convert_texture_data_format(texture_data.format, &texture_data.pixels, is_linear);
        let texture = engine
            .texture_sampler_manager
            .create_texture_from_raw_bytes(
                vec![pixel_data],
                TextureDimension::D2,
                texture_data.width,
                texture_data.height,
                1,
                format,
                false,
                true,
            );
        texture
    }

    fn get_sampler(
        &self,
        engine: &mut Engine,
        document: &gltf::Document,
        texture_index: usize,
    ) -> SamplerHandle {
        let texture = document.textures().nth(texture_index);
        if let Some(texture) = texture {
            let sampler = texture.sampler();
            let wrap_u = Self::get_sampler_wrapping_mode(sampler.wrap_s());
            let wrap_v = Self::get_sampler_wrapping_mode(sampler.wrap_t());
            let wrap_w = wrap_u;
            let mag_filter_mode = if let Some(mag_filter) = sampler.mag_filter() {
                match mag_filter {
                    gltf::texture::MagFilter::Nearest => FilterMode::Nearest,
                    gltf::texture::MagFilter::Linear => FilterMode::Linear,
                }
            } else {
                FilterMode::Linear
            };
            let (min_filter_mode, mip_filter) = if let Some(min_filter) = sampler.min_filter() {
                match min_filter {
                    gltf::texture::MinFilter::Nearest => (FilterMode::Nearest, FilterMode::Nearest),
                    gltf::texture::MinFilter::Linear => (FilterMode::Linear, FilterMode::Nearest),
                    gltf::texture::MinFilter::NearestMipmapNearest => {
                        (FilterMode::Nearest, FilterMode::Nearest)
                    }
                    gltf::texture::MinFilter::LinearMipmapNearest => {
                        (FilterMode::Linear, FilterMode::Nearest)
                    }
                    gltf::texture::MinFilter::NearestMipmapLinear => {
                        (FilterMode::Nearest, FilterMode::Linear)
                    }
                    gltf::texture::MinFilter::LinearMipmapLinear => {
                        (FilterMode::Linear, FilterMode::Linear)
                    }
                }
            } else {
                (FilterMode::Linear, FilterMode::Linear)
            };
            let sampler = engine.texture_sampler_manager.create_sampler(
                wrap_u,
                wrap_v,
                wrap_w,
                mag_filter_mode,
                min_filter_mode,
                mip_filter,
            );
            sampler
        } else {
            SamplerHandle::INVALID
        }
    }

    fn get_sampler_wrapping_mode(gltf_wrapping_mode: gltf::texture::WrappingMode) -> AddressMode {
        match gltf_wrapping_mode {
            gltf::texture::WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
            gltf::texture::WrappingMode::MirroredRepeat => AddressMode::MirrorRepeat,
            gltf::texture::WrappingMode::Repeat => AddressMode::Repeat,
        }
    }
}
