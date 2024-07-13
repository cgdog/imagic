use wgpu::{Adapter, Device, Surface, SurfaceCapabilities, SurfaceConfiguration, SurfaceTexture};
use winit::dpi::PhysicalSize;

#[derive(Default)]
pub struct SurfaceWrapper {
    surface: Option<Surface<'static>>,
    config: Option<SurfaceConfiguration>,
}

impl SurfaceWrapper {
    pub fn get(&self) -> &Surface {
        match &self.surface {
            Some(surface) => surface,
            None => panic!("Surface is None"),
        }
    }

    pub fn set(&mut self, surface: Option<Surface<'static>>) {
        self.surface = surface;
    }

    pub fn get_config(&self) -> &SurfaceConfiguration {
        match &self.config {
            Some(config) => config,
            None => panic!("Surface config is None"),
        }
    }

    pub fn get_config_mut(&mut self) -> &mut SurfaceConfiguration {
        match &mut self.config {
            Some(config) => config,
            None => panic!("Surface config is None"),
        }
    }

    pub fn configure(&self, device: &Device) {
        self.get().configure(device, self.get_config());
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let config = self.get_config_mut();
        config.width = new_size.width.max(1);
        config.height = new_size.height.max(1);
    }

    pub fn get_cur_size(&self) -> [u32; 2] {
        let config = self.get_config();
        [config.width, config.height]
    }

    pub fn get_current_texture(&self) -> SurfaceTexture {
        let surface_texture = self
            .get()
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        surface_texture
    }

    pub fn get_capabilities(&self, adapter: &Adapter) -> SurfaceCapabilities {
        self.get().get_capabilities(adapter)
    }

    pub fn retrieve_default_config(&mut self, adapter: &Adapter, width: u32, height: u32) {
        let config = self
            .get()
            .get_default_config(adapter, width, height)
            .unwrap();
        self.config = Some(config);
    }
}
