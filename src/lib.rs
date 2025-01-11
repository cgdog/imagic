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
pub mod asset_loader;
pub mod constants;
pub mod math;

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
    pub use crate::asset_loader::*;
    pub use crate::constants::*;
    pub use crate::math;
}