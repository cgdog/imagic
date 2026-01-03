use std::{
    hash::{Hash, Hasher},
    u64,
};

use ahash::AHasher;

use crate::{
    assets::{
        ShaderHandle, ShaderManager, TextureSamplerManager, asset::IAsset, shaders::
            shader_property::{
                BuiltinMaterialShaderFeatures, BuiltinShaderUniformNames,
            }
        , textures::{
            sampler::SamplerHandle,
            texture::TextureHandle,
        }
    }, graphics::{
        bind_group::BindGroupID,
        graphics_context::GraphicsContext,
        render_states::RenderState, uniform::Uniforms,
    }, math::{IVec4, Mat3, Mat4, UVec4, Vec4, color::Color}, prelude::bind_group::INVALID_BINDGROUP_ID, types::HashID
};

/// Material tag used to declare MaterialHandle.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MaterialTag {}

/// Handle type for Material.
pub type MaterialHandle = crate::types::Handle<MaterialTag>;

/// Material struct
#[derive(PartialEq, Clone)]
pub struct Material {
    /// Render state.
    pub render_state: RenderState,
    /// Shader handle.
    pub shader_handle: ShaderHandle,
    material_hash: HashID,
    is_inited: bool,
    pub(crate) uniforms: Uniforms,
    /// Enabled features bit mask.
    /// 4 u32 is enough for 128 features.
    features: UVec4,

    is_dirty: bool,
    /// Changing texture needs to recreate bind group.
    is_texture_changed: bool,
}

impl IAsset for Material {}

#[allow(unused)]
impl Material {
    pub(crate) fn new(shader_handle: ShaderHandle, shader_manager: &mut ShaderManager) -> Self {
        let shader = shader_manager.get_shader_forcely(&shader_handle);
        log::info!("shader hash: {}", shader.hash);
        let mut hasher = AHasher::default();
        let shader_hash = shader.hash;
        shader_hash.hash(&mut hasher);
        let render_state = RenderState::default();
        render_state.hash(&mut hasher);
        let material_hash = hasher.finish();

        let uniforms = Uniforms::new(&shader.shader_properties.per_material_properties);
        let material = Self {
            render_state,
            shader_handle,
            material_hash,
            is_inited: false,
            uniforms,
            features: UVec4::ZERO,
            is_dirty: false,
            is_texture_changed: false,
        };
        material
    }

    /// Mark material as dirty, it will be refreshed when it is used.
    /// This recalculates material hash which may cause render pipeline to be recreated.
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    fn refresh_hash(&mut self, shader_manager: &mut ShaderManager) {
        if let Some(shader) = shader_manager.get_shader(&self.shader_handle) {
            let mut hasher = AHasher::default();
            let shader_hash = shader.hash;
            shader_hash.hash(&mut hasher);
            self.render_state.hash(&mut hasher);
            self.material_hash = hasher.finish();
            log::info!("Material marked dirty, new hash: {}", self.material_hash);
        } else {
            #[cfg(debug_assertions)]
            {
                log::warn!(
                    "Failed to refresh material hash: shader {:?} not found in shader manager.",
                    self.shader_handle
                );
            }
        }
    }

    pub fn set_float(&mut self, property_name: &str, value: f32) {
        self.uniforms.set_float(property_name, value);
    }

    // TODO: optimize: value use renference.
    pub fn set_vec4f(&mut self, property_name: &str, value: Vec4) {
        self.uniforms.set_vec4f(property_name, value);
    }

    pub fn get_vec4f(&self, property_name: &str) -> Option<Vec4> {
        self.uniforms.get_vec4f(property_name)
    }

    // TODO: optimize: value use renference.
    pub fn set_vec4i(&mut self, property_name: &str, value: IVec4) {
        self.uniforms.set_vec4i(property_name, value);
    }

    // TODO: optimize: value use renference.
    pub fn set_vec4u(&mut self, property_name: &str, value: UVec4) {
        self.uniforms.set_vec4u(property_name, value);
    }

    pub fn set_color(&mut self, property_name: &str, value: Color) {
        self.uniforms.set_color(property_name, value);
    }

    pub fn set_matrix3x3(&mut self, property_name: &str, value: Mat3) {
        self.uniforms.set_matrix3x3(property_name, value);
    }

    pub fn set_matrix4x4(&mut self, property_name: &str, value: Mat4) {
        self.uniforms.set_matrix4x4(property_name, value);
    }

    pub fn set_struct(&mut self, property_name: &str, value: Vec<u8>) {
        self.uniforms.set_struct(property_name, value);
    }

    pub fn set_texture(&mut self, property_name: &str, value: TextureHandle) {
        self.uniforms.set_texture(property_name, value);
        self.is_texture_changed = true;
    }

    pub fn get_texture(&self, property_name: &str) -> TextureHandle {
        self.uniforms.get_texture(property_name)
    }

    pub fn set_sampler(&mut self, property_name: &str, value: SamplerHandle) {
        self.uniforms.set_sampler(property_name, value);
    }

    pub fn get_sampler(&self, property_name: &str) -> SamplerHandle {
        self.uniforms.get_sampler(property_name)
    }

    fn create_bind_group(&mut self, graphics_context: &mut GraphicsContext, texture_sampler_manager: &TextureSamplerManager, shader_manager: &mut ShaderManager) {
        if let Some(shader) = shader_manager.get_shader(&self.shader_handle) {
            self.uniforms.bind_group_id = graphics_context.create_bind_group(
                &shader.shader_properties.per_material_properties,
                &self.uniforms.uniforms,
                "Create material bindgroup",
                texture_sampler_manager,
            );
        }
    }

    pub(crate) fn on_update(&mut self, graphics_context: &mut GraphicsContext, texture_sampler_manager: &mut TextureSamplerManager, shader_manager: &mut ShaderManager) {
        if self.is_dirty {
            self.refresh_hash(shader_manager);
            self.is_dirty = false;
        }
        if !self.is_inited {
            if let Some(shader_mut_ref) = shader_manager.get_shader_mut(&self.shader_handle) {
                if !shader_mut_ref.is_inited {
                    shader_mut_ref.init(graphics_context);
                    shader_mut_ref.is_inited = true;
                }
            } else {
                #[cfg(debug_assertions)]
                log::warn!("Failed to init material: shader {:?} not found in shader manager.", self.shader_handle);
            }
            self.is_inited = true;
        }

        self.uniforms.sync_properties(graphics_context, texture_sampler_manager);
        // info!("lxy before create bind groups");
        if self.uniforms.should_create_bind_group() || self.is_texture_changed {
            if (self.uniforms.bind_group_id != INVALID_BINDGROUP_ID) {
                graphics_context.remove_bind_group(&self.uniforms.bind_group_id);
            }
            self.create_bind_group(graphics_context, texture_sampler_manager, shader_manager);
            self.is_texture_changed = false;
        }
    }

    pub(crate) fn hash_value(&self) -> u64 {
        self.material_hash
    }

    pub(crate) fn get_bind_group(&self) -> BindGroupID {
        self.uniforms.bind_group_id
    }

    /// Get the shader handle of this material.
    /// # Returns
    ///
    /// The shader handle of this material.
    pub fn get_shader_handle(&self) -> &ShaderHandle {
        &self.shader_handle
    }

    pub(crate) fn set_time(&mut self, time: Vec4) {
        self.set_vec4f(BuiltinShaderUniformNames::_TIME, time);
    }

    ////////////////// Builtin unlit / pbr properties //////////////////

    /// Set albedo color property for builtin Unlit or PBR shader.
    ///
    /// # Parameters
    ///
    /// - `albedo_color`: the new albedo color.
    pub fn set_albedo_color(&mut self, albedo_color: Color) {
        self.set_color(BuiltinShaderUniformNames::_ALBEDO_COLOR, albedo_color);
    }

    /// Set albedo map property for builtin Unlit or PBR shader.
    ///
    /// # Parameters
    ///
    /// - `albedo_map`: the texture handle of albedo map.
    pub fn set_albedo_map(&mut self, albedo_map: TextureHandle) {
        self.set_texture(BuiltinShaderUniformNames::_ALBEDO_MAP, albedo_map);
        if albedo_map != TextureHandle::INVALID {
            self.enable_feature(BuiltinMaterialShaderFeatures::FEATURE_FLAG_ALBEDO_MAP);
        } else {
            self.disable_feature(BuiltinMaterialShaderFeatures::FEATURE_FLAG_ALBEDO_MAP);
        }
    }

    /// Set the sampler use to sample albedo map. It also could be used to sample other maps.
    ///
    /// # Parameters
    ///
    /// - `sampler`: the sampler handle which identifies a sampler.
    pub fn set_albedo_map_sampler(&mut self, sampler: SamplerHandle) {
        self.set_sampler(BuiltinShaderUniformNames::_ALBEDO_MAP_SAMPLER, sampler);
    }

    /// Set metallic for builtin PBR shader
    ///
    /// # Parameters
    ///
    /// - `metallic`: the new metallic property of PBR.
    pub fn set_metallic(&mut self, metallic: f32) {
        if let Some(metallic_roughness_ao) = self.get_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO) {
            self.set_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO,
                Vec4::new(metallic, metallic_roughness_ao.y, metallic_roughness_ao.z, metallic_roughness_ao.w));
        }
    }

    /// Get metallic property for builtin PBR shader.
    ///
    /// # Returns
    ///
    /// The metallic property of PBR.
    pub fn get_metallic(&self) -> f32 {
        if let Some(metallic_roughness_ao) = self.get_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO) {
            metallic_roughness_ao.x
        } else {
            0.0
        }
    }

    /// Set metallic roughness map property for builtin Unlit or PBR shader.
    ///
    /// # Parameters
    ///
    /// - `metallic_roughness_map`: the texture handle of metallic roughness map.
    pub fn set_metallic_roughness_map(&mut self, metallic_roughness_map: TextureHandle) {
        self.set_texture(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_MAP, metallic_roughness_map);
        if metallic_roughness_map != TextureHandle::INVALID {
            self.enable_feature(BuiltinMaterialShaderFeatures::FEATURE_FLAG_METALLIC_ROUGHNESS_MAP);
        } else {
            self.disable_feature(BuiltinMaterialShaderFeatures::FEATURE_FLAG_METALLIC_ROUGHNESS_MAP);
        }
    }

    /// Set ao map property for builtin PBR shader.
    ///
    /// # Parameters
    ///
    /// - `ao_map`: the texture handle of ao map.
    pub fn set_ao_map(&mut self, ao_map: TextureHandle) {
        self.set_texture(BuiltinShaderUniformNames::_AO_MAP, ao_map);
        if ao_map != TextureHandle::INVALID {
            self.enable_feature(BuiltinMaterialShaderFeatures::FEATURE_FLAG_AO_MAP);
        } else {
            self.disable_feature(BuiltinMaterialShaderFeatures::FEATURE_FLAG_AO_MAP);
        }
    }

    /// Set emissive color property for builtin Unlit or PBR shader.
    ///
    /// # Parameters
    ///
    /// - `emissive_color`: the new emissive color.
    pub fn set_emissive_color(&mut self, emissive_color: Color) {
        self.set_color(BuiltinShaderUniformNames::_EMISSIVE_COLOR, emissive_color);
    }

    /// Set emissive map property for builtin PBR shader.
    ///
    /// # Parameters
    ///
    /// - `emissive_map`: the texture handle of emissive map.
    pub fn set_emissive_map(&mut self, emissive_map: TextureHandle) {
        self.set_texture(BuiltinShaderUniformNames::_EMISSIVE_MAP, emissive_map);
        if emissive_map != TextureHandle::INVALID {
            self.enable_feature(BuiltinMaterialShaderFeatures::FEATURE_FLAG_EMISSIVE_MAP);
        } else {
            self.disable_feature(BuiltinMaterialShaderFeatures::FEATURE_FLAG_EMISSIVE_MAP);
        }
    }

    /// Set normal map property for builtin PBR shader.
    ///
    /// # Parameters
    ///
    /// - `_normal_map`: the texture handle of normal map.
    pub fn set_normal_map(&mut self, normal_map: TextureHandle) {
        self.set_texture(BuiltinShaderUniformNames::_NORMAL_MAP, normal_map);
        if normal_map != TextureHandle::INVALID {
            self.enable_feature(BuiltinMaterialShaderFeatures::FEATURE_FLAG_NORMAL_MAP);
        } else {
            self.disable_feature(BuiltinMaterialShaderFeatures::FEATURE_FLAG_NORMAL_MAP);
        }
    }

    /// Set metallic for builtin PBR shader
    ///
    /// # Parameters
    ///
    /// - `roughness`: the new roughness property of PBR.
    pub fn set_roughness(&mut self, roughness: f32) {
        if let Some(metallic_roughness_ao) = self.get_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO) {
            self.set_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO,
                Vec4::new(metallic_roughness_ao.x, roughness, metallic_roughness_ao.z, metallic_roughness_ao.w));
        }
    }

    /// Get roughness property for builtin PBR shader.
    ///
    /// # Returns
    ///
    /// The roughness property of PBR.
    pub fn get_roughness(&self) -> f32 {
        if let Some(metallic_roughness_ao) = self.get_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO) {
            metallic_roughness_ao.y
        } else {
            0.0
        }
    }

    /// Set ao property for builtin PBR shader.
    ///
    /// # Parameters
    ///
    /// - `ao`: the new ao property of PBR.
    pub fn set_ao(&mut self, ao: f32) {
        if let Some(metallic_roughness_ao) = self.get_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO) {
            self.set_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO,
                Vec4::new(metallic_roughness_ao.x, metallic_roughness_ao.y, ao, metallic_roughness_ao.w));
        }
    }

    /// Get ao property for builtin PBR shader.
    ///
    /// # Returns
    ///
    /// The ao property of PBR.
    pub fn get_ao(&self) -> f32 {
        if let Some(metallic_roughness_ao) = self.get_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO) {
            metallic_roughness_ao.z
        } else {
            0.0
        }
    }

    /// Enable a feature by its index.
    ///
    /// At preset, material support 128 features.
    ///
    /// Feature index should be defined in your shader code.
    ///
    /// # Parameters
    /// - feature_index: the index of feature to be enabled which is in [0, 128).
    pub fn enable_feature(&mut self, feature_index: u32) {
        let array_index = (feature_index >> 5) as usize; // feature_index / 32
        let bit_index = (feature_index & 0b11111); // feature_index % 32;
        if array_index < 4 {
            let bit_mask = 1 << bit_index;
            if (self.features[array_index] & bit_mask) == 0 {
                self.features[array_index] |= bit_mask;
                self.set_vec4u(BuiltinShaderUniformNames::_MATERIAL_FEATURES, self.features);
            }
        } else {
            log::warn!("Feature index {} is out of range.", feature_index);
        }
    }

    /// Check if a feature is enabled by its index.
    /// # Parameters
    /// - feature_index: the index of feature to be checked which is in [0, 128).
    /// # Returns
    /// - true if the feature is enabled, false otherwise.
    pub fn is_feature_enabled(&self, feature_index: u32) -> bool {
        let array_index = (feature_index >> 5) as usize; // feature_index / 32
        let bit_index = (feature_index & 0b11111); // feature_index % 32;
        if array_index < 4 {
            let bit_mask = 1 << bit_index;
            (self.features[array_index] & bit_mask) != 0
        } else {
            log::warn!("Feature index {} is out of range.", feature_index);
            false
        }
    }

    /// Disable a feature by its index.
    ///
    /// At preset, material support 128 features.
    ///
    /// Feature index should be defined in your shader code.
    ///
    /// # Parameters
    /// - feature_index: the index of feature to be enabled which is in [0, 128).
    pub fn disable_feature(&mut self, feature_index: u32) {
        let array_index = (feature_index >> 5) as usize; // feature_index / 32
        let bit_index = (feature_index & 0b11111); // feature_index % 32;
        if array_index < 4 {
            let bit_mask = 1 << bit_index;
            if (self.features[array_index] & bit_mask) != 0 {
                self.features[array_index] &= !bit_mask;
                self.set_vec4u(BuiltinShaderUniformNames::_MATERIAL_FEATURES, self.features);
            }
        } else {
            log::warn!("Feature index {} is out of range.", feature_index);
        }
    }
}
