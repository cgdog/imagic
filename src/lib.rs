pub mod graphics;
pub mod window;
pub mod imagic_core;
pub use imagic_core::Imagic;
pub mod ui;
pub mod model;
pub mod scene;
pub mod camera;
pub mod material;
pub mod light;
pub mod asset;
pub mod constants;
pub mod math;
pub mod types;
pub mod input;
pub mod utils;
pub mod ecs;

pub mod prelude {
    pub use crate::graphics::*;
    pub use crate::graphics::texture::*;
    pub use crate::imagic_core::*;
    pub use crate::imagic_core::core::*;
    pub use crate::imagic_core::imagic_app::*;
    pub use crate::imagic_core::render_item::*;
    pub use crate::imagic_core::imagic_context::*;
    pub use crate::ui::ui_renderer::*;
    pub use crate::model::*;
    pub use crate::scene::*;
    pub use crate::camera::*;
    pub use crate::material::*;
    pub use crate::light::*;
    pub use crate::asset::*;
    pub use crate::constants::*;
    pub use crate::math::*;
    pub use crate::types::*;
    pub use crate::input::*;
    pub use crate::utils::*;
}

pub use wgpu;
pub use winit;
pub use egui;
pub use glam;
pub use image;
pub use naga_oil;
pub use log;
pub use env_logger;