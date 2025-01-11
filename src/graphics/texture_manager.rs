use crate::types::ID;

use super::texture::Texture;

pub struct TextureManager {
    textures: Vec<Texture>,
}

impl Default for TextureManager {
    fn default() -> Self {
        Self {
            textures: Vec::new()
        }
    }
}

impl TextureManager {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_texture(&mut self, texture: Texture) -> ID {
        let id = self.textures.len();
        self.textures.push(texture);
        id
    }

    pub fn get_texture(&self, id: usize) -> &Texture {
        &self.textures[id]
    }

    pub fn get_texture_view(&self, id: usize) -> &wgpu::TextureView {
        self.textures[id].get_texture_view()
    }
}