//！**Imagic** is a thin rendering framework, implemented by Rust and WGPU.
//！It is designed to be simple, efficient, and easy to use.


pub mod core;
pub mod graphics;
pub mod math;
pub mod assets;
pub mod components;
pub mod renderer;
pub mod behaviors;
pub mod types;
pub mod window;
pub mod input;
pub mod event;
pub mod utils;
pub mod time;

pub mod prelude {
    pub use crate::assets::*;
    pub use crate::behaviors::*;
    pub use crate::components::*;
    pub use crate::core::*;
    pub use crate::math::*;
    pub use crate::types::*;
    pub use crate::utils::*;
    pub use crate::window::*;
    pub use crate::time::*;
    pub use crate::graphics::*;
}