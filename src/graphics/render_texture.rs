use wgpu::{
    TextureAspect, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
};

use crate::{
    math::{Vec3, Vec4},
    prelude::ImagicContext,
    types::ID,
};

use super::texture::Texture;

pub trait RenderTexture {
    /// Get color attachment texture id.
    fn get_color_attachment_id(&self) -> ID;

    fn get_color_attachment_format(&self) -> wgpu::TextureFormat;

    /// Get depth attachment texture id.
    fn get_depth_attachment_id(&self) -> ID;

    /// Get render texture views. 2D rt has only one view, which is stored in its Texture instance.
    /// Cube Render Texture has 6 views.
    fn get_rt_views(&self) -> &[TextureView];

    fn get_width(&self) -> f32;
    fn get_height(&self) -> f32;
    fn get_physical_viewport(&self) -> Vec4;
    fn get_aspect(&self) -> f32;

    /// Only used by cube render texture, whose each face has a specific view matrix.
    /// Thre return is (camera position, camera pos, up).
    // fn get_rt_view_matrix(&self, index: usize) -> &Mat4;
    fn get_per_face_camera_params(&self, index: usize) -> (Vec3, Vec3, Vec3);
}

/// Create a depth texture, used internally.
fn create_depth_texture(imagic_context: &mut ImagicContext, width: u32, height: u32) -> ID {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;
    let dpeth_texture = Texture::create_depth_texture(
        imagic_context.graphics_context(),
        width,
        height,
        DEPTH_FORMAT,
    );
    let depth_texture = imagic_context
        .texture_manager_mut()
        .add_texture(dpeth_texture);
    depth_texture
}

pub struct RenderTexture2D {
    color_attachment: ID,
    color_attachment_format: wgpu::TextureFormat,
    color_attachment_view: [TextureView; 1],
    depth_attachment: ID,
    width: f32,
    height: f32,
}

impl RenderTexture for RenderTexture2D {
    fn get_color_attachment_id(&self) -> ID {
        self.color_attachment
    }

    fn get_color_attachment_format(&self) -> wgpu::TextureFormat {
        self.color_attachment_format
    }

    fn get_depth_attachment_id(&self) -> ID {
        self.depth_attachment
    }

    fn get_rt_views(&self) -> &[TextureView] {
        &self.color_attachment_view
    }

    fn get_width(&self) -> f32 {
        self.width
    }

    fn get_height(&self) -> f32 {
        self.height
    }

    fn get_physical_viewport(&self) -> Vec4 {
        Vec4::new(0.0, 0.0, self.width, self.height)
    }

    fn get_aspect(&self) -> f32 {
        self.width / self.height
    }

    fn get_per_face_camera_params(&self, _index: usize) -> (Vec3, Vec3, Vec3) {
        todo!()
    }
}

impl RenderTexture2D {
    pub fn new(
        imagic_context: &mut ImagicContext,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let usage = TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;
        let mut texture = Texture::create(
            imagic_context.graphics_context(),
            width,
            height,
            1,
            format,
            usage,
            1,
        );
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        texture.set_view(texture_view);

        let rt_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let color_attachment_id = imagic_context
            .texture_manager_mut()
            .add_texture(texture);

        let depth_attachment_id = create_depth_texture(imagic_context, width, height);


        Self {
            color_attachment: color_attachment_id,
            color_attachment_format: format,
            color_attachment_view: [rt_view],
            depth_attachment: depth_attachment_id,
            width: width as f32,
            height: height as f32,
        }
    }
}

pub struct CubeRenderTexture {
    color_attachment: ID,
    color_attachment_format: wgpu::TextureFormat,
    depth_attachment: ID,
    face_views: [TextureView; 6],
    face_size: f32,
    // view_matrices: [Mat4; 6],
    per_face_camera_params: [(Vec3, Vec3, Vec3); 6],
}

impl RenderTexture for CubeRenderTexture {
    fn get_color_attachment_id(&self) -> ID {
        self.color_attachment
    }

    fn get_color_attachment_format(&self) -> wgpu::TextureFormat {
        self.color_attachment_format
    }

    fn get_depth_attachment_id(&self) -> ID {
        self.depth_attachment
    }

    fn get_rt_views(&self) -> &[TextureView] {
        &self.face_views
    }

    fn get_width(&self) -> f32 {
        self.face_size
    }

    fn get_height(&self) -> f32 {
        self.face_size
    }

    fn get_physical_viewport(&self) -> Vec4 {
        Vec4::new(0.0, 0.0, self.face_size, self.face_size)
    }

    fn get_aspect(&self) -> f32 {
        1.0
    }

    fn get_per_face_camera_params(&self, index: usize) -> (Vec3, Vec3, Vec3) {
        self.per_face_camera_params[index]
    }

    // fn get_rt_view_matrix(&self, index: usize) -> &Mat4 {
    //     &self.view_matrices[index]
    // }
}

impl CubeRenderTexture {
    /// Create a cube render texture.
    pub fn new(
        imagic_context: &mut ImagicContext,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        mip_level_count: u32,
    ) -> Self {
        let usage = TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC;
        let cube_texture = Texture::create_cube_texture(
            imagic_context.graphics_context(),
            format,
            width,
            height,
            usage,
            mip_level_count,
        );

        let cube_texture_views = (0..6)
            .map(|i| {
                cube_texture.create_view(&TextureViewDescriptor {
                    label: Some(&format!("Cube Texture View {}", i)),
                    dimension: Some(TextureViewDimension::D2),
                    aspect: TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: Some(1),
                    base_array_layer: i,
                    array_layer_count: Some(1),
                    format: Some(format),
                })
            })
            .collect::<Vec<_>>();

        let face_views: [TextureView; 6] = cube_texture_views
            .try_into()
            .expect("Failed to create cube rt.");

        let color_attachment_id = imagic_context
            .texture_manager_mut()
            .add_texture(cube_texture);

        let depth_attachment_id = create_depth_texture(imagic_context, width, height);

        // let view_matrices: [Mat4; 6] = [
        //         Mat4::look_at_rh(Vec3::ZERO, Vec3::new( 1.0,  0.0,  0.0), Vec3::new(0.0, -1.0,  0.0)),
        //         Mat4::look_at_rh(Vec3::ZERO, Vec3::new(-1.0,  0.0,  0.0), Vec3::new(0.0, -1.0,  0.0)),
        //         Mat4::look_at_rh(Vec3::ZERO, Vec3::new( 0.0, -1.0,  0.0), Vec3::new(0.0,  0.0, -1.0)),
        //         Mat4::look_at_rh(Vec3::ZERO, Vec3::new( 0.0,  1.0,  0.0), Vec3::new(0.0,  0.0,  1.0)),
        //         Mat4::look_at_rh(Vec3::ZERO, Vec3::new( 0.0,  0.0,  1.0), Vec3::new(0.0, -1.0,  0.0)),
        //         Mat4::look_at_rh(Vec3::ZERO, Vec3::new( 0.0,  0.0, -1.0), Vec3::new(0.0, -1.0,  0.0))
        //     ];

        let per_face_camera_params: [(Vec3, Vec3, Vec3); 6] = [
            (
                Vec3::ZERO,
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, -1.0, 0.0),
            ),
            (
                Vec3::ZERO,
                Vec3::new(-1.0, 0.0, 0.0),
                Vec3::new(0.0, -1.0, 0.0),
            ),
            // note the follow two line (bottom and top), which is different from opengl?
            (
                Vec3::ZERO,
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(0.0, 0.0, -1.0),
            ),
            (
                Vec3::ZERO,
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            ),
            (
                Vec3::ZERO,
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(0.0, -1.0, 0.0),
            ),
            (
                Vec3::ZERO,
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(0.0, -1.0, 0.0),
            ),
        ];

        Self {
            color_attachment: color_attachment_id,
            color_attachment_format: format,
            depth_attachment: depth_attachment_id,
            face_views,
            face_size: width as f32,
            // view_matrices,
            per_face_camera_params,
        }
    }
}
