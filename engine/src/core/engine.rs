use crate::{
    assets::{ShaderManager, TextureSamplerManager},
    core::{World},
    event::{event_dispatcher::EventDispatcher, events::Events},
    graphics::graphics_context::GraphicsContext,
    input::input_manager::InputManager,
    prelude::{
        Behavior,
        behavior_wrapper::{BehaviorStatus, BehaviorWrapper}, uniform::BuiltinUniforms,
    },
    renderer::frame_renderer::FrameRenderer,
    time::Time,
    types::RR,
    utils::PerformanceTracker,
    window::{WindowSize, window_input_processor::WindowInputProcessor},
};

pub struct EngineOptions {
    pub window_size: WindowSize,
    pub app_name: &'static str,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            window_size: WindowSize::default(),
            app_name: "Imagic Engine",
        }
    }
}

pub struct LogicContext<'a> {
    pub world: &'a mut World,
    pub time: &'a mut Time,
    pub performance_tracker: &'a mut PerformanceTracker,
    pub shader_manager: &'a mut ShaderManager,
    pub texture_sampler_manager: &'a mut TextureSamplerManager,
    pub input_manager: &'a mut InputManager,
}

pub struct Engine {
    pub options: EngineOptions,
    pub world: World,
    pub time: Time,
    pub event_dispatcher: RR<EventDispatcher>,
    pub performance_tracker: PerformanceTracker,
    pub input_manager: InputManager,
    pub shader_manager: ShaderManager,
    pub texture_sampler_manager: TextureSamplerManager,
    pub(crate) frame_renderer: FrameRenderer,
    pub(crate) global_uniforms: BuiltinUniforms,
    pub(crate) _is_inited: bool,
    pub(crate) _window_input_processor: WindowInputProcessor,
    pub(crate) _graphics_context: Option<GraphicsContext>,
    _behavior_wrappers: Vec<BehaviorWrapper>,
}

impl Engine {
    /// Create the default Engine instance.
    pub fn default() -> Box<Engine> {
        let options = EngineOptions::default();
        Self::new(options)
    }

    /// Create an Engine instance, which is allocated in Heap memory.
    ///
    /// You can access current World by Engine.
    pub fn new(options: EngineOptions) -> Box<Engine> {
        let engine = Engine {
            options,
            world: World::new(),
            time: Time::new(),
            event_dispatcher: EventDispatcher::new(),
            performance_tracker: PerformanceTracker::new(),
            input_manager: InputManager::new(),
            shader_manager: ShaderManager::new(),
            texture_sampler_manager: TextureSamplerManager::new(),
            frame_renderer: FrameRenderer::new(),
            global_uniforms: BuiltinUniforms::new("Global".to_owned()),
            _graphics_context: None,
            _window_input_processor: WindowInputProcessor::new(),
            _behavior_wrappers: vec![],
            _is_inited: false,
        };
        let engine_in_heap = Box::new(engine);
        engine_in_heap
    }

    /// Launch the Engine instance and run the game loop.
    pub fn run(&mut self) {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let _ = event_loop.run_app(self);
    }

    pub(crate) fn graphics_context(&mut self) -> &mut GraphicsContext {
        if let Some(context) = &mut self._graphics_context {
            context
        } else {
            panic!(
                "GraphicsContext fo Engine has not been instantiated. Just use it after the run function of Application has been called. (When world is inited)"
            )
        }
    }

    pub(crate) fn init(&mut self, mut graphics_context: GraphicsContext) {
        self.texture_sampler_manager.init(graphics_context.device.clone(), graphics_context.queue.clone());
        self.world.on_init(&mut graphics_context, &mut self.texture_sampler_manager, &mut self.time);
        self.execute_behaviors();
        self._graphics_context = Some(graphics_context);
    }

    pub(crate) fn stop(&mut self) {
        self.remove_all_behavior();
        self.world.stop(&mut self.time);
        self.event_dispatcher
            .borrow_mut()
            .emit(Events::EVENT_WINDOW_CLOSED);
    }

    pub(crate) fn on_resize(&mut self, new_physical_size: winit::dpi::PhysicalSize<u32>) {
        match &mut self._graphics_context {
            Some(context) => {
                context.on_resize(new_physical_size);
                self.world.on_resize(context, &mut self.texture_sampler_manager);
            }
            None => unreachable!(),
        }
    }

    pub(crate) fn on_update(&mut self) {
        self.time.on_update();
        self.performance_tracker.on_update(self.time.time_data.x);
        self.execute_behaviors();
        self.world.updpate(&mut self.time);
        self.render();
    }

    pub(crate) fn render(&mut self) {
        match &mut self._graphics_context {
            Some(graphics_context) => {
                self.world.generate_render_frame(graphics_context, &mut self.texture_sampler_manager,
                    &mut self.time, &mut self.frame_renderer, &mut self.global_uniforms);
                let mut logic_context = LogicContext {
                    world: &mut self.world,
                    time: &mut self.time,
                    performance_tracker: &mut self.performance_tracker,
                    shader_manager: &mut self.shader_manager,
                    texture_sampler_manager: &mut self.texture_sampler_manager,
                    input_manager: &mut self.input_manager,
                };
                self.frame_renderer.render(&mut logic_context, graphics_context, &mut self._behavior_wrappers);
                self.graphics_context().request_redraw();
            }
            None => unreachable!(),
        }
    }

    pub fn add_behavior<T: 'static + Behavior>(&mut self, behavior: T) {
        self._behavior_wrappers
            .push(BehaviorWrapper::new(Box::new(behavior)));
    }

    pub fn remove_behavior<T: 'static + Behavior>(&mut self) {
        self._behavior_wrappers.retain_mut(|behavior_wrapper| {
            if let Some(_behavior) = behavior_wrapper.behavior.as_any_mut().downcast_mut::<T>() {
                let mut logic_context = LogicContext {
                    world: &mut self.world,
                    time: &mut self.time,
                    performance_tracker: &mut self.performance_tracker,
                    shader_manager: &mut self.shader_manager,
                    texture_sampler_manager: &mut self.texture_sampler_manager,
                    input_manager: &mut self.input_manager,
                };
                behavior_wrapper.on_destroy(&mut logic_context);
                false
            } else {
                true
            }
        });
    }

    pub fn remove_all_behavior(&mut self) {
        let mut logic_context = LogicContext {
            world: &mut self.world,
            time: &mut self.time,
            performance_tracker: &mut self.performance_tracker,
            shader_manager: &mut self.shader_manager,
            texture_sampler_manager: &mut self.texture_sampler_manager,
            input_manager: &mut self.input_manager,
        };
        for behavior in &mut self._behavior_wrappers {
            behavior.on_destroy(&mut logic_context);
        }
        self._behavior_wrappers.clear();
    }

    pub(crate) fn execute_behaviors(&mut self) {
        for behavior_wrapper in &mut self._behavior_wrappers {
            let mut logic_context = LogicContext {
                world: &mut self.world,
                time: &mut self.time,
                performance_tracker: &mut self.performance_tracker,
                shader_manager: &mut self.shader_manager,
                texture_sampler_manager: &mut self.texture_sampler_manager,
                input_manager: &mut self.input_manager,
            };
            match behavior_wrapper.status {
                BehaviorStatus::Start => {
                    behavior_wrapper.status = BehaviorStatus::Update;
                    behavior_wrapper.on_start(&mut logic_context);
                }
                BehaviorStatus::Update => {
                    behavior_wrapper.on_update(&mut logic_context);
                }
            }
        }
    }
}