pub mod render_item_manager;

pub enum VertexOrIndexCount {
    VertexCount{vertex_count: u32, instance_count: u32},
    IndexCount{index_count: u32, base_vertex: i32, instance_count: u32, index_format: wgpu::IndexFormat},
}

pub struct RenderItem {
    pipeline_id: usize,
    bind_group_ids: Vec<usize>,
    vertex_buffer_id: usize,
    index_buffer_id: usize,
    material_id: usize,
    transform_id: usize,
    vertex_or_index_count: VertexOrIndexCount,
    pub is_visible: bool,
}

impl Default for RenderItem {
    fn default() -> Self {
        Self {
            pipeline_id: usize::MAX,
            bind_group_ids: Vec::new(),
            vertex_buffer_id: usize::MAX,
            index_buffer_id: usize::MAX,
            material_id: usize::MAX,
            transform_id: usize::MAX,
            vertex_or_index_count: VertexOrIndexCount::VertexCount{vertex_count: 0, instance_count: 1},
            is_visible: true,
        }
    }
}

impl RenderItem {
    pub fn new(vertex_or_index_count: VertexOrIndexCount, vertex_buffer_id: usize,
            index_buffer_id: usize, transform_id: usize, is_visible: bool) -> Self {
        Self {
            vertex_buffer_id,
            index_buffer_id,
            material_id: usize::MAX,
            transform_id,
            vertex_or_index_count: vertex_or_index_count,
            is_visible,
            ..Default::default()
        }
    }

    pub fn new_thinly(pipeline_id: usize, vertex_or_index_count: VertexOrIndexCount) -> Self {
        Self {
            pipeline_id,
            // bind_group_ids: bind_group_id,
            vertex_or_index_count,
            ..Default::default()
        }
    }

    pub fn set_pipeline(&mut self, pipeline_id: usize) {
        self.pipeline_id = pipeline_id;
    }

    pub fn get_pipeline(&self) -> usize {
        self.pipeline_id
    }

    pub fn set_bind_groups(&mut self, bind_group_ids: Vec<usize>) {
        self.bind_group_ids = bind_group_ids;
    }

    pub fn get_bind_group(&self) -> &Vec<usize> {
        &self.bind_group_ids
    }

    pub fn set_vertex_buffer_id(&mut self, vertex_buffer_id: usize) {
        self.vertex_buffer_id = vertex_buffer_id;
    }

    pub fn get_vertex_buffer_id(&self) -> usize {
        self.vertex_buffer_id
    }

    pub fn set_index_buffer_id(&mut self, index_buffer_id: usize) {
        self.index_buffer_id = index_buffer_id;
    }

    pub fn get_index_buffer_id(&self) -> usize {
        self.index_buffer_id
    }

    pub fn set_material_id(&mut self, material_id: usize) {
        self.material_id = material_id;
    }

    pub fn get_material_id(&self) -> usize {
        self.material_id
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

    pub fn get_transform_id(&self) -> usize {
        self.transform_id
    }

}