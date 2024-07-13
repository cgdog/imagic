use crate::ui::ui_renderer::UIRenderer;

use super::imagic_context::ImagicContext;

pub trait ImagicAppTrait {
    fn on_update(&mut self, _imagic_context: &mut ImagicContext, _ui_renderer: &mut UIRenderer) {

    }
    fn on_render_ui(&mut self, _ctx: &egui::Context) {
        
    }
}