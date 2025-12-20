use std::usize;

use crate::graphics::buffer::BufferUsages;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BufferView {
    pub start: u64,
    pub size: u64,
    pub block_index: usize,
    pub usage: BufferUsages,
}

impl BufferView {

    pub const INVALID: BufferView = BufferView {
        start: 0,
        size: 0,
        block_index: usize::MAX,
        usage: BufferUsages::empty(),
    };

    // pub fn new(start: u64, end: u64, buffer: RR<Buffer>, block_index: usize) -> Self {
    pub fn new(start: u64, size: u64, block_index: usize, usage: BufferUsages) -> Self {
        Self {
            start,
            size,
            // buffer,
            block_index,
            usage,
        }
    }
}