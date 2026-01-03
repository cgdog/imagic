use std::{hash::{Hash, Hasher}, num::NonZero};

use wgpu::{ShaderStages, naga::AddressSpace};

use crate::assets::shaders::shader::BuilinUniformFlags;

#[derive(PartialEq, Clone)]
pub enum ShaderPropertyType {
    Float(Option<NonZero<u64>>),
    Vec2(Option<NonZero<u64>>),
    Vec3(Option<NonZero<u64>>),
    Vec4(Option<NonZero<u64>>),
    UVec4(Option<NonZero<u64>>),
    IVec4(Option<NonZero<u64>>),
    Matrix3x3(Option<NonZero<u64>>),
    Matrix4x4(Option<NonZero<u64>>),
    Struct(Option<NonZero<u64>>),
    Image(wgpu::TextureViewDimension, wgpu::TextureSampleType, bool),// (view_dimension, type, multisampled)
    Sampler(bool), // comparison
}

impl Hash for ShaderPropertyType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

#[derive(PartialEq, Clone, Hash)]
pub struct ShaderPropertyBinding {
    pub group: u32,
    pub binding: u32,
}

impl ShaderPropertyBinding {
    pub fn new(group: u32, binding: u32) -> Self {
        Self {
            group,
            binding
        }
    }
}

impl From<wgpu::naga::ResourceBinding> for ShaderPropertyBinding {
    fn from(value: wgpu::naga::ResourceBinding) -> Self {
        Self {
            group: value.group,
            binding: value.binding,
        }
    }
}

impl From<ShaderPropertyBinding> for wgpu::naga::ResourceBinding {
    fn from(value: ShaderPropertyBinding) -> Self {
        Self {
            group: value.group,
            binding: value.binding,
        }
    }
}

/// A property of a shader, such as a uniform, storage buffer, texture, or sampler.
#[derive(PartialEq, Clone, Hash)]
pub(crate) struct ShaderProperty {
    /// The name of the property, i.e. the variable name in the shader.
    pub name: String,
    /// The binding of the property, i.e. the group and binding number in the shader.
    pub binding: ShaderPropertyBinding,
    /// Visible in which shader stages
    pub(crate) visibility: ShaderStages,
    /// The data type of the property.
    pub data_type: ShaderPropertyType,
    /// The address space of the property, i.e. uniform, storage, or function.
    pub(crate) space: AddressSpace,
}

impl ShaderProperty {
    pub fn new(name: String, binding: ShaderPropertyBinding, data_type: ShaderPropertyType, stages: ShaderStages, space: AddressSpace) -> Self {
        Self {
            name,
            binding,
            data_type,
            visibility: stages,
            space,
        }
    }
}

/// The names of builtin shader uniforms, which will be upload to shader automatically.
pub struct BuiltinShaderUniformNames {
}

impl BuiltinShaderUniformNames {
    // {{begine per object uniforms
    /// The uniform name of transform matrix used to transform object from local space to world space.
    pub const _MODEL_MATRIX: &'static str = "_model_matrix";
    /// The uniform name of the combined Model View matrix
    pub const _MV_MATRIX: &'static str = "_mv_matrix";
    /// The uniform name of the combined Model View Projection matrix
    pub const _MVP_MATRIX: &'static str = "_mvp_matrix";
    /// The uniform name of normal matrix, which is the inverse transpose of the upper-left 3x3 of model matrix.
    pub const _NORMAL_MATRIX: &'static str = "_normal_matrix";
    /// The struct uniform name of transform matrices, which contains model, view, projection matrices.
    pub const _M_V_P_MATRICES: &'static str = "_m_v_p_matrices";
    /// The struct uniform name of transform matrices, which contains model, view, projection, normal matrices.
    pub const _M_V_P_N_MATRICES: &'static str = "_m_v_p_n_matrices";
    // end per object uniforms}}

    // {{begin per camera uniforms
    /// The uniform name of view matrix of current camera.
    pub const _VIEW_MATRIX: &'static str = "_view_matrix";
    /// The uniform name of projection matrix of current camera.
    pub const _PROJECTION_MATRIX: &'static str = "_projection_matrix";
    /// The uniform name of the combined Vew Projection matrix of current camera
    pub const _VP_MATRIX: &'static str = "_vp_matrix";
    /// The struct uniform name, which contains two matrices: View Matrix & Projection Matrix.
    pub const _V_P_MATRICES: &'static str = "_v_p_matrices";
    /// The uniform name of camera position in world space.
    pub const _CAMERA_POSITION: &'static str = "_camera_position";
    // end per camera uniforms}}

    // {{begin per scene uniforms
    /// x: time since started, y: delta time, z: scaled delta time, w: sin(time)
    pub const _TIME: &'static str = "_time";
    /// The uniform name of the irradiance cube map. Deprecated. Use _SH instead.
    pub const _IRRADIANCE_CUBE_MAP: &'static str = "_irradiance_cube_map";
    /// The uniform name of the prefiltered reflection cube map.
    pub const _REFLECTION_CUBE_MAP: &'static str = "_prefiltered_reflection_map";
    /// The uniform name of the sampler of the prefiltered reflection cube map.
    pub const _REFLECTION_CUBE_SAMPLER: &'static str = "_reflection_cube_sampler";
    /// The uniform name of the BRDF lookup texture.
    pub const _BRDF_LUT: &'static str = "_brdf_lut";
    /// The uniform name of the SH coefficients of the scene.
    pub const _SH: &'static str = "_sh";
    /// The uniform name of the lighting infos.
    pub const _LIGHTING_INFOS: &'static str = "_lighting_infos";
    // end per scene uniforms}}

    /// The uniform name of the albedo color.
    pub const _ALBEDO_COLOR: &'static str = "_albedo_color";
    /// The uniform name of the albedo map.
    pub const _ALBEDO_MAP: &'static str = "_albedo_map";
    /// The uniform name of the sampler of the albedo map.
    pub const _ALBEDO_MAP_SAMPLER: &'static str = "_albedo_map_sampler";
    /// The uniform name of the metallic factor.
    pub const _METALLIC: &'static str = "_metallic";
    /// The uniform name of the roughness factor.
    pub const _ROUGHNESS: &'static str = "_roughness";
    /// The uniform name of the metallic roughness ao.
    pub const _METALLIC_ROUGHNESS_AO: &'static str = "_metallic_roughness_ao";
    /// The uniform name of the metallic roughness map.
    pub const _METALLIC_ROUGHNESS_MAP: &'static str = "_metallic_roughness_map";
    /// The uniform name of the ao map.
    pub const _AO_MAP: &'static str = "_ao_map";
    /// The uniform name of normal map.
    pub const _NORMAL_MAP: &'static str = "_normal_map";
    /// The uniform name of emissive map.
    pub const _EMISSIVE_MAP: &'static str = "_emissive_map";
    /// The uniform name of emissive color.
    pub const _EMISSIVE_COLOR: &'static str = "_emissive_color";

    /// The per-material features uniform, which is a uvec4 to support 128 features (one feature one bit).
    /// Each bit in the uvec4 represents a feature, 1 for enabled, 0 for disabled.
    pub const _MATERIAL_FEATURES: &'static str = "_material_features";

    /// The global or per-scene features uniform.
    pub const _GLOBAL_FEATURES: &'static str = "_global_features";

    pub(crate) fn is_per_object_uniform(name: &str, builtin_uniform_flags: &mut BuilinUniformFlags) -> bool {
        match name {
            Self::_MODEL_MATRIX => {
                builtin_uniform_flags.has_model_matrix = true;
                true
            }
            Self::_NORMAL_MATRIX => {
                builtin_uniform_flags.has_normal_matrix = true;
                true
            }
            Self::_MV_MATRIX => {
                builtin_uniform_flags.has_mv_matrix = true;
                true
            }
            Self::_MVP_MATRIX => {
                builtin_uniform_flags.has_mvp_matrix = true;
                true
            }
            Self::_M_V_P_MATRICES => {
                builtin_uniform_flags.has_m_v_p_matrices = true;
                true
            }
            Self::_M_V_P_N_MATRICES => {
                builtin_uniform_flags.has_m_v_p_n_matrices = true;
                true
            },
            _ => false,
        }
    }

    pub(crate) fn is_per_camera_uniform(name: &str, builtin_uniform_flags: &mut BuilinUniformFlags) -> bool {
        match name {
            Self::_VIEW_MATRIX => {
                builtin_uniform_flags.has_view_matrix = true;
                true
            }
            Self::_PROJECTION_MATRIX => {
                builtin_uniform_flags.has_projection_matrix = true;
                true
            }
            Self::_VP_MATRIX => {
                builtin_uniform_flags.has_vp_matrix = true;
                true
            }
            Self::_V_P_MATRICES => {
                builtin_uniform_flags.has_v_p_matrices = true;
                true
            }
            Self::_CAMERA_POSITION => {
                builtin_uniform_flags.has_camera_position = true;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn is_per_scene_uniform(name: &str, builtin_uniform_flags: &mut BuilinUniformFlags) -> bool {
        match name {
            Self::_TIME=> {
                builtin_uniform_flags.has_time = true;
                true
            }
            Self::_IRRADIANCE_CUBE_MAP => {
                builtin_uniform_flags.has_irradiance_cube_map = true;
                true
            }
            Self::_REFLECTION_CUBE_MAP => {
                builtin_uniform_flags.has_reflection_cube_map = true;
                true
            }
            Self::_REFLECTION_CUBE_SAMPLER => {
                builtin_uniform_flags.has_reflection_cube_sampler = true;
                true
            }
            Self::_BRDF_LUT => {
                builtin_uniform_flags.has_brdf_lut = true;
                true
            }
            Self::_SH => {
                true
            }
            Self::_GLOBAL_FEATURES => {
                true
            }
            Self::_LIGHTING_INFOS => {
                builtin_uniform_flags.has_lights = true;
                true
            }
            _ => false,
        }
    }
}

/// The per-material features supported by builtin shaders.
/// At preset, material support 128 features, whose indices are in [0, 128).
/// Feature index should be defined in your shader code.
pub struct BuiltinMaterialShaderFeatures {}

impl BuiltinMaterialShaderFeatures {
    /// The feature index of albedo map. It is used to enable/disable albedo map in shader.
    pub const FEATURE_FLAG_ALBEDO_MAP: u32 = 0;
    pub const FEATURE_FLAG_NORMAL_MAP: u32 = 1;
    pub const FEATURE_FLAG_METALLIC_ROUGHNESS_MAP: u32 = 2;
    pub const FEATURE_FLAG_AO_MAP: u32 = 3;
    pub const FEATURE_FLAG_EMISSIVE_MAP: u32 = 4;
}

/// The global features supported by builtin shaders.
pub struct BuiltinGlobalShaderFeatures {}

impl BuiltinGlobalShaderFeatures {
    /// IBL feature.
    pub const FEATURE_FLAG_IBL: u32 = 0;
}