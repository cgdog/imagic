use crate::{
    assets::{ShaderHandle, shaders::shader::Shader}, core::arena::Arena
};

/// Builtin shader handles.
pub(crate) struct BuiltinShaderHandles {
    pub pbr: ShaderHandle,
    pub unlit: ShaderHandle,
    pub blit: ShaderHandle,
    pub skybox: ShaderHandle,
    pub equirect_to_cube: ShaderHandle,
}

impl BuiltinShaderHandles {
    pub(crate) fn new() -> Self {
        Self {
            pbr: ShaderHandle::INVALID,
            unlit: ShaderHandle::INVALID,
            blit: ShaderHandle::INVALID,
            skybox: ShaderHandle::INVALID,
            equirect_to_cube: ShaderHandle::INVALID,
        }
    }
}

/// Shader manager.
pub struct ShaderManager {
    pub(crate) shaders: Arena<Shader>,
    pub(crate) builtin_shader_handles: BuiltinShaderHandles,
}

impl ShaderManager {

    /// Create a new shader manager.
    pub(crate) fn new() -> Self {
        ShaderManager {
            shaders: Arena::new(),
            builtin_shader_handles: BuiltinShaderHandles::new(),
        }
    }

    /// Create a new shader.
    /// # Arguments
    /// 
    /// * `source` - The shader source code.
    /// * `name` - The shader name.
    /// # Returns
    /// 
    /// * `ShaderHandle` - The shader handle.
    pub fn create_shader(&mut self, source: &str, name: String) -> ShaderHandle {
        self.shaders.add(Shader::new(source, name))
    }

    /// Get a shader by handle.
    /// # Arguments
    /// 
    /// * `handle` - The shader handle to get.
    /// # Returns
    /// 
    /// * `Some(&Shader)` - The shader if found.
    /// * `None` - The shader not found.
    pub fn get_shader(&self, handle: &ShaderHandle) -> Option<&Shader> {
        self.shaders.get(handle)
    }

    /// Get a shader by handle forcely.
    /// # Arguments
    /// 
    /// * `handle` - The shader handle to get.
    /// # Returns
    /// 
    /// * `&Shader` - The shader.
    /// # Panics
    /// 
    /// * If the shader not found.
    pub fn get_shader_forcely(&self, handle: &ShaderHandle) -> &Shader {
        self.shaders.get_forcely(handle)
    }
    /// Get a shader by handle mutably.
    /// # Arguments
    /// 
    /// * `handle` - The shader handle to get.
    /// # Returns
    /// 
    /// * `Some(&mut Shader)` - The shader if found.
    /// * `None` - The shader not found.
    pub fn get_shader_mut(&mut self, handle: &ShaderHandle) -> Option<&mut Shader> {
        self.shaders.get_mut(handle)
    }

    /// Get a shader by handle forcely mutably.
    /// # Arguments
    /// 
    /// * `handle` - The shader handle to get.
    /// # Returns
    /// 
    /// * `&mut Shader` - The shader.
    /// # Panics
    /// 
    /// * If the shader not found.
    pub fn get_shader_mut_forcely(&mut self, handle: &ShaderHandle) -> &mut Shader {
        self.shaders.get_mut_forcely(handle)
    }

    /// Destroy a shader by handle.
    /// # Arguments
    /// 
    /// * `handle` - The shader handle to destroy.
    /// # Returns
    /// 
    /// * `true` - The shader is destroyed.
    /// * `false` - The shader not found.
    pub fn destroy_shader(&mut self, handle: &ShaderHandle) -> bool {
        self.shaders.remove(handle).is_some()
    }

    /// Get the builtin PBR shader.
    /// # Returns
    /// 
    /// * `&Shader` - The PBR shader.
    pub fn get_builtin_pbr_shader(&mut self) -> (&Shader, &ShaderHandle) {
        
        if self.builtin_shader_handles.pbr == ShaderHandle::INVALID {
            let pbr_shader = Shader::new(include_str!("wgsl/pbr.wgsl"), "pbr".into());
            self.builtin_shader_handles.pbr = self.shaders.add(pbr_shader);
        }
        (self.shaders.get_forcely(&self.builtin_shader_handles.pbr), &self.builtin_shader_handles.pbr)
    }

    /// Get the builtin unlit shader.
    /// # Returns   
    /// 
    /// * `&Shader` - The unlit shader.
    pub fn get_builtin_unlit_shader(&mut self) -> (&Shader, &ShaderHandle) {
        if self.builtin_shader_handles.unlit == ShaderHandle::INVALID {
            let unlit_shader = Shader::new(include_str!("wgsl/unlit.wgsl"), "unlit".into());
            self.builtin_shader_handles.unlit = self.shaders.add(unlit_shader);
        }
        (self.shaders.get_forcely(&self.builtin_shader_handles.unlit), &self.builtin_shader_handles.unlit)
    }

    /// Get the builtin blit shader.
    /// # Returns
    /// 
    /// * `&Shader` - The blit shader.
    pub fn get_builtin_blit_shader(&mut self) -> (&Shader, &ShaderHandle) {
        if self.builtin_shader_handles.blit == ShaderHandle::INVALID {
            let blit_shader = Shader::new(include_str!("wgsl/blit.wgsl"), "blit".into());
            self.builtin_shader_handles.blit = self.shaders.add(blit_shader);
        }
        (self.shaders.get_forcely(&self.builtin_shader_handles.blit), &self.builtin_shader_handles.blit)
    }

    /// Get the builtin skybox shader.
    /// # Returns
    /// 
    /// * `&Shader` - The skybox shader.
    pub fn get_builtin_skybox_shader(&mut self) -> (&Shader, &ShaderHandle) {
        if self.builtin_shader_handles.skybox == ShaderHandle::INVALID {
            let skybox_shader = Shader::new(include_str!("wgsl/skybox.wgsl"), "skybox".into());
            self.builtin_shader_handles.skybox = self.shaders.add(skybox_shader);
        }
        (self.shaders.get_forcely(&self.builtin_shader_handles.skybox), &self.builtin_shader_handles.skybox)
    }

    /// Get the builtin equirect to cube shader.
    /// # Returns
    /// 
    /// * `&Shader` - The equirect to cube shader.
    pub fn get_builtin_equirect_to_cube_shader(&mut self) -> (&Shader, &ShaderHandle) {
        if self.builtin_shader_handles.equirect_to_cube == ShaderHandle::INVALID {
            let equirect_to_cube_shader = Shader::new(include_str!("wgsl/equirect_to_cube.wgsl"), "equirect_to_cube".into());
            self.builtin_shader_handles.equirect_to_cube = self.shaders.add(equirect_to_cube_shader);
        }
        (self.shaders.get_forcely(&self.builtin_shader_handles.equirect_to_cube), &self.builtin_shader_handles.equirect_to_cube)
    }
}
