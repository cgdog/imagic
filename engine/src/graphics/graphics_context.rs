use std::rc::Rc;

use log::info;
use wgpu::{
    Adapter, Backends, BindGroupEntry, Device, Instance, Queue, Surface, SurfaceConfiguration,
};
use winit::dpi::{LogicalSize, PhysicalSize};

use crate::{
    assets::{
        sampler::{Sampler, INVALID_SAMPLER_HANDLE}, shaders::shader::ShaderPropertyPacket, textures::{texture::TextureFormat, texture_sampler_manager::TextureSamplerManager}, Texture, INVALID_TEXTURE_HANDLE
    },
    graphics::{
        bind_group::{BindGroupID, BindGroupManager},
        buffer::BufferManager,
        render_pipeline::RenderPipelineManager, uniform::{UniformValue, UniformMap},
    },
    window::Window,
};

/// Device limits, for example, min_uniform_buffer_offset_alignment.
pub struct GraphicsLimits {
    pub min_uniform_buffer_offset_alignment: u32,
    pub min_storage_buffer_offset_alignment: u32,
    pub is_support_polygon_mode_line: bool,
    pub is_support_polygon_mode_point: bool,
    /// Maximum number of bind groups that can be used in a single pipeline layout. It is at least 4. It is 8 in my computer.
    pub max_bind_groups: u32,
}

impl GraphicsLimits {
    pub(crate) fn new(device: &Device) -> Rc<Self> {
        let limits = GraphicsLimits {
            min_uniform_buffer_offset_alignment: device
                .limits()
                .min_uniform_buffer_offset_alignment,
            min_storage_buffer_offset_alignment: device
                .limits()
                .min_storage_buffer_offset_alignment,
            is_support_polygon_mode_line: device
                .features()
                .contains(wgpu::Features::POLYGON_MODE_LINE),
            is_support_polygon_mode_point: device
                .features()
                .contains(wgpu::Features::POLYGON_MODE_POINT),
            max_bind_groups: device.limits().max_bind_groups,
        };
        if cfg!(debug_assertions) {
            info!(
                "min_uniform_buffer_offset_alignment: {}, \nmin_storage_buffer_offset_alignment: {}, \nmax_bind_groups:{}",
                limits.min_uniform_buffer_offset_alignment,
                limits.min_storage_buffer_offset_alignment,
                limits.max_bind_groups,
            )
        }
        Rc::new(limits)
    }
}

#[allow(dead_code)]
pub struct GraphicsContext {
    instance: Instance,
    pub(crate) surface: Surface<'static>,
    pub(crate) surface_config: SurfaceConfiguration,
    pub(crate) device: Rc<Device>,
    pub(crate) queue: Rc<Queue>,
    pub(crate) adapter: Adapter,
    pub(crate) render_pipelines: RenderPipelineManager,
    pub(crate) bind_group_manager: BindGroupManager,
    pub(crate) buffer_manager: BufferManager,
    pub(crate) limits: Rc<GraphicsLimits>,
    _main_window: Window,
}

impl GraphicsContext {
    pub(crate) fn new(
        window: Window,
    ) -> GraphicsContext {
        let instance = wgpu::Instance::default();
        let surface_target = window.get();
        let surface_target_size = surface_target.inner_size();
        let width = surface_target_size.width.max(1);
        let height = surface_target_size.height.max(1);
        let surface = instance
            .create_surface(surface_target)
            .expect("Failed to create surface");
        // log all adapter infomation.
        if cfg!(debug_assertions) {
            info!("All available adapter info:");
            let candidate_adapters = instance.enumerate_adapters(Backends::all());
            for candidate_adapter in candidate_adapters {
                let adapter_info = candidate_adapter.get_info();
                info!("{:?}", adapter_info);
            }
        }

        let adapter = pollster::block_on(Self::async_request_adapter(&instance, &surface));

        let (device, queue) = pollster::block_on(Self::async_request_device_and_queue(&adapter));
        let limits = GraphicsLimits::new(&device);
        let surface_config = surface
            .get_default_config(&adapter, width, height)
            .expect("Failed to get surface config");
        surface.configure(&device, &surface_config);

        let device = Rc::new(device);
        let queue = Rc::new(queue);
        let buffer_manager = BufferManager::new(device.clone(), queue.clone(), limits.clone());
        let render_pipeline_manager = RenderPipelineManager::new(device.clone(), limits.clone());
       
        let graphics_context = GraphicsContext {
            instance,
            surface,
            surface_config,
            device,
            queue,
            adapter,
            render_pipelines: render_pipeline_manager,
            bind_group_manager: BindGroupManager::new(),
            buffer_manager,
            limits,
            _main_window: window,
        };
        graphics_context
    }

    async fn async_request_adapter(instance: &Instance, surface: &Surface<'static>) -> Adapter {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");
        let adpater_info = adapter.get_info();
        if cfg!(debug_assertions) {
            info!("Selected adpater info:{:#?}", adpater_info);
            let adapter_limits = adapter.limits();
            info!(
                " Adapter limits: max_bind_groups: {}, max_bindings_per_bind_group: {}, max compute workgroups per dimension: {}, max compute invocations per workgroup: {}, {}, {}, {} ",
                adapter_limits.max_bind_groups,
                adapter_limits.max_bindings_per_bind_group,
                adapter_limits.max_compute_workgroups_per_dimension,
                adapter_limits.max_compute_invocations_per_workgroup,
                adapter_limits.max_compute_workgroup_size_x,
                adapter_limits.max_compute_workgroup_size_y,
                adapter_limits.max_compute_workgroup_size_z,
            );
        }
        adapter
    }

    async fn async_request_device_and_queue(adapter: &Adapter) -> (Device, Queue) {
        let features = adapter.features();
        let mut required_features =
            wgpu::Features::TEXTURE_FORMAT_16BIT_NORM | wgpu::Features::FLOAT32_FILTERABLE;
        if features.contains(wgpu::Features::POLYGON_MODE_LINE) {
            required_features |= wgpu::Features::POLYGON_MODE_LINE;
        }
        if features.contains(wgpu::Features::POLYGON_MODE_POINT) {
            required_features |= wgpu::Features::POLYGON_MODE_POINT;
        }
        assert!(
            features.contains(required_features),
            "Adapter does not support required features"
        );

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: required_features, // wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    max_storage_buffers_per_shader_stage: 8,
                    max_storage_buffer_binding_size: 256,
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    // ..wgpu::Limits::downlevel_webgl2_defaults()
                    ..wgpu::Limits::downlevel_defaults() // To support compute shader
                        .using_resolution(adapter.limits())
                },
                // TODO: choose better memoty_hints
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                experimental_features: wgpu::ExperimentalFeatures::default(),
            })
            .await
            .expect("Failed to create device");
        (device, queue)
    }

    pub(crate) fn on_resize(&mut self, new_physical_size: PhysicalSize<u32>) {
        info!(
            "GraphicsContext on_resize: ({}, {})",
            new_physical_size.width, new_physical_size.height
        );
        let dpi = self._main_window.force_get_scale_factor();
        self.surface_config.width = new_physical_size.width.max(1);
        self.surface_config.height = new_physical_size.height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
        let new_logical_size: LogicalSize<u32> = new_physical_size.to_logical(dpi);
        self._main_window.set_physical_size(
            new_physical_size.width as f32,
            new_physical_size.height as f32,
        );
        self._main_window.set_logical_size(
            new_logical_size.width as f32,
            new_logical_size.height as f32,
        );
        self._main_window.request_redraw();
    }

    pub(crate) fn on_dpi_changed(&mut self, new_dpi: f64) {
        self._main_window.dpi = new_dpi;
    }

    pub fn dpi(&self) -> f64 {
        self._main_window.dpi
    }

    pub fn request_redraw(&self) {
        self._main_window.request_redraw();
    }

    pub fn main_window(&self) -> &Window {
        &self._main_window
    }

    pub fn get_swapchain_format(&self) -> TextureFormat {
        let swapchain_capabilities = self.surface.get_capabilities(&self.adapter);
        let swapchain_format = swapchain_capabilities.formats[0];
        swapchain_format
    }

    /// Create BindGroups for all uniforms.
    pub(crate) fn create_bind_group(
        &mut self,
        shader_property_packet: &ShaderPropertyPacket,
        uniforms: &UniformMap,
        bind_group_label: &str,
        texture_sampler_manager: &TextureSamplerManager,
    ) -> BindGroupID {
        let mut bind_group_entries = Vec::<BindGroupEntry>::new();

        for (property_name, shader_property) in &shader_property_packet.properties {
            // let group = shader_property.binding.group;
            let binding = shader_property.binding.binding;
            if let Some(uniform) = uniforms.get(property_name) {
                match &uniform.value {
                    UniformValue::Float(_, buffer_view)
                    | UniformValue::Vec2(_, buffer_view)
                    | UniformValue::Vec3(_, buffer_view)
                    | UniformValue::Vec4(_, buffer_view)
                    | UniformValue::IVec4(_, buffer_view)
                    | UniformValue::UVec4(_, buffer_view)
                    | UniformValue::Mat3(_, buffer_view)
                    | UniformValue::Mat4(_, buffer_view)
                    | UniformValue::Struct(_, buffer_view) => {
                        // info!("property_name: {}, group: {}, binding: {}", property_name, group, binding);
                        if let Some(real_buffer_view) = buffer_view {
                            let bind_group_entry = wgpu::BindGroupEntry {
                                binding,
                                resource: self
                                    .buffer_manager
                                    .get_buffer_slice(real_buffer_view)
                                    .into(),
                            };
                            bind_group_entries.push(bind_group_entry);
                        }
                    }
                    UniformValue::Sampler(sampler_handle) => {
                        let mut sampler_handle = *sampler_handle;
                        if INVALID_SAMPLER_HANDLE == sampler_handle {
                            sampler_handle = Sampler::default_sampler();
                        }
                        if let Some(sampler) = texture_sampler_manager.get_sampler(&sampler_handle)
                            && let Some(gpu_sampler) = &sampler.gpu_sampler
                        {
                            let bind_group_entry = wgpu::BindGroupEntry {
                                binding,
                                resource: wgpu::BindingResource::Sampler(gpu_sampler),
                            };
                            bind_group_entries.push(bind_group_entry);
                        }
                    }
                    UniformValue::Texture(texture_handle) => {
                        let mut texture_handle = *texture_handle;
                        if INVALID_TEXTURE_HANDLE == texture_handle {
                            texture_handle = Texture::white();
                        }
                        if let Some(texture) = texture_sampler_manager.get_texture(&texture_handle)
                            && let Some(texture_view) = &texture.view
                        {
                            let bind_group_entry = wgpu::BindGroupEntry {
                                binding,
                                resource: wgpu::BindingResource::TextureView(&texture_view.view),
                            };
                            bind_group_entries.push(bind_group_entry);
                        }
                    }
                }
            } else {
                log::warn!(
                    "Shader property {} is not found in uniform properties when crate bind group.",
                    property_name
                );
            }
        }
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(bind_group_label),
            layout: shader_property_packet.bind_group_layout.as_ref().unwrap(),
            entries: &bind_group_entries,
        });
        let bind_group_id = self.bind_group_manager.add(bind_group);
        bind_group_id
    }
}
