use crate::ecs::world::World;

use super::core::ImagicOption;

pub trait ImagicAppTrait {
    fn init(&mut self, world: &mut World);

    fn get_imagic_option(& self) -> ImagicOption;
    
    #[allow(unused)]
    fn on_update(&mut self, world: &mut World) {

    }

    #[allow(unused)]
    fn on_render_ui(&mut self, ui_context: &egui::Context) {
        
    }
}