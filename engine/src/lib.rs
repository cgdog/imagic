//！**Imagic** is a thin rendering framework, implemented by Rust and WGPU.
//！It is designed to be simple, efficient, and easy to use.
//! At the moment, Imagic only supports a few features:
//! - Materials
//!    - PBR Material which only supports IBL (image based lighting) now.
//!    - Unlit Material.
//!    - Custom Material.
//! > In fact, there is only one Material struct. PBR, Unlit or custom Materials are all implemented by this Material. The differences are the shaders. The Material will parse the shader source to know what uniforms (including their groups and bindings) are needed. So it is easy to custom shader or material.
//! - HDR textures
//! - Multi camera
//! - Render textures, both 2D and Cube
//! - Integrated UI framework powered by EGUI
//! - Support gltf 2.0 partially
//! - Perspective camera and orbit camera controller.
//! 
//! # Examples
//! There are some examples in folder [`engine\examples\`](https://github.com/cgdog/imagic/tree/main/engine/examples) on GitHub.
//! 
//! For example, you can run `cargo run --example gltf_demo` to see the [`gltf_demo.rs`](https://github.com/cgdog/imagic/blob/main/engine/examples/gltf_demo.rs) example:
//! > - hold left mouse button down and move mouse to rotate camera.
//! > - hold right mouse button down and move mouse to zoom in and out.
//! 
//! Here are some tips:
//! ## The basic boilerplate
//! 
//! ```rust
//! fn main() {
//!     // 1. create the engine instance, which is the core API.
//!     let options = EngineOptions {
//!         window_size: WindowSize::new(800.0, 500.0),
//!         app_name: "lxy gltf demo",
//!     };
//!     let mut engine = Engine::new(options);
//!     // 2. add nodes to scene or add Behaviors, or anything else.
//!     load_model(&mut engine);
//!     create_camera(&mut engine);
//!     add_skybox(&mut engine);
//!     
//!     // 3. launch the engine.
//!     engine.run();
//! }
//! ```
//! 
//! For more details, see the [`primitives_demo.rs`](https://github.com/cgdog/imagic/blob/main/engine/examples/primitives_demo.rs),
//!  [`gltf_demo.rs`](https://github.com/cgdog/imagic/blob/main/engine/examples/gltf_demo.rs), 
//! [`ibl_demo.rs`](https://github.com/cgdog/imagic/blob/main/engine/examples/ibl_demo.rs), 
//! [`unlit_demo.rs`](https://github.com/cgdog/imagic/blob/main/engine/examples/unlit_demo.rs) or other examples.
//! 

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