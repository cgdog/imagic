use std::usize;

use crate::{asset::asset::Handle, camera::Layer, prelude::{Material, INVALID_ID}, types::ID};

pub mod render_item_manager;

#[derive(Clone, Copy)]
pub enum VertexOrIndexCount {
    VertexCount{vertex_count: u32, instance_count: u32},
    IndexCount{index_count: u32, base_vertex: i32, instance_count: u32, index_format: wgpu::IndexFormat},
}

pub struct RenderItem {
    // pipeline_id: usize,
    // model matrix bind group
    item_bind_group_id: usize,
    vertex_buffer_id: usize,
    index_buffer_id: usize,
    material_id: Handle<Material>,
    transform_id: usize,
    vertex_or_index_count: VertexOrIndexCount,
    pub is_visible: bool,
    pub layer: Layer,
}

impl Default for RenderItem {
    fn default() -> Self {
        Self {
            // pipeline_id: INVALID_ID,
            item_bind_group_id: INVALID_ID,
            vertex_buffer_id: INVALID_ID,
            index_buffer_id: INVALID_ID,
            material_id: Handle::INVALID,
            transform_id: INVALID_ID,
            vertex_or_index_count: VertexOrIndexCount::VertexCount{vertex_count: 0, instance_count: 1},
            is_visible: true,
            layer: Layer::Default,
        }
    }
}

impl RenderItem {
    pub fn new(vertex_or_index_count: VertexOrIndexCount, vertex_buffer_id: usize,
            index_buffer_id: usize, transform_id: usize, is_visible: bool) -> Self {
        Self {
            vertex_buffer_id,
            index_buffer_id,
            material_id: Handle::INVALID,
            transform_id,
            vertex_or_index_count: vertex_or_index_count,
            is_visible,
            ..Default::default()
        }
    }

    pub fn new_thinly(vertex_or_index_count: VertexOrIndexCount) -> Self {
        Self {
            // pipeline_id,
            // bind_group_ids: bind_group_id,
            vertex_or_index_count,
            ..Default::default()
        }
    }

    // pub fn set_pipeline(&mut self, pipeline_id: usize) {
    //     self.pipeline_id = pipeline_id;
    // }

    // pub fn get_pipeline(&self) -> ID {
    //     self.pipeline_id
    // }

    pub fn set_item_bind_group_id(&mut self, bind_group_id: usize) {
        self.item_bind_group_id = bind_group_id;
    }

    pub fn get_item_bind_group_id(&self) -> ID {
        self.item_bind_group_id
    }

    pub fn set_vertex_buffer_id(&mut self, vertex_buffer_id: usize) {
        self.vertex_buffer_id = vertex_buffer_id;
    }

    pub fn get_vertex_buffer_id(&self) -> ID {
        self.vertex_buffer_id
    }

    pub fn set_index_buffer_id(&mut self, index_buffer_id: usize) {
        self.index_buffer_id = index_buffer_id;
    }

    pub fn get_index_buffer_id(&self) -> ID {
        self.index_buffer_id
    }

    pub fn set_material_id(&mut self, material_handle: Handle<Material>) {
        self.material_id = material_handle;
    }

    pub fn get_material_id(&self) -> &Handle<Material> {
        &self.material_id
    }

    pub fn set_vertex_or_index_count(&mut self, vertex_or_index_count: VertexOrIndexCount) {
        self.vertex_or_index_count = vertex_or_index_count;
    }

    pub fn get_vertex_or_index_count(&self) ->& VertexOrIndexCount {
        &self.vertex_or_index_count
    }

    pub fn set_transform_id(&mut self, transform_id: usize) {
        self.transform_id = transform_id;
    }

    pub fn get_transform_id(&self) -> ID {
        self.transform_id
    }

}