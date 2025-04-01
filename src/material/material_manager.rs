use crate::{
    asset::asset_manager::AssetManager, prelude::{
        bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager, GraphicsContext,
    }, types::ID
};

use super::material_trait::MaterialTrait;

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
    pub(crate) fn add_material(
        &mut self,
        mut material: Box<dyn MaterialTrait>,
        graphics_context: &GraphicsContext,
        bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
        // texture_manager: &TextureManager,
        asset_manager: &AssetManager,
    ) -> ID {
        material.on_init(graphics_context, bind_group_layout_manager);
        material.create_bind_group(
            graphics_context,
            bind_group_manager,
            bind_group_layout_manager,
            asset_manager,
        );
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

    // pub(crate) fn on_update(&mut self, graphics_context: &GraphicsContext) {
    //     for material in self.materials.iter_mut() {
    //         material.on_update(graphics_context);
    //     }
    // }

    // pub fn init_after_app(
    //     &mut self,
    //     graphics_context: &GraphicsContext,
    //     bind_group_manager: &mut BindGroupManager,
    //     bind_group_layout_manager: &mut BindGroupLayoutManager,
    //     texture_manager: &TextureManager,
    // ) {
    //     for material in self.materials.iter_mut() {
    //         material.init(graphics_context, bind_group_layout_manager);
    //         material.create_bind_group(
    //             graphics_context,
    //             bind_group_manager,
    //             bind_group_layout_manager,
    //             texture_manager,
    //         );
    //     }
    // }
}
