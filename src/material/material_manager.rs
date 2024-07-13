use crate::prelude::{bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager, texture_manager::TextureManager, GraphicsContext};

use super::material::MaterialTrait;


pub struct MaterialManager {
    materials: Vec<Box<dyn MaterialTrait>>,
}

impl Default for MaterialManager {
    fn default() -> Self {
        Self {
            materials: Vec::new(),
        }
    }
}

impl MaterialManager {
    pub fn add_material(&mut self, material: Box<dyn MaterialTrait>) -> usize {
        let index = self.materials.len();
        self.materials.push(material);
        index
    }

    pub fn get_material(&self, index: usize) -> &Box<dyn MaterialTrait> {
        &self.materials[index]
    }

    pub fn get_material_mut(&mut self, index: usize) -> &mut Box<dyn MaterialTrait> {
        &mut self.materials[index]
    }

    pub fn init_after_app(&mut self, graphics_context: &GraphicsContext, bind_group_manager: &mut BindGroupManager
        , bind_group_layout_manager: &mut BindGroupLayoutManager, texture_manager: &TextureManager) {

        for material in self.materials.iter_mut() {
            material.init(graphics_context, bind_group_layout_manager);
            material.create_bind_group(graphics_context, bind_group_manager, bind_group_layout_manager, texture_manager);
        }
    }
}