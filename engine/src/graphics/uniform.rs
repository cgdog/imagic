use std::u32;

use ahash::AHashMap;

use crate::{
    assets::{
        BuiltinShaderUniformNames, INVALID_TEXTURE_HANDLE, Texture, TextureHandle, TextureSamplerManager, sampler::{INVALID_SAMPLER_HANDLE, SamplerHandle}, shaders::{
            shader::ShaderPropertyPacket,
            shader_property::ShaderPropertyType,
        }
    },
    graphics::{bind_group::{BindGroupID, INVALID_BINDGROUP_ID}, buffer::BufferManager, buffer_view::BufferView, graphics_context::GraphicsContext},
    math::{IVec4, Mat3, Mat4, UVec4, Vec2, Vec3, Vec4, color::Color},
};

/// Flags to indicate which camera builtin uniforms have been synced to GPU in current frame.
/// For optimization, we only sync the changed builtin uniforms once per frame.
pub(crate) struct CameraUniformSyncFlags {
    pub(crate) has_view_matrix_synced: bool,
    pub(crate) has_projection_matrix_synced: bool,
    pub(crate) has_vp_matrix_synced: bool,
    pub(crate) has_v_p_matrices_synced: bool,
    pub(crate) has_camera_position_synced: bool,
}

/// Flags to indicate which global builtin uniforms have been synced to GPU in current frame.
/// For optimization, we only sync the changed builtin uniforms once per frame.
impl CameraUniformSyncFlags {
    pub(crate) fn new() -> Self {
        Self {
            has_view_matrix_synced: false,
            has_projection_matrix_synced: false,
            has_vp_matrix_synced: false,
            has_v_p_matrices_synced: false,
            has_camera_position_synced: false,
        }
    }
}

pub(crate) struct GlobalUniformSyncFlags {
    pub(crate) has_time_synced: bool,
    pub(crate) has_reflection_maps_synced: bool,
}

impl GlobalUniformSyncFlags {
    pub(crate) fn new() -> Self {
        Self {
            has_time_synced: false,
            has_reflection_maps_synced: false,
        }
    }
}

/// Uniform value used in uniform buffer or as texture/sampler.
#[derive(Debug, PartialEq, Clone)]
pub enum UniformValue {
    Float(f32, Option<BufferView>),
    Vec2(Vec2, Option<BufferView>),
    Vec3(Vec3, Option<BufferView>),
    Vec4(Vec4, Option<BufferView>),
    IVec4(IVec4, Option<BufferView>),
    UVec4(UVec4, Option<BufferView>),
    // Color(Vec4),
    Mat3(Mat3, Option<BufferView>),
    Mat4(Mat4, Option<BufferView>),
    Struct(Vec<u8>, Option<BufferView>), // for struct, use Vec<u8> to store data.
    Texture(TextureHandle),
    Sampler(SamplerHandle),
}

impl UniformValue {
    pub fn write(&mut self, graphics_context: &mut GraphicsContext, texture_sampler_manager: &mut TextureSamplerManager) {
        let size = self.size();
        match self {
            UniformValue::Float(value, buffer_view) => {
                Self::write_uniform_buffer(
                    size,
                    &mut graphics_context.buffer_manager,
                    bytemuck::cast_slice(&[*value]),
                    buffer_view,
                );
            }
            UniformValue::Vec2(value, buffer_view) => {
                Self::write_uniform_buffer(
                    size,
                    &mut graphics_context.buffer_manager,
                    bytemuck::cast_slice(value.as_ref()),
                    buffer_view,
                );
            }
            UniformValue::Vec3(value, buffer_view) => {
                Self::write_uniform_buffer(
                    size,
                    &mut graphics_context.buffer_manager,
                    bytemuck::cast_slice(value.as_ref()),
                    buffer_view,
                );
            }
            UniformValue::Vec4(value, buffer_view) => {
                Self::write_uniform_buffer(
                    size,
                    &mut graphics_context.buffer_manager,
                    bytemuck::cast_slice(value.as_ref()),
                    buffer_view,
                );
            }
            UniformValue::UVec4(value, buffer_view) => {
                Self::write_uniform_buffer(
                    size,
                    &mut graphics_context.buffer_manager,
                    bytemuck::cast_slice(value.as_ref()),
                    buffer_view,
                );
            }
            UniformValue::IVec4(value, buffer_view) => {
                Self::write_uniform_buffer(
                    size,
                    &mut graphics_context.buffer_manager,
                    bytemuck::cast_slice(value.as_ref()),
                    buffer_view,
                );
            }
            // MaterialPropertyValue::Color(value) => {
            //     buffer_manager.write_data(buffer_view, bytemuck::cast_slice(value.as_ref()))
            // }
            UniformValue::Mat3(value, buffer_view) => {
                Self::write_uniform_buffer(
                    size,
                    &mut graphics_context.buffer_manager,
                    bytemuck::cast_slice(value.as_ref()),
                    buffer_view,
                );
            }
            UniformValue::Struct(value, buffer_view) => {
                Self::write_uniform_buffer(
                    size,
                    &mut graphics_context.buffer_manager,
                    &value,
                    buffer_view,
                );
            }
            UniformValue::Mat4(value, buffer_view) => {
                Self::write_uniform_buffer(
                    size,
                    &mut graphics_context.buffer_manager,
                    bytemuck::cast_slice(value.as_ref()),
                    buffer_view,
                );
            }
            UniformValue::Texture(handle) => {
                if *handle != INVALID_TEXTURE_HANDLE {
                    texture_sampler_manager.ensure_gpu_texture_valid(handle);
                }
            }
            UniformValue::Sampler(handle) => {
                if *handle != INVALID_TEXTURE_HANDLE
                    && let Some(texture) = texture_sampler_manager.get_sampler_mut(handle)
                {
                    if texture.gpu_sampler.is_none() {
                        texture_sampler_manager.create_gpu_sampler(handle);
                    }
                }
            }
        }
    }

    fn write_uniform_buffer(
        size: u64,
        buffer_manager: &mut BufferManager,
        data: &[u8],
        buffer_view: &mut Option<BufferView>,
    ) {
        if let Some(real_buffer_view) = buffer_view {
            buffer_manager.write_data(real_buffer_view, data);
        } else {
            let new_buffer_view = buffer_manager.allocate_uniform_buffer(size);
            buffer_manager.write_data(&new_buffer_view, data);
            // let new_buffer_view = buffer_manager.allocate_uniform_buffer_init(size, data);
            *buffer_view = Some(new_buffer_view);
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            UniformValue::Float(_value, _) => 4,
            UniformValue::Vec2(_value, _) => 8,
            UniformValue::Vec3(_value, _) => 12,
            UniformValue::Vec4(_value, _) => 16,
            UniformValue::UVec4(_value, _) => 16,
            UniformValue::IVec4(_value, _) => 16,
            // MaterialPropertyValue::Color(_value) => 16,
            UniformValue::Mat3(_value, _) => 48,
            UniformValue::Mat4(_value, _) => 64,
            UniformValue::Struct(_value, _) => _value.len() as u64,
            UniformValue::Texture(_value) => 0,
            UniformValue::Sampler(_value) => 0,
            // _ => 0,
        }
    }
}

/// A uniform used in shader.
#[derive(Debug, PartialEq, Clone)]
pub struct Uniform {
    pub name: String,
    pub is_dirty: bool,
    pub value: UniformValue,
}

impl Uniform {
    pub fn new(name: String, value: UniformValue) -> Self {
        Self {
            name,
            is_dirty: true,
            value,
        }
    }
}

/// A map of uniform name to uniform.
pub type UniformMap = ahash::AHashMap<String, Uniform>;

/// Uniforms belong to a bind group.
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Uniforms {
    pub(crate) uniforms: UniformMap,
    /// The index of bind group in shader.
    pub(crate) bind_group_index: u32,
    /// The indexi of bind group in BindGroupManager.
    pub(crate) bind_group_id: BindGroupID,
    pub(crate) is_dirty: bool,
}

impl Uniforms {
    pub(crate) fn new(shader_property_packet: &ShaderPropertyPacket) -> Self {
        let uniforms =
            Self::construct_uniforms_from_shader(shader_property_packet);
        Self {
            uniforms,
            bind_group_index: shader_property_packet.bind_group_index,
            bind_group_id: INVALID_BINDGROUP_ID,
            is_dirty: true,
        }
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.bind_group_index != u32::MAX
    }

    /// Whether need to create bind group for this uniforms struct.
    pub(crate) fn should_create_bind_group(&self) -> bool {
        self.uniforms.len() > 0 && self.bind_group_id == INVALID_BINDGROUP_ID
    }

    fn construct_uniforms_from_shader(shader_property_packet: &ShaderPropertyPacket) -> UniformMap {
        let mut properties = UniformMap::new();
        for (property_name, shader_property) in &shader_property_packet.properties {
            match shader_property.data_type {
                ShaderPropertyType::Float(_) => {
                    properties.insert(
                        property_name.to_owned(),
                        Uniform::new(property_name.to_owned(), UniformValue::Float(0.0, None)),
                    );
                }
                ShaderPropertyType::Vec2(_) => {
                    properties.insert(
                        property_name.to_owned(),
                        Uniform::new(
                            property_name.to_owned(),
                            UniformValue::Vec2(Vec2::ZERO, None),
                        ),
                    );
                }
                ShaderPropertyType::Vec3(_) => {
                    properties.insert(
                        property_name.to_owned(),
                        Uniform::new(
                            property_name.to_owned(),
                            UniformValue::Vec3(Vec3::ZERO, None),
                        ),
                    );
                }
                ShaderPropertyType::Vec4(_) => {
                    properties.insert(
                        property_name.to_owned(),
                        Uniform::new(
                            property_name.to_owned(),
                            UniformValue::Vec4(Vec4::ZERO, None),
                        ),
                    );
                }
                ShaderPropertyType::UVec4(_) => {
                    // TODO: make material support Vec4I
                    properties.insert(
                        property_name.to_owned(),
                        Uniform::new(
                            property_name.to_owned(),
                            UniformValue::UVec4(UVec4::ZERO, None),
                        ),
                    );
                }
                ShaderPropertyType::IVec4(_) => {
                    // TODO: make material support Vec4I
                    properties.insert(
                        property_name.to_owned(),
                        Uniform::new(
                            property_name.to_owned(),
                            UniformValue::IVec4(IVec4::ZERO, None),
                        ),
                    );
                }
                ShaderPropertyType::Matrix3x3(_) => {
                    properties.insert(
                        property_name.to_owned(),
                        Uniform::new(
                            property_name.to_owned(),
                            UniformValue::Mat3(Mat3::IDENTITY, None),
                        ),
                    );
                }
                ShaderPropertyType::Matrix4x4(_) => {
                    properties.insert(
                        property_name.to_owned(),
                        Uniform::new(
                            property_name.to_owned(),
                            UniformValue::Mat4(Mat4::IDENTITY, None),
                        ),
                    );
                }
                ShaderPropertyType::Struct(Some(size)) => {
                    properties.insert(
                        property_name.to_owned(),
                        Uniform::new(
                            property_name.to_owned(),
                            UniformValue::Struct(vec![0; size.get() as usize], None),
                        ),
                    );
                }
                ShaderPropertyType::Sampler(_comparison) => {
                    properties.insert(
                        property_name.to_owned(),
                        Uniform::new(
                            property_name.to_owned(),
                            UniformValue::Sampler(INVALID_SAMPLER_HANDLE),
                        ),
                    );
                }
                ShaderPropertyType::Image(_dimention, _sample_type, _multiampled) => {
                    properties.insert(
                        property_name.to_owned(),
                        Uniform::new(
                            property_name.to_owned(),
                            UniformValue::Texture(Texture::white()),
                        ),
                    );
                }
                _ => {
                    unimplemented!();
                }
            }
        }
        properties
    }

    pub fn set_float(&mut self, property_name: &str, value: f32) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::Float(f, _) => {
                    *f = value;
                    property_value.is_dirty = true;
                    self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        }
    }

    // TODO: optimize: value use renference.
    pub fn set_vec4f(&mut self, property_name: &str, value: Vec4) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::Vec4(v4, _) => {
                    *v4 = value;
                    property_value.is_dirty = true;
                    self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn get_vec4f(&self, property_name: &str) -> Option<Vec4> {
        if let Some(property_value) = self.uniforms.get(property_name) {
            match &property_value.value {
                UniformValue::Vec4(v4, _) => {
                    return Some(*v4);
                }
                _=> {
                    return None;
                }
            }
        }
        None
    }

    // TODO: optimize: value use renference.
    pub fn set_vec4i(&mut self, property_name: &str, value: IVec4) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::IVec4(v4, _) => {
                    *v4 = value;
                    property_value.is_dirty = true;
                    self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        }
    }

    // TODO: optimize: value use renference.
    pub fn set_vec4u(&mut self, property_name: &str, value: UVec4) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::UVec4(v4, _) => {
                    *v4 = value;
                    property_value.is_dirty = true;
                    self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn set_color(&mut self, property_name: &str, value: Color) {
        // TODO: maybe do some color related operations here.
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::Vec4(color, _) => {
                    *color = value.into();
                    property_value.is_dirty = true;
                    self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn set_matrix3x3(&mut self, property_name: &str, value: Mat3) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::Mat3(mat3, _) => {
                    *mat3 = value;
                    property_value.is_dirty = true;
                    self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        }
    }

    // pub(crate) fn set_matrix3x3_buffer_view(&mut self, property_name: &str, buffer_view: BufferView) {
    //     if let Some(property_value) = self.uniforms.get_mut(property_name) {
    //         match &mut property_value.value {
    //             UniformValue::Mat3(_mat3, bv) => {
    //                 *bv = Some(buffer_view);
    //                 property_value.is_dirty = true;
    //                 self.is_dirty = true;
    //             }
    //             _ => unreachable!(),
    //         }
    //     }
    // }

    pub fn set_matrix4x4(&mut self, property_name: &str, value: Mat4) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::Mat4(mat4, _) => {
                    *mat4 = value;
                    property_value.is_dirty = true;
                    self.is_dirty = true;
                }
                _ => unreachable!("do not find mat4 prop: {}", property_name),
            }
        }
    }

    // pub(crate) fn set_matrix4x4_buffer_view(&mut self, property_name: &str, buffer_view: BufferView) {
    //     if let Some(property_value) = self.uniforms.get_mut(property_name) {
    //         match &mut property_value.value {
    //             UniformValue::Mat4(_mat4, bv) => {
    //                 *bv = Some(buffer_view);
    //                 property_value.is_dirty = true;
    //                 self.is_dirty = true;
    //             }
    //             _ => unreachable!(),
    //         }
    //     }
    // }

    pub fn set_struct(&mut self, property_name: &str, value: Vec<u8>) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::Struct(data, _) => {
                    *data = value;
                    property_value.is_dirty = true;
                    self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn set_texture(&mut self, property_name: &str, value: TextureHandle) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::Texture(tex) => {
                    *tex = value;
                    // property_value.is_dirty = true; // already upload to GPU
                    // self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn get_texture(&self, property_name: &str) -> TextureHandle {
        if let Some(property_value) = self.uniforms.get(property_name) {
            match property_value.value {
                UniformValue::Texture(tex) => {
                    tex
                }
                _ => unreachable!(),
            }
        } else {
            INVALID_TEXTURE_HANDLE
        }
    }

    pub fn set_sampler(&mut self, property_name: &str, value: SamplerHandle) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::Sampler(sampler) => {
                    *sampler = value;
                    // property_value.is_dirty = true;
                    // self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn get_sampler(&self, property_name: &str) -> SamplerHandle {
        if let Some(property_value) = self.uniforms.get(property_name) {
            match property_value.value {
                UniformValue::Sampler(sampler) => {
                    sampler
                }
                _ => unreachable!(),
            }
        } else {
            panic!("sampler {} not found", property_name);
        }
    }

    pub(crate) fn sync_properties(&mut self, graphics_context: &mut GraphicsContext, texture_sampler_manager: &mut TextureSamplerManager) {
        if self.is_dirty {
            for property in self.uniforms.values_mut() {
                if property.is_dirty {
                    property.value.write(graphics_context, texture_sampler_manager);
                    property.is_dirty = false;
                }
            }
            self.is_dirty = false;
        }
    }
}

/// Builtin uniforms, like model matrix, view matrix, projection matrix etc.
/// Multiple bind groups share the same uniform with the same name。
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct BuiltinUniforms {
    /// Builtin uniform name to uniform map.
    pub(crate) uniforms: AHashMap<String, Uniform>,
    /// The key is hash of Builtin uniform names of a bind group.
    /// The value is the bind group id or index in BindGroupManager.
    pub(crate) bind_groups: AHashMap<u64, BindGroupID>,
    pub(crate) is_dirty: bool,
    pub(crate) label: String,
    pub(crate) features: UVec4,
}

impl BuiltinUniforms {
    pub(crate) fn new(label: String) -> Self {
        Self {
            uniforms: AHashMap::new(),
            bind_groups: AHashMap::new(),
            is_dirty: true,
            label,
            features: UVec4::ZERO,
        }
    }

    pub(crate) fn set_matrix4x4(&mut self, property_name: &str, value: Mat4) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::Mat4(mat4, _) => {
                    *mat4 = value;
                    property_value.is_dirty = true;
                    self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        } else {
            self.uniforms.insert(
                property_name.to_string(),
                Uniform::new(
                    property_name.to_string(),
                    UniformValue::Mat4(value, None),
                ),
            );
            self.is_dirty = true;
        }
    }

    pub(crate) fn set_matrix3x3(&mut self, property_name: &str, value: Mat3) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::Mat3(mat3, _) => {
                    *mat3 = value;
                    property_value.is_dirty = true;
                    self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        } else {
            self.uniforms.insert(
                property_name.to_string(),
                Uniform::new(
                    property_name.to_string(),
                    UniformValue::Mat3(value, None),
                ),
            );
            self.is_dirty = true;
        }
    }

    pub(crate) fn set_vec4f(&mut self, property_name: &str, value: Vec4) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::Vec4(v4, _) => {
                    *v4 = value;
                    property_value.is_dirty = true;
                    self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        } else {
            self.uniforms.insert(
                property_name.to_string(),
                Uniform::new(
                    property_name.to_string(),
                    UniformValue::Vec4(value, None),
                ),
            );
            self.is_dirty = true;
        }
    }

    pub(crate) fn set_vec4u(&mut self, property_name: &str, value: UVec4) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::UVec4(v4, _) => {
                    *v4 = value;
                    property_value.is_dirty = true;
                    self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        } else {
            self.uniforms.insert(
                property_name.to_string(),
                Uniform::new(
                    property_name.to_string(),
                    UniformValue::UVec4(value, None),
                ),
            );
            self.is_dirty = true;
        }
    }

    pub(crate) fn set_texture(&mut self, property_name: &str, value: TextureHandle) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::Texture(tex) => {
                    *tex = value;
                    // property_value.is_dirty = true; // already upload to GPU
                    // self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        } else {
            self.uniforms.insert(
                property_name.to_string(),
                Uniform::new(
                    property_name.to_string(),
                    UniformValue::Texture(value),
                ),
            );
        }
    }

    pub(crate) fn set_sampler(&mut self, property_name: &str, value: SamplerHandle) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::Sampler(sampler) => {
                    *sampler = value;
                    // property_value.is_dirty = true;
                    // self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        } else {
            self.uniforms.insert(
                property_name.to_string(),
                Uniform::new(
                    property_name.to_string(),
                    UniformValue::Sampler(value),
                ),
            );
        }
    }

    #[allow(unused)]
    pub(crate) fn set_struct(&mut self, property_name: &str, value: Vec<u8>) {
        if let Some(property_value) = self.uniforms.get_mut(property_name) {
            match &mut property_value.value {
                UniformValue::Struct(data, _) => {
                    *data = value;
                    property_value.is_dirty = true;
                    self.is_dirty = true;
                }
                _ => unreachable!(),
            }
        } else {
            self.uniforms.insert(
                property_name.to_string(),
                Uniform::new(
                    property_name.to_string(),
                    UniformValue::Struct(value, None),
                ),
            );
            self.is_dirty = true;
        }
    }

    /// Enable a global feature by its index.
    ///
    /// At preset, support 128 features.
    ///
    /// Feature index should be defined in your shader code.
    ///
    /// # Parameters
    /// - feature_index: the index of feature to be enabled which is in [0, 128).
    pub fn enable_global_feature(&mut self, feature_index: u32) {
        let array_index = (feature_index >> 5) as usize; // feature_index / 32
        let bit_index = feature_index & 0b11111; // feature_index % 32;
        if array_index < 4 {
            let bit_mask = 1 << bit_index;
            if (self.features[array_index] & bit_mask) == 0 {
                self.features[array_index] |= bit_mask;
                self.set_vec4u(BuiltinShaderUniformNames::_GLOBAL_FEATURES, self.features);
            }
        } else {
            log::warn!("Global eature index {} is out of range.", feature_index);
        }
    }

    /// Disable a global feature by its index.
    ///
    /// At preset, support 128 features.
    ///
    /// Feature index should be defined in your shader code.
    ///
    /// # Parameters
    /// - feature_index: the index of feature to be enabled which is in [0, 128).
    pub fn disable_global_feature(&mut self, feature_index: u32, force_sync: bool) {
        let array_index = (feature_index >> 5) as usize; // feature_index / 32
        let bit_index = feature_index & 0b11111; // feature_index % 32;
        if array_index < 4 {
            let bit_mask = 1 << bit_index;
            if (self.features[array_index] & bit_mask) != 0 {
                self.features[array_index] &= !bit_mask;
                self.set_vec4u(BuiltinShaderUniformNames::_GLOBAL_FEATURES, self.features);
            } else if force_sync {
                self.set_vec4u(BuiltinShaderUniformNames::_GLOBAL_FEATURES, self.features);
            }
        } else {
            log::warn!("Global feature index {} is out of range.", feature_index);
        }
    }

    pub(crate) fn get_bind_group(&mut self, graphics_context: &mut GraphicsContext,
        texture_sampler_manager: &TextureSamplerManager, shader_properties: &ShaderPropertyPacket) -> BindGroupID {
        if !shader_properties.is_valid() {
            return INVALID_BINDGROUP_ID;
        }

        let hash = &shader_properties.hash;
        if let Some(bind_group_id) = self.bind_groups.get(hash) {
            *bind_group_id
        } else {
            let bind_group_id = graphics_context.create_bind_group(
                shader_properties,
                &self.uniforms,
                &format!("Create bind group for builtin uniforms of {}.", self.label),
                texture_sampler_manager,
            );
            self.bind_groups.insert(*hash, bind_group_id);
            bind_group_id
        }
    }

    pub(crate) fn sync_properties(&mut self, graphics_context: &mut GraphicsContext, texture_sampler_manager: &mut TextureSamplerManager) {
        if self.is_dirty {
            for property in self.uniforms.values_mut() {
                if property.is_dirty {
                    property.value.write(graphics_context, texture_sampler_manager);
                    property.is_dirty = false;
                }
            }
            self.is_dirty = false;
        }
    }
}