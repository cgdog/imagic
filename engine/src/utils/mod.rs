pub mod sparse_set;
pub mod hash_utils;
pub mod skybox_builder;
pub mod image_utils;
pub mod sh_tools;
pub mod performance;

pub use skybox_builder::*;
pub use performance::*;

/// get a aligned size value. `alignment` is power of 2.
pub fn get_aligned_size(alignment: u64, size: u64) -> u64 {
    // let remainder = size % alignment;
    let remainder = size & (alignment - 1);
    let aligned_size = if remainder == 0 {
        size
    } else {
        size + alignment - remainder
    };
    aligned_size
}