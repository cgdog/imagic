use ahash::AHashMap;

use crate::{
    assets::shaders::shader::Shader, types::RR,
};

pub struct ShaderManager {
    pub(crate) shaders: AHashMap<String, RR<Shader>>,
}

impl ShaderManager {
    pub(crate) const PBR: &'static str = "pbr";
    pub(crate) const UNLIT: &'static str = "unlit";
    pub(crate) const BLIT: &'static str = "blit";
    pub(crate) const SKYBOX: &'static str = "skybox";
    pub(crate) const EQUIRECT_TO_CUBE_SHADER: &'static str = "equirect_to_cube";

    pub(crate) fn new() -> Self {
        ShaderManager {
            shaders: AHashMap::new(),
        }
    }

    pub fn get_builtin_pbr_shader(&mut self) -> RR<Shader> {
        if let Some(pbr_shader) = self.shaders.get(Self::PBR) {
            pbr_shader.clone()
        } else {
            let pbr_shader = Shader::new(include_str!("wgsl/pbr.wgsl"), Self::PBR.into());
            self.shaders.insert(Self::PBR.to_owned(), pbr_shader.clone());
            pbr_shader
        }
    }

    pub fn get_builtin_unlit_shader(&mut self) -> RR<Shader> {
        if let Some(shader) = self.shaders.get(Self::UNLIT) {
            shader.clone()
        } else {
            let shader = Shader::new(include_str!("wgsl/unlit.wgsl"), Self::UNLIT.into());
            self.shaders.insert(Self::UNLIT.to_owned(), shader.clone());
            shader
        }
    }

    pub fn get_builtin_blit_shader(&mut self) -> RR<Shader> {
        if let Some(shader) = self.shaders.get(Self::BLIT) {
            shader.clone()
        } else {
            let shader = Shader::new(include_str!("wgsl/blit.wgsl"), Self::BLIT.into());
            self.shaders.insert(Self::BLIT.to_owned(), shader.clone());
            shader
        }
    }

    pub fn get_builtin_skybox_shader(&mut self) -> RR<Shader> {
        if let Some(shader) = self.shaders.get(Self::SKYBOX) {
            shader.clone()
        } else {
            let shader = Shader::new(include_str!("wgsl/skybox.wgsl"), Self::SKYBOX.into());
            self.shaders.insert(Self::SKYBOX.to_owned(), shader.clone());
            shader
        }
    }

    pub fn get_builtin_equirect_to_cube_shader(&mut self) -> RR<Shader> {
        if let Some(shader) = self.shaders.get(Self::EQUIRECT_TO_CUBE_SHADER) {
            shader.clone()
        } else {
            let shader = Shader::new(include_str!("wgsl/equirect_to_cube.wgsl"), Self::EQUIRECT_TO_CUBE_SHADER.into());
            self.shaders.insert(Self::EQUIRECT_TO_CUBE_SHADER.to_owned(), shader.clone());
            shader
        }
    }
}
