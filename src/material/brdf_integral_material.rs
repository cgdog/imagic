use std::borrow::Cow;

use crate::{prelude::{bind_group_layout::BindGroupLayoutManager, GraphicsContext, INVALID_ID}, types::ID};

use super::MaterialTrait;

pub struct BRDFIntegralMaterial {
    bind_group_id: ID,
}

impl Default for BRDFIntegralMaterial {
    fn default() -> Self {
        Self {
            bind_group_id: INVALID_ID,
        }
    }
}

impl MaterialTrait for BRDFIntegralMaterial {
    fn on_init(
        &mut self,
        graphics_context: &crate::prelude::GraphicsContext,
        bind_group_layout_manager: &mut crate::prelude::bind_group_layout::BindGroupLayoutManager,
    ) {
        Self::try_create_bind_group_layout(graphics_context, bind_group_layout_manager);
    }

    fn create_shader_module(&self, graphics_context: &crate::prelude::GraphicsContext) -> wgpu::ShaderModule {
        let shader = graphics_context.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("create brdf integral shader module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/brdf_integral.wgsl"))),
        });
        shader
    }

    fn create_bind_group(
        &mut self,
        graphics_context: &crate::prelude::GraphicsContext,
        bind_group_manager: &mut crate::prelude::bind_group::BindGroupManager,
        bind_group_layout_manager: &mut crate::prelude::bind_group_layout::BindGroupLayoutManager,
        _texture_manager: &crate::prelude::texture_manager::TextureManager,
    ) -> crate::prelude::ID {
        let bind_group_layout =
            bind_group_layout_manager.get_bind_group_layout(self.get_bind_group_layout_id());
        let bind_group = graphics_context.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            label: Some("brdf integral bind group"),
            entries: &[
            ],
        });

        let bind_group_id = bind_group_manager.add_bind_group(bind_group);
        self.bind_group_id = bind_group_id;
        self.bind_group_id
    }

    fn enable_lights(&self) -> bool {
        false
    }

    fn get_bind_group_layout_id(&self) -> ID {
        Self::internal_bind_group_layout_id(INVALID_ID)
    }

    fn get_bind_group_id(&self) -> ID {
        self.bind_group_id
    }
}

impl BRDFIntegralMaterial {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    fn internal_bind_group_layout_id(new_id: usize) -> ID {
        // All SkyboxMaterial instances share the same bind group layout.
        static mut LAYOUT_ID: ID = INVALID_ID;
        if new_id != INVALID_ID {
            unsafe { LAYOUT_ID = new_id };
        }
        unsafe { LAYOUT_ID }
    }

    fn try_create_bind_group_layout(
        graphics_context: &GraphicsContext,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
    ) {
        let layout_id = Self::internal_bind_group_layout_id(INVALID_ID);
        if layout_id == INVALID_ID {
            let bind_group_layout = Self::create_bind_group_layout(graphics_context);
            let layout_id = bind_group_layout_manager.add_bind_group_layout(bind_group_layout);
            Self::internal_bind_group_layout_id(layout_id);
        }
    }

    fn create_bind_group_layout(graphics_context: &GraphicsContext) -> wgpu::BindGroupLayout {
        let bind_group_layout =
            graphics_context.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                ],
                label: Some("BRDFIntegralMaterial_bind_group_layout"),
            });
        bind_group_layout
    }
}