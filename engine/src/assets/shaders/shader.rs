use std::{borrow::Cow, hash::{Hash, Hasher}};

use ahash::AHasher;
use log::info;
use wgpu::{BindGroupLayout, BindGroupLayoutEntry, ShaderModule, naga::{AddressSpace, StorageAccess}};

use crate::{
    assets::{
        asset::IAsset,
        shaders::{
            module_parser::parse_shader_module,
            shader_property::{ShaderProperty, ShaderPropertyType},
        },
    },
    graphics::graphics_context::GraphicsContext,
};

/// Map of shader property name to ShaderProperty.
pub(crate) type ShaderPropertyMap = ahash::AHashMap<String, ShaderProperty>;

#[derive(Clone)]
pub(crate) struct ShaderPropertyPacket {
    pub(crate) properties: ShaderPropertyMap,
    /// bind group index in shader.
    pub(crate) bind_group_index: u32,
    pub(crate) bind_group_layout: Option<BindGroupLayout>,
    pub(crate) hash: u64,
}

impl ShaderPropertyPacket {
    pub(crate) fn new() -> Self {
        Self {
            properties: ahash::AHashMap::new(),
            bind_group_index: u32::MAX,
            bind_group_layout: None,
            hash: u64::MAX,
        }
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.bind_group_index != u32::MAX
    }

    pub(crate) fn insert(&mut self, name: String, shader_property: ShaderProperty) {
        if self.bind_group_index == u32::MAX {
            self.bind_group_index = shader_property.binding.group;
        } else {
            assert_eq!(
                self.bind_group_index, shader_property.binding.group,
                "All properties in a ShaderPropertyPacket must belong to the same bind group, name: {}", name
            );
        }
        self.properties.insert(name, shader_property);
    }

    pub(crate) fn create_bind_group_layout(&mut self, graphics_context: &mut GraphicsContext) {
        if self.properties.is_empty() {
            return;
        }
        assert_ne!(
            self.bind_group_index,
            u32::MAX,
            "Invalid bind group index when createing bind group layout"
        );
        let mut bind_group_layout_entries = Vec::<BindGroupLayoutEntry>::new();
        let mut hasher = AHasher::default();
        for property in self.properties.values() {
            property.hash(&mut hasher);
            let binding = property.binding.binding;
            let visibility = property.visibility;
            // let min_binding_size = property.min_binding_size;
            match &property.data_type {
                ShaderPropertyType::Float(min_binding_size)
                | ShaderPropertyType::Vec2(min_binding_size)
                | ShaderPropertyType::Vec3(min_binding_size)
                | ShaderPropertyType::Vec4(min_binding_size)
                | ShaderPropertyType::UVec4(min_binding_size)
                | ShaderPropertyType::IVec4(min_binding_size)
                | ShaderPropertyType::Matrix3x3(min_binding_size)
                | ShaderPropertyType::Matrix4x4(min_binding_size)
                | ShaderPropertyType::Struct(min_binding_size) => {
                    // uniform or storage buffer
                    let buffer_binding_type = match property.space {
                        AddressSpace::Storage{access} => {
                            let is_readonly = access == StorageAccess::LOAD;
                            wgpu::BufferBindingType::Storage { read_only: is_readonly }
                        }
                        _ => wgpu::BufferBindingType::Uniform,
                    };
                    let bind_group_layout_entry = wgpu::BindGroupLayoutEntry {
                        binding,
                        visibility,
                        ty: wgpu::BindingType::Buffer {
                            ty: buffer_binding_type,
                            has_dynamic_offset: false,
                            min_binding_size: *min_binding_size,
                        },
                        count: None,
                    };
                    bind_group_layout_entries.push(bind_group_layout_entry);
                }
                ShaderPropertyType::Image(view_dimension, sample_type, multisampled) => {
                    let bind_group_layout_entry = wgpu::BindGroupLayoutEntry {
                        // albedo map
                        binding,
                        visibility,
                        ty: wgpu::BindingType::Texture {
                            multisampled: *multisampled,
                            view_dimension: *view_dimension,
                            sample_type: *sample_type,
                        },
                        count: None,
                    };
                    bind_group_layout_entries.push(bind_group_layout_entry);
                }
                ShaderPropertyType::Sampler(_comparison) => {
                    let bind_group_layout_entry = wgpu::BindGroupLayoutEntry {
                        // sampler
                        binding,
                        visibility,
                        // This should match the filterable field of the
                        // corresponding Texture entry.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    };
                    bind_group_layout_entries.push(bind_group_layout_entry);
                }
            }
        }
        self.hash = hasher.finish();

        let bind_group_layout =
            graphics_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &bind_group_layout_entries,
                    label: Some(&format!(
                        "create bind group layout of bind group {}",
                        self.bind_group_index
                    )),
                });
        self.bind_group_layout = Some(bind_group_layout);
    }
}

/// Builtin uniform properties flags.
#[derive(PartialEq, Clone)]
pub(crate) struct BuilinUniformFlags {
    pub(crate) has_model_matrix: bool,
    pub(crate) has_view_matrix: bool,
    pub(crate) has_projection_matrix: bool,
    pub(crate) has_vp_matrix: bool,
    pub(crate) has_v_p_matrices: bool,
    pub(crate) has_mv_matrix: bool,
    pub(crate) has_mvp_matrix: bool,
    pub(crate) has_normal_matrix: bool,
    pub(crate) has_m_v_p_matrices: bool,
    pub(crate) has_m_v_p_n_matrices: bool,
    pub(crate) has_camera_position: bool,
    pub(crate) has_time: bool,
    pub(crate) has_sh: bool,
    pub(crate) has_irradiance_cube_map: bool,
    pub(crate) has_reflection_cube_map: bool,
    pub(crate) has_reflection_cube_sampler: bool,
    pub(crate) has_brdf_lut: bool,
    pub(crate) has_lights: bool,
}

impl BuilinUniformFlags {
    pub(crate) fn new() -> Self {
        Self {
            has_model_matrix: false,
            has_view_matrix: false,
            has_projection_matrix: false,
            has_vp_matrix: false,
            has_v_p_matrices: false,
            has_mv_matrix: false,
            has_mvp_matrix: false,
            has_normal_matrix: false,
            has_m_v_p_matrices: false,
            has_m_v_p_n_matrices: false,
            has_camera_position: false,
            has_time: false,
            has_sh: false,
            has_irradiance_cube_map: false,
            has_reflection_cube_map: false,
            has_reflection_cube_sampler: false,
            has_brdf_lut: false,
            has_lights: false,
        }
    }

    pub(crate) fn has_environment_reflection_info(&self) -> bool {
        self.has_sh || self.has_reflection_cube_map || self.has_reflection_cube_sampler || self.has_brdf_lut || self.has_irradiance_cube_map
    }
}

#[derive(Clone)]
pub(crate) struct ShaderProperties {
    pub(crate) per_material_properties: ShaderPropertyPacket,
    pub(crate) per_object_properties: ShaderPropertyPacket,
    pub(crate) per_camera_properties: ShaderPropertyPacket,
    pub(crate) per_scene_properties: ShaderPropertyPacket, // e.g, lights, time, fog
}

impl ShaderProperties {
    pub fn new() -> Self {
        Self {
            per_material_properties: ShaderPropertyPacket::new(),
            per_object_properties: ShaderPropertyPacket::new(),
            per_camera_properties: ShaderPropertyPacket::new(),
            per_scene_properties: ShaderPropertyPacket::new(),
        }
    }

    pub(crate) fn create_bind_group_layout(&mut self, graphics_context: &mut GraphicsContext) {
        self.per_material_properties
            .create_bind_group_layout(graphics_context);
        self.per_object_properties
            .create_bind_group_layout(graphics_context);
        self.per_camera_properties
            .create_bind_group_layout(graphics_context);
        self.per_scene_properties
            .create_bind_group_layout(graphics_context);
    }

    /// Get all bind group layouts used in this shader properties, sorted by bind group index.
    /// It will be used to create pipeline layout.
    pub(crate) fn get_bind_group_layouts(&self) -> Vec<&BindGroupLayout> {
        let mut shader_property_packets = Vec::new();
        if self.per_material_properties.bind_group_index != u32::MAX {
            shader_property_packets.push(&self.per_material_properties);
        }
        if self.per_object_properties.bind_group_index != u32::MAX {
            shader_property_packets.push(&self.per_object_properties);
        }
        if self.per_camera_properties.bind_group_index != u32::MAX {
            shader_property_packets.push(&self.per_camera_properties);
        }
        if self.per_scene_properties.bind_group_index != u32::MAX {
            shader_property_packets.push(&self.per_scene_properties);
        }
        shader_property_packets.sort_by(|a, b| a.bind_group_index.cmp(&b.bind_group_index));

        let bind_group_layouts = shader_property_packets.iter().map(|p| {
            p.bind_group_layout.as_ref().expect("No bind group layout")
        }).collect::<Vec<&BindGroupLayout>>();
        bind_group_layouts
    }
}

/// Shader tag used to declare ShaderHandle.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ShaderTag {}

/// Handle type for Shader.
pub type ShaderHandle = crate::types::Handle<ShaderTag>;

/// Representation of a shader asset and instance.
#[derive(Clone)]
pub struct Shader {
    /// The shader module.
    pub(crate) shader_module: Option<wgpu::ShaderModule>,
    /// The name of the shader.
    pub name: String,
    /// The properties defined in the shader.
    pub(crate) shader_properties: ShaderProperties,
    /// Has builtin uniforms in this shader?
    pub(crate) builtin_uniform_flags: BuilinUniformFlags,
    /// Unique hash of the shader source code.
    pub hash: u64,
    /// Whether the shader has been initialized.
    pub(crate) is_inited: bool,
    /// The Naga module.
    pub(crate) naga_module: Option<wgpu::naga::Module>,
}

impl PartialEq for Shader {
    fn eq(&self, other: &Self) -> bool {
        // self.shader_module == other.shader_module
        // && self.source == other.source
        // && self.name == other.name
        // && self.properties == other.properties
        self.hash == other.hash
            // && self.max_bind_group == other.max_bind_group
            // && self.bind_group_layouts == other.bind_group_layouts
            && self.is_inited == other.is_inited
    }
}

impl IAsset for Shader {}

impl Shader {
    pub(crate) fn new(source: &str, name: String) -> Self {
        let mut hasher = AHasher::default();
        let (shader_properties, _max_bind_group, naga_module, builtin_uniform_flags) =
            parse_shader_module(&source, &mut hasher);
        let hash = hasher.finish();
        info!("shader name: {}, hash: {}", name, hash);
        let shader = Self {
            shader_module: None,
            // source,
            name,
            shader_properties,
            builtin_uniform_flags,
            hash,
            // bind_group_layouts: Vec::<BindGroupLayout>::new(),
            is_inited: false,
            naga_module: Some(naga_module),
        };
        shader
    }

    pub(crate) fn init(&mut self, graphics_context: &mut GraphicsContext) {
        self.create_bind_group_layout(graphics_context);
        // info!("shader BindGroupLayouts: {:?}", self.bind_group_layouts);
        self.compile(graphics_context);
    }

    /// Get all bind group layouts used in this shader, sorted by bind group index.
    /// It will be used to create pipeline layout.
    pub(crate) fn get_bind_group_layouts(&self) -> Vec<&BindGroupLayout> {
        self.shader_properties.get_bind_group_layouts()
    }

    fn create_bind_group_layout(&mut self, graphics_context: &mut GraphicsContext) {
        self.shader_properties
            .create_bind_group_layout(graphics_context);
    }

    pub(crate) fn compile(&mut self, graphics_context: &mut GraphicsContext) {
        if self.shader_module.is_none() {
            let naga_module = self.naga_module.take().unwrap();
            let shader_module =
                graphics_context
                    .device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("create shader module. for {$self.name}"),
                        // source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(self.source.as_str())),
                        source: wgpu::ShaderSource::Naga(Cow::Owned(naga_module)),
                    });
            info!(
                "compile shader module of {}: {:?}",
                &self.name, shader_module
            );
            // for (var_handle, var) in shader_module.
            self.shader_module = Some(shader_module);
        }
    }

    pub(crate) fn get_shader_module(&self) -> &ShaderModule {
        self.shader_module.as_ref().expect("No shader module")
    }
}
