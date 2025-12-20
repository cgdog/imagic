use crate::{behaviors::behavior::{SystemBehavior}, core::LogicContext};

pub(crate) enum BehaviorStatus {
    Start,
    Update,
}

pub(crate) struct BehaviorWrapper {
    pub(crate) behavior: Box<dyn SystemBehavior>,
    pub(crate) status: BehaviorStatus,
}

impl BehaviorWrapper {
    pub(crate) fn new(behavior: Box<dyn SystemBehavior>) -> Self {
        Self {
            behavior,
            status: BehaviorStatus::Start,
        }
    }

    pub(crate) fn on_start(&mut self, logic_context: &mut LogicContext) {
        self.behavior.on_start(logic_context);
    }
    pub(crate) fn on_update(&mut self, logic_context: &mut LogicContext) {
        self.behavior.on_update(logic_context);
    }
    pub(crate) fn on_destroy(&mut self, logic_context: &mut LogicContext) {
        self.behavior.on_destroy(logic_context);
    }
    #[allow(unused)]
    pub(crate) fn on_gui(&mut self, logic_context: &mut LogicContext, ui_context: &egui::Context) {
        self.behavior.on_gui(logic_context, ui_context);
    }
}