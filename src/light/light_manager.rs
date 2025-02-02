use crate::{
    math::{UVec4, Vec4},
    prelude::{
        bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager, GraphicsContext,
        SceneObject, TransformManager, INVALID_ID,
    },
    types::ID,
};

use super::point_light::PointLight;

pub struct LightManager {
    point_lights: Vec<PointLight>,
    light_buffer: Option<wgpu::Buffer>,

    // bind_group_layout_id: usize,
    bind_group_id: usize,
}

impl Default for LightManager {
    fn default() -> Self {
        Self {
            point_lights: Vec::new(),
            light_buffer: None,
            // bind_group_layout_id: INVALID_ID,
            bind_group_id: INVALID_ID,
        }
    }
}

impl LightManager {
    pub fn init_after_app(
        &mut self,
        graphics_context: &GraphicsContext,
        bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
        transform_manager: &TransformManager,
    ) {
        self.create_light_buffer(graphics_context, transform_manager);
        self.create_bind_group(
            graphics_context,
            bind_group_manager,
            bind_group_layout_manager,
        );
    }

    pub fn add_point_light(&mut self, light: PointLight) -> ID {
        let index = self.point_lights.len();
        self.point_lights.push(light);
        index
    }

    pub fn get_point_light(&self, index: usize) -> &PointLight {
        &self.point_lights[index]
    }

    /// TODO: to support add lights at any runtime frame.
    fn create_light_buffer(
        &mut self,
        graphics_context: &GraphicsContext,
        transform_manager: &TransformManager,
    ) {
        let light_count = UVec4::new(0, self.point_lights.len() as u32, 0, 0);
        let light_count_arr = light_count.to_array();

        let light_count_ref: &[u8] = bytemuck::cast_slice(&light_count_arr);
        let mut lights_storage_data: Vec<f32> = Vec::new();
        for light in self.point_lights.iter() {
            let light_pos = transform_manager
                .get_transform(*light.transform())
                .get_position();
            let position = Vec4::new(light_pos.x, light_pos.y, light_pos.z, 1.0);
            lights_storage_data.extend_from_slice(position.as_ref());
            let c = light.get_color();
            let color = Vec4::new(c.x, c.y, c.z, 1.0);
            lights_storage_data.extend_from_slice(color.as_ref());
        }

        let lights_info_ref: &[u8] = bytemuck::cast_slice(&lights_storage_data);

        let mut lights_ref = [light_count_ref, lights_info_ref].concat();
        // align with 32 bytes.
        let padding_size = (32 - (lights_ref.len() % 32)) % 32;
        lights_ref.extend(vec![0; padding_size]); 

        let lights_storage_buffer =
            graphics_context.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Fragment Storage Buffer"),
                contents: bytemuck::cast_slice(&lights_ref),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        self.light_buffer = Some(lights_storage_buffer);
    }

    pub fn get_binding_resource(&self) -> Option<wgpu::BindingResource> {
        match &self.light_buffer {
            Some(light_storage_buffer) => Some(light_storage_buffer.as_entire_binding()),
            _ => None,
        }
    }

    fn create_bind_group(
        &mut self,
        graphics_context: &GraphicsContext,
        bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
    ) -> ID {
        let bind_group_layout = bind_group_layout_manager.get_lighting_bind_group_layout();
        let bind_group = graphics_context.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            label: Some("Camera bind group"),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.get_binding_resource().unwrap(),
            }],
        });
        let bind_group_id = bind_group_manager.add_bind_group(bind_group);
        self.bind_group_id = bind_group_id;
        bind_group_id
    }

    pub fn get_bind_group_id(&self) -> ID {
        self.bind_group_id
    }

    // pub fn get_bind_group_layout_id(&self) -> ID {
    //     self.bind_group_layout_id
    // }
}
