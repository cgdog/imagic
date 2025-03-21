use wgpu::{util::{align_to, DeviceExt}, CommandEncoder};

use crate::{asset::asset::Asset, types::ID};

use super::GraphicsContext;

impl Asset for wgpu::Buffer {}

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

pub struct SyncBuffer {
    sync_buffer: wgpu::Buffer,
    staging_buffer: wgpu::Buffer,
}

impl SyncBuffer {
    const BUFFER_SIZE: u64 = 4;

    pub fn new(graphics_context: &GraphicsContext) -> Self {
        let aligned_size = align_to(Self::BUFFER_SIZE, wgpu::COPY_BUFFER_ALIGNMENT);
        let sync_buffer = graphics_context.get_device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sync Buffer"),
            size: aligned_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let data = vec![1, 2, 3, 4];
        let mut padded_data = data.clone();
        padded_data.extend(vec![0; (aligned_size - data.len() as u64).try_into().unwrap()]); // 填充数据

        let staging_buffer = graphics_context.get_device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Staging Buffer"),
            contents: &padded_data,
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        Self {
            sync_buffer,
            staging_buffer,
        }
    }

    pub fn sync(&self, encoder: &mut CommandEncoder) {
        encoder.copy_buffer_to_buffer(&self.staging_buffer, 0, &self.sync_buffer, 0, Self::BUFFER_SIZE);
    }

    pub fn receive(&self, graphics_context: &GraphicsContext) {

        let buffer_slice = self.sync_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        // waiting for blocked GPU task to finish.
        graphics_context.get_device().poll(wgpu::Maintain::Wait);

        // check callback result
        if let Ok(result) = receiver.recv() {
            result.unwrap(); // panic if GPU task failed.

            // access Buffer data
            let data = buffer_slice.get_mapped_range();
            println!("Received sync buffer data: {:?}", &*data);
        } else {
            println!("Not received sync buffer data");
        }
    }
}