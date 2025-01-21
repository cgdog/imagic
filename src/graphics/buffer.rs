use crate::types::ID;

pub struct GPUBufferManager {
    buffers: Vec<wgpu::Buffer>,
}

impl Default for GPUBufferManager {
    fn default() -> Self {
        Self { buffers: Vec::new() }
    }
}

impl GPUBufferManager {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_buffer(&mut self, buffer: wgpu::Buffer) -> ID {
        let index = self.buffers.len();
        self.buffers.push(buffer);
        index
    }

    pub fn get_buffer(&self, index: usize) -> &wgpu::Buffer {
        if index >= self.buffers.len() {
            panic!("buffer index out of bound.");
        }
        &self.buffers[index]
    }

    pub fn get_buffer_mut(&mut self, index: usize) -> &mut wgpu::Buffer {
        if index >= self.buffers.len() {
            panic!("buffer index of bound.");
        }
        &mut self.buffers[index]
    }
}