use super::{core::ImagicOption, imagic_context::ImagicContext, Imagic};

pub trait ImagicAppTrait {
    fn init(&mut self, imagic: &mut Imagic);

    fn get_imagic_option(& self) -> ImagicOption;
    
    fn on_update(&mut self, _imagic_context: &mut ImagicContext) {

    }

    fn on_render_ui(&mut self, _ctx: &egui::Context) {
        
    }
}