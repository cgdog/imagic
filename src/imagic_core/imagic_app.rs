use super::{core::ImagicOption, imagic_context::ImagicContext};

pub trait ImagicAppTrait {
    fn init(&mut self, imagic: &mut ImagicContext);

    fn get_imagic_option(& self) -> ImagicOption;
    
    #[allow(unused)]
    fn on_update(&mut self, imagic_context: &mut ImagicContext) {

    }

    #[allow(unused)]
    fn on_render_ui(&mut self, ui_context: &egui::Context) {
        
    }
}