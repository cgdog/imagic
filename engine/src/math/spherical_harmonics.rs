use crate::math::Vec3;

/// Spherical Harmonics up to 3nd order
/// Stealed from Babylon.js, licensed by Apache-2.0.
#[derive(Default, Debug, Clone, Copy)]
pub struct SphericalHarmonics {
    /**
     * The l0,0 coefficients of the spherical harmonics
     */
    pub l00: Vec3,

    /**
     * The l1,-1 coefficients of the spherical harmonics
     */
    pub l1_1: Vec3,

    /**
     * The l1,0 coefficients of the spherical harmonics
     */
    pub l10: Vec3,

    /**
     * The l1,1 coefficients of the spherical harmonics
     */
    pub l11: Vec3,

    /**
     * The l2,-2 coefficients of the spherical harmonics
     */
    pub l2_2: Vec3,

    /**
     * The l2,-1 coefficients of the spherical harmonics
     */
    pub l2_1: Vec3,

    /**
     * The l2,0 coefficients of the spherical harmonics
     */
    pub l20: Vec3,

    /**
     * The l2,1 coefficients of the spherical harmonics
     */
    pub l21: Vec3,

    /**
     * The l2,2 coefficients of the spherical harmonics
     */
    pub l22: Vec3,
}


const SH3YLM_BASIS_CONSTANTS: [f32; 9] = [
    // l00 = sqrt(1 / (4 * PI))
    0.28209479177387814,
    
    // l1_1 = -sqrt(3 / (4 * PI))
    -0.4886025119029199,
    // l10 = sqrt(3 / (4 * PI))
    0.4886025119029199,
    // l11 = -sqrt(3 / (4 * PI))
    -0.4886025119029199,
    
    // l2_2 = sqrt(15 / (4 * PI))
    1.0925484305920792,
    // l2_1 = -sqrt(15 / (4 * PI))
    -1.0925484305920792,
    // l20 = sqrt(5 / (16 * PI))
    0.31539156525252005,
    // l21 = -sqrt(15 / (4 * PI))
    -1.0925484305920792,
    // l22 = sqrt(15 / (16 * PI))
    0.5462742152960396,
];

type ShBasisFn = fn(&Vec3) -> f32;

const SH3YLM_BASIS_TRIGONOMETRIC_TERMS: [ShBasisFn; 9] = [
    // l00
    |_direction| 1.0,
    
    // l1_1
    |direction| direction.y,
    // l10
    |direction| direction.z,
    // l11
    |direction| direction.x,
    
    // l2_2
    |direction| direction.x * direction.y,
    // l2_1
    |direction| direction.y * direction.z,
    // l20
    |direction| 3.0 * direction.z * direction.z - 1.0,
    // l21
    |direction| direction.x * direction.z,
    // l22
    |direction| direction.x * direction.x - direction.y * direction.y,
];

// Derived from the integration of the a kernel convolution to SH.
// Great explanation here: https://patapom.com/blog/SHPortal/#about-distant-radiance-and-irradiance-environments
const SHCOS_KERNEL_CONVOLUTION: [f32; 9] = [
    std::f32::consts::PI,
    (2.0 * std::f32::consts::PI) / 3.0,
    (2.0 * std::f32::consts::PI) / 3.0,
    (2.0 * std::f32::consts::PI) / 3.0,
    std::f32::consts::PI / 4.0,
    std::f32::consts::PI / 4.0,
    std::f32::consts::PI / 4.0,
    std::f32::consts::PI / 4.0,
    std::f32::consts::PI / 4.0
];


// Wrap the full compute
fn apply_sh3(lm: usize, direction: &Vec3) -> f32 {
    SH3YLM_BASIS_CONSTANTS[lm] * SH3YLM_BASIS_TRIGONOMETRIC_TERMS[lm](direction)
}

impl SphericalHarmonics {
    /// Creates a new SphericalHarmonics object
    pub fn new() -> Self {
        Self::default()
    }

    // Keep for references.
    /**
     * Gets the spherical harmonics from polynomial
     * @param polynomial the spherical polynomial
     * @returns the spherical harmonics
     */
    pub fn from_polynomial(polynomial: SphericalPolynomial) -> SphericalHarmonics {
        let mut result = SphericalHarmonics::new();

        result.l00 = polynomial.xx * 0.376127 + (polynomial.yy * 0.376127) + (polynomial.zz * 0.376126);
        result.l1_1 = polynomial.y * 0.977204;
        result.l10 = polynomial.z * 0.977204;
        result.l11 = polynomial.x * 0.977204;
        result.l2_2 = polynomial.xy * 1.16538;
        result.l2_1 = polynomial.yz * 1.16538;
        result.l20 = polynomial.zz * 1.34567 - (polynomial.xx * 0.672834) - (polynomial.yy * 0.672834);
        result.l21 = polynomial.zx * 1.16538;
        result.l22 = polynomial.xx * 1.16538 - (polynomial.yy * 1.16538);

        result.l1_1 = -result.l1_1;
        result.l11 = -result.l11;
        result.l2_1 = -result.l2_1;
        result.l21 = -result.l21;

        result.scale_in_place(std::f32::consts::PI);

        return result;
    }

    pub fn add_light(&mut self, direction: &Vec3, color: &Vec3, delta_solid_angle: f32) {
        let c = color * delta_solid_angle;
        self.l00 += c * apply_sh3(0, direction);
        self.l1_1 += c * apply_sh3(1, direction);
        self.l10 += c * apply_sh3(2, direction);
        self.l11 += c * apply_sh3(3, direction);
        self.l2_2 += c * apply_sh3(4, direction);
        self.l2_1 += c * apply_sh3(5, direction);
        self.l20 += c * apply_sh3(6, direction);
        self.l21 += c * apply_sh3(7, direction);
        self.l22 += c * apply_sh3(8, direction);
    }

    pub fn scale_in_place(&mut self, scale: f32) {
        self.l00 *= scale;
        self.l1_1 *= scale;
        self.l10 *= scale;
        self.l11 *= scale;
        self.l2_2 *= scale;
        self.l2_1 *= scale;
        self.l20 *= scale;
        self.l21 *= scale;
        self.l22 *= scale;
    }

    /**
     * Integrates the reconstruction coefficients directly in to the SH preventing further
     * required operations at run time.
     *
     * This is simply done by scaling back the SH with Ylm constants parameter.
     * The trigonometric part being applied by the shader at run time.
     */
    pub fn pre_scale_for_rendering(&mut self) {
        self.l00 *= SH3YLM_BASIS_CONSTANTS[0];

        self.l1_1 *= SH3YLM_BASIS_CONSTANTS[1];
        self.l10 *= SH3YLM_BASIS_CONSTANTS[2];
        self.l11 *= SH3YLM_BASIS_CONSTANTS[3];

        self.l2_2 *= SH3YLM_BASIS_CONSTANTS[4];
        self.l2_1 *= SH3YLM_BASIS_CONSTANTS[5];
        self.l20 *= SH3YLM_BASIS_CONSTANTS[6];
        self.l21 *= SH3YLM_BASIS_CONSTANTS[7];
        self.l22 *= SH3YLM_BASIS_CONSTANTS[8];
    }

    /**
     * Convert from incident radiance (Li) to irradiance (E) by applying convolution with the cosine-weighted hemisphere.
     *
     * ```
     * E_lm = A_l * L_lm
     * ```
     *
     * In spherical harmonics this convolution amounts to scaling factors for each frequency band.
     * This corresponds to equation 5 in "An Efficient Representation for Irradiance Environment Maps", where
     * the scaling factors are given in equation 9.
     */
    pub fn convert_incident_radiance_to_irradiance(&mut self) {
        // Constant (Band 0)
        self.l00 *= SHCOS_KERNEL_CONVOLUTION[0];

        // Linear (Band 1)
        self.l1_1 *= SHCOS_KERNEL_CONVOLUTION[1];
        self.l10 *= SHCOS_KERNEL_CONVOLUTION[2];
        self.l11 *= SHCOS_KERNEL_CONVOLUTION[3];

        // Quadratic (Band 2)
        self.l2_2 *= SHCOS_KERNEL_CONVOLUTION[4];
        self.l2_1 *= SHCOS_KERNEL_CONVOLUTION[5];
        self.l20 *= SHCOS_KERNEL_CONVOLUTION[6];
        self.l21 *= SHCOS_KERNEL_CONVOLUTION[7];
        self.l22 *= SHCOS_KERNEL_CONVOLUTION[8];
    }

     /**
     * Convert from irradiance to outgoing radiance for Lambertian BDRF, suitable for efficient shader evaluation.
     *
     * ```
     * L = (1/pi) * E * rho
     * ```
     *
     * This is done by an additional scale by 1/pi, so is a fairly trivial operation but important conceptually.
     */
    pub fn convert_irradiance_to_lambertian_radiance(&mut self) {
        self.scale_in_place(1.0 / std::f32::consts::PI);

        // The resultant SH now represents outgoing radiance, so includes the Lambert 1/pi normalisation factor but without albedo (rho) applied
        // (The pixel shader must apply albedo after texture fetches, etc).
    }
}

/// Stealed from Babylon.js. Not used now, keep for references.
#[derive(Default, Debug, Clone, Copy)]
pub struct SphericalPolynomial {
    _harmonics: Option<SphericalHarmonics>,
    /**
     * The x coefficients of the spherical polynomial
     */
    pub x: Vec3,

    /**
     * The y coefficients of the spherical polynomial
     */
    pub y: Vec3,

    /**
     * The z coefficients of the spherical polynomial
     */
    pub z: Vec3,

    /**
     * The xx coefficients of the spherical polynomial
     */
    pub xx: Vec3,

    /**
     * The yy coefficients of the spherical polynomial
     */
    pub yy: Vec3,

    /**
     * The zz coefficients of the spherical polynomial
     */
    pub zz: Vec3,

    /**
     * The xy coefficients of the spherical polynomial
     */
    pub xy: Vec3,

    /**
     * The yz coefficients of the spherical polynomial
     */
    pub yz: Vec3,

    /**
     * The zx coefficients of the spherical polynomial
     */
    pub zx: Vec3,
}

impl SphericalPolynomial {
    /**
     * Gets the spherical polynomial from harmonics
     * @param harmonics the spherical harmonics
     * @returns the spherical polynomial
     */
    pub fn from_harmonics(harmonics: SphericalHarmonics) -> SphericalPolynomial {
        let mut result = SphericalPolynomial::default();
        result.update_from_harmonics(harmonics);
        result
    }

    pub fn scale_in_place(&mut self, scale: f32) {
        self.x *= scale;
        self.y *= scale;
        self.z *= scale;
        self.xx *= scale;
        self.yy *= scale;
        self.zz *= scale;
        self.xy *= scale;
        self.yz *= scale;
        self.zx *= scale;
    }

    /**
     * Updates the spherical polynomial from harmonics
     * @param harmonics the spherical harmonics
     * @returns the spherical polynomial
     */
    pub fn update_from_harmonics(&mut self, harmonics: SphericalHarmonics) {
        self._harmonics = Some(harmonics);

        self.x = harmonics.l11;
        self.x *= -1.02333;
        self.y = harmonics.l1_1;
        self.y *= -1.02333;
        self.z = harmonics.l10;
        self.z *= 1.02333;

        self.xx = harmonics.l00;
        let tmp_0 = harmonics.l20 * 0.247708;
        let tmp_1 = harmonics.l22 * 0.429043;
        self.xx *= 0.886277;
        self.xx = self.xx - tmp_0 + tmp_1;

        self.yy = harmonics.l00;
        self.yy *= 0.886277;
        self.yy = self.yy - tmp_0 + tmp_1;

        self.zz = harmonics.l00;
        let tmp_0 = harmonics.l20 * 0.495417;
        self.zz = self.zz * 0.886277 + tmp_0;

        self.yz = harmonics.l2_1 * -0.858086;
        self.zx = harmonics.l21 * -0.858086;
        self.xy = harmonics.l2_2 * 0.858086;

        self.scale_in_place(1.0 / std::f32::consts::PI);
    }
}