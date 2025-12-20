use std::any::Any;

use crate::core::LogicContext;

/// SystemBehavior is a trait for behaviors that operate at the system level.
/// It provides lifecycle methods that are called at different stages of the application's existence.
#[allow(unused)]
pub trait SystemBehavior : Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    fn on_start(&mut self, logic_context: &mut LogicContext) {

    }
    fn on_update(&mut self, logic_context: &mut LogicContext) {

    }
    fn on_destroy(&mut self, logic_context: &mut LogicContext) {
        
    }
    #[allow(unused)]
    fn on_gui(&mut self, logic_context: &mut LogicContext, ui_context: &egui::Context) {

    }
}