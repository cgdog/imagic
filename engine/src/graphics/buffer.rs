use crate::{graphics::{buffer_view::BufferView, graphics_context::GraphicsLimits}, utils::get_aligned_size};
use std::{cmp::Ordering, rc::Rc, usize};
use ahash::AHashMap;
use log::warn;
use wgpu::{util::DeviceExt, wgt::BufferDescriptor, Device, Queue};

pub type BufferUsages = wgpu::BufferUsages;
#[derive(PartialEq)]
pub struct Buffer {
    pub(crate) buffer: wgpu::Buffer,
}

impl From<wgpu::Buffer> for Buffer {
    fn from(value: wgpu::Buffer) -> Self {
        Self { buffer: value }
    }
}

impl Buffer {}


pub struct BufferBlockRange {
    pub start: u64,
    pub size: u64,
}

impl BufferBlockRange {
    pub fn new(start: u64, size: u64) -> Self {
        Self { start, size }
    }
}

pub struct BufferBlock {
    pub total_size: u64,
    pub min_range_size: u64,
    pub max_range_size: u64,
    pub free_ranges: Vec<BufferBlockRange>,

    pub buffer: Buffer,
}

impl BufferBlock {
    pub fn new(total_size: u64, buffer: Buffer) -> Self {
        Self {
            total_size,
            min_range_size: total_size,
            max_range_size: total_size,
            free_ranges: vec![BufferBlockRange::new(0, total_size)],
            buffer,
        }
    }

    pub fn allocate(&mut self, size: u64) -> Option<BufferBlockRange> {
        if size > self.max_range_size {
            return None;
        }
        let mut range_index = usize::MAX;
        let mut start = 0;
        let mut remain_start = 0;
        let mut reamin_size = 0;
        // TODO: optimize speed.
        for (index, range) in self.free_ranges.iter().enumerate() {
            if range.size >= size {
                range_index = index;
                start = range.start;
                remain_start = range.start + size;
                reamin_size = range.size - size;
            }
        }
        if range_index != usize::MAX {
            let cur_remain_range = BufferBlockRange::new(remain_start, reamin_size);
            self.free_ranges.remove(range_index);
            self.free_ranges.push(cur_remain_range);
            self._refresh_min_max_size();
            Some(BufferBlockRange::new(start, size))
        } else {
            None
        }
    }

    pub fn deallocate(&mut self, free_block_range: BufferBlockRange) {
        self.free_ranges.push(free_block_range);
        if self.free_ranges.len() < 2 {
            return;
        }
        self.free_ranges.sort_by(|range_a, range_b| {
            if range_a.start < range_b.start
                || (range_a.start == range_b.start) && range_a.size >= range_b.size
            {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });

        // merge ranges
        let mut new_ranges: Vec<BufferBlockRange> = Vec::new();
        let mut start = self.free_ranges[0].start;
        let mut end = start + self.free_ranges[0].size;
        for i in 1..(self.free_ranges.len() - 1) {
            let cur_range = &self.free_ranges[i];
            if end >= cur_range.start {
                end = end.max(cur_range.start + cur_range.size);
            } else {
                let new_block_range = BufferBlockRange::new(start, end - start);
                new_ranges.push(new_block_range);
                start = cur_range.start;
                end = start + cur_range.size;
            }
        }

        let new_block_range = BufferBlockRange::new(start, end - start);
        new_ranges.push(new_block_range);
        self.free_ranges = new_ranges;
        self._refresh_min_max_size();
    }

    fn _refresh_min_max_size(&mut self) {
        let mut min_size = u64::MAX;
        let mut max_size = u64::MIN;
        for range in &self.free_ranges {
            min_size = min_size.min(range.size);
            max_size = max_size.max(range.size);
        }
        self.min_range_size = min_size;
        self.max_range_size = max_size;
    }
}

/// Buffer manager, which allocates, deallocate and manager Buffers and BufferViews.
pub struct BufferManager {
    /// TODO: Allocate available buffer from free_buffers，return a new BufferView, and update start and end of free_buffers[i], which are available range.
    pub(crate) buffer_blocks: AHashMap<BufferUsages, Vec<BufferBlock>>,
    device: Rc<Device>,
    queue: Rc<Queue>,
    limits: Rc<GraphicsLimits>,
}

impl BufferManager {
    /// The size of an allocated buffer will be multiple of [`MIN_BUFFER_SIZE`]. It must be a power of 2.
    const MIN_BUFFER_SIZE: u64 = 1024; // 1 Kb
    const INIT_BUFFER_SIZE: u64 = 2048; // 2 kb

    pub(crate) fn new(device: Rc<Device>, queue: Rc<Queue>, limits: Rc<GraphicsLimits>) -> Self {
        let uniform_buffer_usage = BufferUsages::UNIFORM | BufferUsages::COPY_DST;
        let buffer_block = Self::create_buffer_block(&device, Self::INIT_BUFFER_SIZE, uniform_buffer_usage, Some("Create init bultin uniform buffer"));
        let mut buffer_blocks = AHashMap::new();
        buffer_blocks.insert(uniform_buffer_usage, vec![buffer_block]);

        let buffer_manager = Self {
            buffer_blocks,
            device,
            queue,
            limits,
        };

        buffer_manager
    }

    pub fn allocate_vertex_buffer(&mut self, size: u64) -> BufferView {
        self.allocates(size, BufferUsages::VERTEX | BufferUsages::COPY_DST, Some("A Vertex Buffer"))
    }

    pub fn allocate_index_buffer(&mut self, size: u64) -> BufferView {
        self.allocates(size, BufferUsages::INDEX | BufferUsages::COPY_DST, Some("An Index Buffer"))
    }

    pub fn allocate_uniform_buffer(&mut self, size: u64) -> BufferView {
        // process min_uniform_buffer_offset_alignment
        let aligned_size = get_aligned_size(self.limits.min_uniform_buffer_offset_alignment as u64, size);
        self.allocates(aligned_size, BufferUsages::UNIFORM | BufferUsages::COPY_DST, Some("A Uniform Buffer"))
    }

    pub fn allocate_uniform_buffer_init(&mut self, size: u64, data: &[u8]) -> BufferView {
        self.allocates_init(size, BufferUsages::UNIFORM | BufferUsages::COPY_DST, data, Some("A Uniform Buffer With Init Data"))
    }

    pub fn allocates(&mut self, size: u64, usage: BufferUsages, label: Option<&str>) -> BufferView {
        let mut new_block_index = 0;
        if let Some(cur_usage_buffer_blocks) = self.buffer_blocks.get_mut(&usage) {
            new_block_index = cur_usage_buffer_blocks.len();
            for (block_index, cur_block) in &mut cur_usage_buffer_blocks.iter_mut().enumerate() {
                if let Some(block_range) = cur_block.allocate(size) {
                    let buffer_view = BufferView::new(block_range.start, block_range.size, block_index, usage);
                    return buffer_view;
                }
            }
        }

        let mut new_buffer_block = Self::create_buffer_block(&self.device, size, usage, label);
        if let Some(block_range) = new_buffer_block.allocate(size) {
            let buffer_view = BufferView::new(block_range.start, block_range.size, new_block_index, usage);
            if let Some(cur_usage_buffer_blocks) = self.buffer_blocks.get_mut(&usage) {
                cur_usage_buffer_blocks.push(new_buffer_block);
            } else {
                self.buffer_blocks.insert(usage, vec![new_buffer_block]);
            }
            return buffer_view;
        }
        // TODO: optimize to avoid panic.
        panic!("Failed to allocate buffer");
    }

    pub fn allocates_init(&mut self, size: u64, usage: BufferUsages, data: &[u8], label: Option<&str>) -> BufferView {
        let mut new_block_index = 0;
        if let Some(cur_usage_buffer_blocks) = self.buffer_blocks.get_mut(&usage) {
            new_block_index = cur_usage_buffer_blocks.len();
            for (block_index, cur_block) in &mut cur_usage_buffer_blocks.iter_mut().enumerate() {
                if let Some(block_range) = cur_block.allocate(size) {
                    let buffer_view = BufferView::new(block_range.start, block_range.size, block_index, usage);
                    self.write_data(&buffer_view, data);
                    return buffer_view;
                }
            }
        }
        
        let mut new_buffer_block = Self::create_buffer_block_init(&self.device, size, usage, data, label);
        // At this moment, the data is already written into buffer.
        if let Some(block_range) = new_buffer_block.allocate(size) {
            let buffer_view = BufferView::new(block_range.start, block_range.size, new_block_index, usage);
            if let Some(cur_usage_buffer_blocks) = self.buffer_blocks.get_mut(&usage) {
                cur_usage_buffer_blocks.push(new_buffer_block);
            } else {
                self.buffer_blocks.insert(usage, vec![new_buffer_block]);
            }
            return buffer_view;
        }
        // TODO: optimize to avoid panic.
        panic!("Failed to allocate buffer init");
    }

    /// Write data to the buffer view.
    pub fn write_data(&self, buffer_view: &BufferView, data: &[u8]) {
        if let Some(cur_usage_buffer_blocks) = self.buffer_blocks.get(&buffer_view.usage) {
            if let Some(buffer_block) = cur_usage_buffer_blocks.get(buffer_view.block_index) {
                self.queue.write_buffer(&buffer_block.buffer.buffer, buffer_view.start, data);
                return;
            }
        }
        warn!("The buffer_view is invalid when calling write_data.")
    }

    pub fn deallocate(&mut self, buffer_view: BufferView) {
        if let Some(cur_usage_buffer_blocks) = self.buffer_blocks.get_mut(&buffer_view.usage) {
            if let Some(buffer_block) = cur_usage_buffer_blocks.get_mut(buffer_view.block_index) {
                buffer_block.deallocate(BufferBlockRange::new(buffer_view.start, buffer_view.size));
            }
        }
    }

    pub fn get_buffer_slice(&'_ self, buffer_view: &BufferView) -> wgpu::BufferSlice<'_> {
        if let Some(cur_usage_buffer_blocks) = self.buffer_blocks.get(&buffer_view.usage) {
            if let Some(buffer_block) = cur_usage_buffer_blocks.get(buffer_view.block_index) {
                let end = buffer_view.start + buffer_view.size;
                return buffer_block.buffer.buffer.slice(buffer_view.start..end);
            }
        }
        panic!("Failed to get buffer slice");
    }

    fn create_buffer_block(device: &Device, size: u64, usage: BufferUsages, label: Option<&str>) -> BufferBlock {
        // allocate new buffer block.
        let new_block_size = get_aligned_size(Self::MIN_BUFFER_SIZE, size);
        let buffer = Self::create_buffer(device, new_block_size, usage, label);
        BufferBlock::new(size, buffer)
    }

    fn create_buffer_block_init(device: &Device, size: u64, usage: BufferUsages, data: &[u8], label: Option<&str>) -> BufferBlock {
        let buffer = Self::create_buffer_init(device, usage, data, label);
        BufferBlock::new(size, buffer)
    }

    fn create_buffer(device: &Device, size: u64, usage: BufferUsages, label: Option<&str>) -> Buffer {
        let buffer = device.create_buffer(&BufferDescriptor {
            label,
            size,
            usage,
            mapped_at_creation: false,
        });
        buffer.into()
    }

    fn create_buffer_init(device: &Device, usage: BufferUsages, data: &[u8], label: Option<&str>) -> Buffer {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: data,
            usage: usage,
        });
        buffer.into()
    }
}
