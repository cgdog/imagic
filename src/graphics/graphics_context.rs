use crate::window::window_core::Window;
use log::info;
use wgpu::{
    util::DeviceExt, Adapter, BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, CommandBuffer,
    CommandEncoder, Device, Instance, PipelineLayout, Queue, RenderPipeline, ShaderModule,
    SubmissionIndex, TextureFormat,
};
use winit::dpi::PhysicalSize;

use super::SurfaceWrapper;

#[derive(Default)]
pub struct GraphicsContext {
    instance: Instance,
    surface: SurfaceWrapper,
    device: Option<Device>,
    queue: Option<Queue>,
    adapter: Option<Adapter>,
}

impl GraphicsContext {
    pub fn new() -> Self {
        Self {
            instance: wgpu::Instance::default(),
            ..Default::default()
        }
    }

    pub async fn init(&mut self, window: &Window) {
        let surface = self
            .instance
            .create_surface(window.get())
            .expect("Instance failed to create surface.");
        self.surface.set(Some(surface));

        let adapter = self
            .instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(self.surface.get()),
            })
            .await
            .expect("Failed to find an appropriate adapter");
        let adpater_info = adapter.get_info();
        info!("adpater_info:{:#?}", adpater_info);

        let adapter_limits = adapter.limits();
        info!(
            "max_bind_groups: {}, max_bindings_per_bind_group: {}, max compute workgroups per dimension: {}, max compute invocations per workgroup: {}, {}, {}, {} ",
            adapter_limits.max_bind_groups, adapter_limits.max_bindings_per_bind_group,
            adapter_limits.max_compute_workgroups_per_dimension,
            adapter_limits.max_compute_invocations_per_workgroup,
            adapter_limits.max_compute_workgroup_size_x,
            adapter_limits.max_compute_workgroup_size_y,
            adapter_limits.max_compute_workgroup_size_z,
        );

        let features = adapter.features();

        let required_features =
            wgpu::Features::TEXTURE_FORMAT_16BIT_NORM | wgpu::Features::FLOAT32_FILTERABLE;
        assert!(
            features.contains(required_features),
            "Adapter does not support required features"
        );

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
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
                    // required_limits: wgpu::Limits::downlevel_defaults(),
                    // TODO: choose better memoty_hints
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        self.device = Some(device);
        self.queue = Some(queue);
        let mut size = window.get().inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);
        self.surface
            .retrieve_default_config(&adapter, size.width, size.height);
        self.adapter = Some(adapter);
        // self.on_resize(new_size);
        self.surface.configure(self.get_device());
    }

    pub fn get_swapchain_format(&self) -> TextureFormat {
        let swapchain_capabilities = self
            .surface
            .get_capabilities(self.adapter.as_ref().unwrap());
        let swapchain_format = swapchain_capabilities.formats[0];
        swapchain_format
    }

    pub fn create_shader_module(
        &self,
        shader_module_descriptor: wgpu::ShaderModuleDescriptor,
    ) -> ShaderModule {
        self.get_device()
            .create_shader_module(shader_module_descriptor)
    }

    pub fn create_pipeline_layout(
        &self,
        pipeline_layout_desc: &wgpu::PipelineLayoutDescriptor,
    ) -> PipelineLayout {
        self.get_device()
            .create_pipeline_layout(pipeline_layout_desc)
    }

    pub fn create_render_pipeline(
        &self,
        render_pipeline_desc: &wgpu::RenderPipelineDescriptor,
    ) -> RenderPipeline {
        self.get_device()
            .create_render_pipeline(render_pipeline_desc)
    }

    pub fn on_resize(&mut self, new_size: PhysicalSize<u32>) {
        self.surface.resize(new_size);
        self.surface.configure(self.get_device());
    }

    pub fn get_surface(&self) -> &SurfaceWrapper {
        &self.surface
    }

    pub fn create_command_encoder(
        &self,
        cmd_encoder_desc: &wgpu::CommandEncoderDescriptor,
    ) -> CommandEncoder {
        self.get_device().create_command_encoder(cmd_encoder_desc)
    }

    pub fn submit(&self, cmd_buffer: Option<CommandBuffer>) -> SubmissionIndex {
        self.get_queue().submit(cmd_buffer)
    }

    pub fn create_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDescriptor<'_>,
    ) -> BindGroupLayout {
        self.get_device().create_bind_group_layout(desc)
    }

    pub fn create_bind_group(&self, bind_group_desc: &wgpu::BindGroupDescriptor) -> BindGroup {
        self.get_device().create_bind_group(bind_group_desc)
    }

    pub fn get_device(&self) -> &Device {
        self.device.as_ref().expect("The device is None.")
    }

    pub fn get_queue(&self) -> &Queue {
        self.queue.as_ref().expect("The queue is None.")
    }

    pub fn create_buffer_init(
        &self,
        buffer_desc: &wgpu::util::BufferInitDescriptor,
    ) -> wgpu::Buffer {
        self.get_device().create_buffer_init(buffer_desc)
    }
}
