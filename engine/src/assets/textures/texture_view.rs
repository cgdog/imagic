#[derive(PartialEq, Clone, Debug)]
pub struct TextureView {
    pub(crate) view: wgpu::TextureView,
}

impl TextureView {
}

impl From<wgpu::TextureView> for TextureView {
    fn from(value: wgpu::TextureView) -> Self {
        Self {
            view: value,
        }        
    }
}