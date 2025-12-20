/// Generates Spherical Harmonics (SH) coefficients from a cube map texture.
/// Stealed from Babylon.js, licensed by Apache-2.0.
/// https://github.com/BabylonJS/Babylon.js/blob/master/packages/dev/core/src/Misc/HighDynamicRange/cubemapToSphericalPolynomial.ts

use crate::{
    assets::{Texture, TextureHandle, TextureSamplerManager},
    math::{Vec3, spherical_harmonics::SphericalHarmonics},
    prelude::{
        graphics_context::GraphicsContext,
        texture_buffer_converter::{CubeTextureData, read_cube_texture},
    },
};

#[allow(unused)]
pub(crate) struct FileFaceOrientation {
    pub(crate) name: String,
    pub(crate) world_axis_for_normal: Vec3, // the world axis corresponding to the normal to the face
    pub(crate) world_axis_for_file_x: Vec3, // the world axis corresponding to texture right x-axis in file
    pub(crate) world_axis_for_file_y: Vec3, // the world axis corresponding to texture down y-axis in file
}

impl FileFaceOrientation {
    pub fn new(
        name: &str,
        world_axis_for_normal: Vec3,
        world_axis_for_file_x: Vec3,
        world_axis_for_file_y: Vec3,
    ) -> Self {
        Self {
            name: name.to_string(),
            world_axis_for_normal,
            world_axis_for_file_x,
            world_axis_for_file_y,
        }
    }
}

pub struct CubeMapToSHTools {
    _file_faces: [FileFaceOrientation; 6],
}

impl CubeMapToSHTools {
    pub fn new() -> Self {
        let file_faces: [FileFaceOrientation; 6] = [
            FileFaceOrientation::new(
                "right",
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(0.0, -1.0, 0.0),
            ), // +X east
            FileFaceOrientation::new(
                "left",
                Vec3::new(-1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(0.0, -1.0, 0.0),
            ), // -X west
            FileFaceOrientation::new(
                "up",
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            ), // +Y north
            FileFaceOrientation::new(
                "down",
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, -1.0),
            ), // -Y south
            FileFaceOrientation::new(
                "front",
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, -1.0, 0.0),
            ), // +Z top
            FileFaceOrientation::new(
                "back",
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(-1.0, 0.0, 0.0),
                Vec3::new(0.0, -1.0, 0.0),
            ), // -Z bottom
        ];
        Self {
            _file_faces: file_faces,
        }
    }
}

impl CubeMapToSHTools {
    const MAX_HDRI_VALUE: f32 = 4096.0;
    pub fn generate_sh_from_cube_texture_handle(
        &self,
        cube_texture_handle: &TextureHandle,
        graphics_context: &GraphicsContext,
        texture_sampler_manager: &mut TextureSamplerManager,
    ) -> [Vec3; 9] {
        if let Some(cube_texture) = texture_sampler_manager.get_texture(cube_texture_handle)
        {
            self.generate_sh(cube_texture, graphics_context)
        } else {
            log::error!("cube texture does not exist");
            [Vec3::ZERO; 9]
        }
    }

    pub fn generate_sh(
        &self,
        cube_texture: &Texture,
        graphics_context: &GraphicsContext,
    ) -> [Vec3; 9] {
        if cube_texture.size.depth_or_array_layers != 6 {
            panic!("The input is not cube texture when generating SH!");
        }
        let cube_texture_content = read_cube_texture(graphics_context, cube_texture);
        match cube_texture_content {
            Ok(content) => match content {
                CubeTextureData::U8(cube_data) => {
                    let cube_data_float = cube_data
                        .iter()
                        .map(|face_data| {
                            face_data
                                .chunks(4)
                                .flat_map(|pixel| {
                                    [
                                        (pixel[0] as f32 / 255.0).powf(2.2),
                                        (pixel[1] as f32 / 255.0).powf(2.2),
                                        (pixel[2] as f32 / 255.0).powf(2.2),
                                    ]
                                })
                                .collect::<Vec<f32>>()
                        })
                        .collect::<Vec<Vec<f32>>>();
                    self.generate_sh_from_f32(cube_data_float, cube_texture.size.width, 3)
                }
                CubeTextureData::F32(cube_data) => {
                    self.generate_sh_from_f32(cube_data, cube_texture.size.width, 4)
                }
                CubeTextureData::U16(_cube_data) => todo!(),
                CubeTextureData::I32(_cube_data) => todo!(),
            },
            Err(e) => panic!("{}", e),
        }
    }

    fn generate_sh_from_f32(&self, cube_data: Vec<Vec<f32>>, size: u32, stride: u32) -> [Vec3; 9] {
        let mut spherical_harmonics = SphericalHarmonics::new();
        let mut total_solid_angle = 0.0;
        let du = 2.0 / size as f32;
        let dv = du;

        let half_texel = 0.5 * du;

        let min_uv = half_texel - 1.0;

        for face_index in 0..6 {
            let file_face = &self._file_faces[face_index];
            let data_array = &cube_data[face_index];
            let mut v = min_uv;
            for y in 0..size {
                let mut u = min_uv;
                for x in 0..size {
                    let mut world_direction = (file_face.world_axis_for_file_x * u) + (file_face.world_axis_for_file_y * v) + file_face.world_axis_for_normal;
                    world_direction = world_direction.normalize();
                    let delta_solid_angle =
                        self._area_element(u - half_texel, v - half_texel) -
                        self._area_element(u - half_texel, v + half_texel) -
                        self._area_element(u + half_texel, v - half_texel) +
                        self._area_element(u + half_texel, v + half_texel);

                    let mut r = data_array[(y * size * stride + x * stride + 0) as usize];
                    let mut g = data_array[(y * size * stride + x * stride + 1) as usize];
                    let mut b = data_array[(y * size * stride + x * stride + 2) as usize];

                    if r.is_nan() {
                        r = 0.0;
                    }
                    if g.is_nan() {
                        g = 0.0;
                    }
                    if b.is_nan() {
                        b = 0.0;
                    }
                    r = r.clamp(0.0, Self::MAX_HDRI_VALUE);
                    g = g.clamp(0.0, Self::MAX_HDRI_VALUE);
                    b = b.clamp(0.0, Self::MAX_HDRI_VALUE);

                    let color = Vec3::new(r, g, b);
                    spherical_harmonics.add_light(&world_direction, &color, delta_solid_angle);
                    total_solid_angle += delta_solid_angle;

                    u += du;
                }
                v += dv;
            }
        }

        // Solid angle for entire sphere is 4*pi
        const SPHERE_SOLID_ANGLE: f32 = 4.0 * std::f32::consts::PI;

        // Adjust the solid angle to allow for how many faces we processed.
        const FACES_PROCESSED: f32 = 6.0;
        const EXPECTED_SOLID_ANGLE: f32 = (SPHERE_SOLID_ANGLE * FACES_PROCESSED) / 6.0;

        // Adjust the harmonics so that the accumulated solid angle matches the expected solid angle.
        // This is needed because the numerical integration over the cube uses a
        // small angle approximation of solid angle for each texel (see deltaSolidAngle),
        // and also to compensate for accumulative error due to float precision in the summation.
        let correction_factor = EXPECTED_SOLID_ANGLE / total_solid_angle;
        spherical_harmonics.scale_in_place(correction_factor);
        spherical_harmonics.convert_incident_radiance_to_irradiance();
        spherical_harmonics.convert_irradiance_to_lambertian_radiance();
        spherical_harmonics.pre_scale_for_rendering(); // needed

        [
            spherical_harmonics.l00,
            spherical_harmonics.l1_1,
            spherical_harmonics.l10,
            spherical_harmonics.l11,
            spherical_harmonics.l2_2,
            spherical_harmonics.l2_1,
            spherical_harmonics.l20,
            spherical_harmonics.l21,
            spherical_harmonics.l22
         ]
    }

    /* 
     * Compute the area on the unit sphere of the rectangle defined by (x,y) and the origin
     * See https://www.rorydriscoll.com/2012/01/15/cubemap-texel-solid-angle/
     */
    fn _area_element(&self, x: f32, y: f32) -> f32 {
        (x * y).atan2((x * x + y * y + 1.0).sqrt())
    }
}
