//////////////// vertex shader ////////////////
struct VSInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) uv0: vec2f,
}

struct VSOutput {
    @location(0) world_pos: vec3f,
    @location(1) world_normal: vec3f,
    @location(2) uv0: vec2f,
    @builtin(position) position: vec4f,
}

@group(0) @binding(0)
var<uniform> _model_matrix: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> _vp_matrix: mat4x4<f32>;

@vertex
fn vs_main(
    vs_in: VSInput
) -> VSOutput {
    var result: VSOutput;
    result.uv0 = vs_in.uv0;
    result.world_normal = normalize((_model_matrix * vec4f(vs_in.normal, 0.0)).xyz);
    let world_pos = _model_matrix * vec4f(vs_in.position, 1.0);
    result.world_pos = world_pos.xyz;
    result.position = _vp_matrix * world_pos;
    return result;
}

//////////////// fragment shader ////////////////
struct LightData {
    /// flags.x is light type, 0: directional, 1: point, 2: spot, 3: area.
    flags: vec4<u32>,
    // color.w is range or max distance for spot or point light.
    color: vec4f,
    // For spot light: direction.w is inner cone cosin.
    direction: vec4f,
    // For spot light: position.w is outter cone cosin.
    position: vec4f,
}

struct LightsInfo {
    // x: lights count
    light_count: vec4<u32>,
    lights_info: array<LightData>,
}

@group(1) @binding(1)
var<uniform> _camera_position: vec4f;

@group(2) @binding(0)
var<uniform> _material_features: vec4<u32>;
@group(2) @binding(1)
var<uniform> _albedo_color: vec4f;
@group(2) @binding(2)
var<uniform> _metallic_roughness_ao: vec4f;
@group(2) @binding(3)
var<uniform> _emissive_color: vec4f;
@group(2) @binding(4)
var _albedo_map_sampler: sampler;
@group(2) @binding(5)
var _albedo_map: texture_2d<f32>;
@group(2) @binding(6)
var _normal_map: texture_2d<f32>;
@group(2) @binding(7)
var _metallic_roughness_map: texture_2d<f32>;
@group(2) @binding(8)
var _ao_map: texture_2d<f32>;
@group(2) @binding(9)
var _emissive_map: texture_2d<f32>;

struct SH {
    sh: array<vec4f, 9>,
}

@group(3) @binding(0)
var _reflection_cube_sampler: sampler;
@group(3) @binding(1)
var<uniform> _sh: SH;

@group(3) @binding(2)
var _prefiltered_reflection_map: texture_cube<f32>;
@group(3) @binding(3)
var _brdf_lut: texture_2d<f32>;
@group(3) @binding(4)
var<uniform> _global_features: vec4<u32>;

@group(3) @binding(5)
var<storage, read> _lighting_infos: LightsInfo;

const PI = 3.1415926;

struct SurfaceProps {
    world_pos: vec3f,
    world_normal: vec3f,
    reflection_dir: vec3f,
    uv0: vec2f,
    f0: vec3f,
    metallic: f32,
    roughness: f32,
    albedo: vec3f,
    surface_ao: f32,
}

struct CameraProps {
    view_dir: vec3f,
}

struct LightingProps {
    light_dir: vec3f,
    radiance: vec3f,
}

struct FSIn {
    @location(0) world_pos: vec3f,
    @location(1) world_normal: vec3f,
    @location(2) uv0: vec2f,
}

// Per-material features
const FEATURE_FLAG_ALBEDO_MAP: u32 = 1u << 0u;
const FEATURE_FLAG_NORMAL_MAP: u32 = 1u << 1u;
const FEATURE_FLAG_METALLIC_ROUGHNESS_MAP: u32 = 1u << 2u;
const FEATURE_FLAG_AO_MAP: u32 = 1u << 3u;
const FEATURE_FLAG_EMISSIVE_MAP: u32 = 1u << 4u;

// Global features
const FEATURE_FLAG_IBL: u32 = 1u << 0u;

fn is_albedo_map_enabled() -> bool {
    return (_material_features.x & FEATURE_FLAG_ALBEDO_MAP) != 0u;
}

fn is_normal_map_enabled() -> bool {
    return (_material_features.x & FEATURE_FLAG_NORMAL_MAP) != 0u;
}

fn is_metallic_roughness_map_enabled() -> bool {
    return (_material_features.x & FEATURE_FLAG_METALLIC_ROUGHNESS_MAP) != 0u;
}

fn is_ao_map_enabled() -> bool {
    return (_material_features.x & FEATURE_FLAG_AO_MAP) != 0u;
}

fn is_emissive_map_enabled() -> bool {
    return (_material_features.x & FEATURE_FLAG_EMISSIVE_MAP) != 0u;
}

fn is_ibl_enabled() -> bool {
    return (_global_features.x & FEATURE_FLAG_IBL) != 0u;
}


fn get_normal_from_map(v_world_normal: vec3f, world_pos: vec3f, uv: vec2f) -> vec3f {
    let N   = normalize(v_world_normal);
    if !is_normal_map_enabled() {
        return N;
    }
    let tangent_normal = textureSample(_normal_map, _albedo_map_sampler, uv).xyz * 2.0 - 1.0;

    // https://www.w3.org/TR/WGSL/#dpdx-builtin
    let Q1  = dpdx(world_pos);
    let Q2  = dpdy(world_pos);
    let st1 = dpdx(uv);
    let st2 = dpdy(uv);
    
    let T  = normalize(Q1*st2.y - Q2*st1.y);
    let B  = -normalize(cross(N, T));
    let TBN = mat3x3<f32>(T, B, N);

    return normalize(TBN * tangent_normal);
}

fn distribution_ggx(normal: vec3f, half_vec: vec3f, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let n_dot_h = max(dot(normal, half_vec), 0.0);
    let n_dot_h_2 = n_dot_h * n_dot_h;
    let nom = a2;
    var denom = n_dot_h_2 * (a2 - 1.0) + 1.0;
    denom = PI * denom * denom;
    return nom / denom;
}

fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;

    let nom = n_dot_v;
    let denom = n_dot_v * (1.0 - k) + k;

    return nom / denom;
}

fn geometry_smith(normal: vec3f, view_dir: vec3f, light_dir: vec3f, roughness: f32) -> f32 {
    let n_dot_v = max(dot(normal, view_dir), 0.0);
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let ggx2 = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx1 = geometry_schlick_ggx(n_dot_l, roughness);

    return ggx1 * ggx2;
}

fn fresnel_schlick(cos_theta: f32, f0: vec3f) -> vec3f {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn fresnel_schlick_roughness(cos_theta: f32, F0: vec3f, roughness: f32) -> vec3f {
    return F0 + (max(vec3f(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn brdf(lighting_props: LightingProps, surface_props: SurfaceProps, camera_props: CameraProps) -> vec3f {
    let half_dir = normalize(lighting_props.light_dir + camera_props.view_dir);
    // Cook-Torrance BRDF
    let NDF = distribution_ggx(surface_props.world_normal, half_dir, surface_props.roughness);
    let G   = geometry_smith(surface_props.world_normal, camera_props.view_dir, lighting_props.light_dir, surface_props.roughness);
    let F   = fresnel_schlick(clamp(dot(half_dir, camera_props.view_dir), 0.0, 1.0), surface_props.f0);

    let numerator    = NDF * G * F;
    let denominator = 4.0 * max(dot(surface_props.world_normal, camera_props.view_dir), 0.0) * max(dot(surface_props.world_normal, lighting_props.light_dir), 0.0) + 0.0001;
    let specular = numerator / denominator;

    // kS is equal to Fresnel
    let kS = F;
    // for energy conservation, the diffuse and specular light can't
    // be above 1.0 (unless the surface emits light); to preserve this
    // relationship the diffuse component (kD) should equal 1.0 - kS.
    var kD = vec3f(1.0) - kS;
    // multiply kD by the inverse metalness such that only non-metals
    // have diffuse lighting, or a linear blend if partly metal (pure metals
    // have no diffuse light).
    kD = kD * (1.0 - surface_props.metallic);

    // scale light by NdotL
    let NdotL = max(dot(surface_props.world_normal, lighting_props.light_dir), 0.0);

    // add to outgoing radiance Lo
    return (kD * surface_props.albedo / PI + specular) * lighting_props.radiance * NdotL;
}

fn compute_environment_irradiance(normal: vec3f) -> vec3f {
    return _sh.sh[0].rgb
    + _sh.sh[1].rgb * (normal.y)
    + _sh.sh[2].rgb * (normal.z)
    + _sh.sh[3].rgb * (normal.x)
    + _sh.sh[4].rgb * (normal.y * normal.x)
    + _sh.sh[5].rgb * (normal.y * normal.z)
    + _sh.sh[6].rgb * ((3.0 * normal.z * normal.z)-1.0)
    + _sh.sh[7].rgb * (normal.z * normal.x)
    + _sh.sh[8].rgb * (normal.x * normal.x-(normal.y * normal.y));
}

// compute ambient lighting
fn ambient_lighting(surface_props: SurfaceProps, camera_props: CameraProps) -> vec3f {
    // let ambient = vec3f(0.03) * albedo * surface_ao;
    let F = fresnel_schlick_roughness(max(dot(surface_props.world_normal, camera_props.view_dir), 0.0), surface_props.f0, surface_props.roughness);
    let kS = F;
    var kD = vec3f(1.0) - kS;
    kD *= 1.0 - surface_props.metallic;
    // let irradiance = textureSample(_irradiance_cube_map, _reflection_cube_sampler, surface_props.world_normal).rgb;
    let irradiance = compute_environment_irradiance(surface_props.world_normal);
    let diffuse    = irradiance * surface_props.albedo;
    // sample both the pre-filter map and the BRDF lut and combine them together as per the Split-Sum approximation to get the IBL specular part.
    let MAX_REFLECTION_LOD: f32 = 4.0;
    let prefiltered_color = textureSampleLevel(_prefiltered_reflection_map, _reflection_cube_sampler, surface_props.reflection_dir, surface_props.roughness * MAX_REFLECTION_LOD).rgb;
    // let prefiltered_color = textureSampleLevel(_prefiltered_reflection_map, _reflection_cube_sampler, surface_props.reflection_dir, 1.5).rgb;
    let brdf  = textureSample(_brdf_lut, _albedo_map_sampler, vec2(clamp(dot(surface_props.world_normal, camera_props.view_dir), 0.0, 1.0), surface_props.roughness)).rg;
    let specular = prefiltered_color * (F * brdf.x + brdf.y);

    let ambient = (kD * diffuse + specular) * surface_props.surface_ao;
    return ambient;
}

fn lighting(surface_props: SurfaceProps, camera_props: CameraProps, surface_emissive: vec3f) -> vec3f {
    var lo = vec3f(0.0);
    for (var i = 0u; i < _lighting_infos.light_count.x; i = i + 1u) {
        let cur_light_data = _lighting_infos.lights_info[i];
        if cur_light_data.flags.x == 0u {
            // directional light
            let light_dir = cur_light_data.direction.xyz;
            let lighting_props = LightingProps(light_dir, cur_light_data.color.rgb);
            lo += brdf(lighting_props, surface_props, camera_props);
        } else if cur_light_data.flags.x == 1u {
            // point light
            // color is in linear space
            let to_light = cur_light_data.position.xyz - surface_props.world_pos;
            let distance = length(to_light);
            let max_distance = cur_light_data.color.a;
            if distance > max_distance {
                continue;
            }
            let light_dir = normalize(to_light);
            let fade = saturate(1.0 - distance / max_distance);
            let attenuation = fade * fade / max(distance * distance, 0.0001);
            let radiance = cur_light_data.color.rgb * attenuation;
            let lighting_props = LightingProps(light_dir, radiance);
            lo += brdf(lighting_props, surface_props, camera_props);
        } else if cur_light_data.flags.x == 2u {
            // spot light
            let to_light = cur_light_data.position.xyz - surface_props.world_pos;
            let distance = length(to_light);
            let max_distance = cur_light_data.color.a;
            if distance > max_distance {
                continue;
            }
            // from fragment to spot light.
            let light_dir = normalize(to_light);
            // spot light direction.
            let spotlight_dir = normalize(cur_light_data.direction.xyz);
            let spotlight_cos = dot(-light_dir, spotlight_dir);
            let spotlight_cos_outer = cur_light_data.position.w;
            let spotlight_cos_inner = cur_light_data.direction.w;
            let spotlight_effect = smoothstep(spotlight_cos_outer, spotlight_cos_inner, spotlight_cos);
            let fade = saturate(1.0 - distance / max_distance);
            let spotlight_attenuation = spotlight_effect * fade;
            let attenuation = spotlight_attenuation * fade * fade / max(distance * distance, 0.0001);
            let radiance = cur_light_data.color.rgb * attenuation;
            let lighting_props = LightingProps(light_dir, radiance);
            lo += brdf(lighting_props, surface_props, camera_props);
        } else if cur_light_data.flags.x == 3u {
            // area light
            // TODO: support area light.
        }
    }

    var color = lo;
    if is_ibl_enabled() {
        let ambient = ambient_lighting(surface_props, camera_props);
        color += ambient;
    }
    color += surface_emissive;
    return color;
}

@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {

    //////////////////////////
    let world_normal = get_normal_from_map(fs_in.world_normal, fs_in.world_pos, fs_in.uv0);
    let view_dir = normalize(_camera_position.xyz - fs_in.world_pos);
    let reflection_dir = reflect(-view_dir, world_normal);

    let camera_props = CameraProps(
        view_dir,
    );
    let metallic = _metallic_roughness_ao.x;
    let roughness = _metallic_roughness_ao.y;
    let ao = _metallic_roughness_ao.z;
    var surface_metallic = metallic;
    var surface_roughness = roughness;
    if is_metallic_roughness_map_enabled() {
        surface_metallic *= textureSample(_metallic_roughness_map, _albedo_map_sampler, fs_in.uv0).b;
        surface_roughness *= textureSample(_metallic_roughness_map, _albedo_map_sampler, fs_in.uv0).g;
    }

    var surface_ao = ao;
    if is_ao_map_enabled() {
        surface_ao *= textureSample(_ao_map, _albedo_map_sampler, fs_in.uv0).r;
    }

    var surface_albedo = _albedo_color.rgb;
    if is_albedo_map_enabled() {
        // Note: albedo texture has format of Rgba8UnormSrgb, which will convert sRGB color to linear space automatically.
        let albedo_texl = textureSample(_albedo_map, _albedo_map_sampler, fs_in.uv0);
        surface_albedo *= albedo_texl.rgb;
    }

    var surface_emissive = _emissive_color.rgb;
    if is_emissive_map_enabled() {
        let emissive_texl = textureSample(_emissive_map, _albedo_map_sampler, fs_in.uv0);
        surface_emissive *= emissive_texl.rgb;
    }

    var f0 = vec3f(0.04);
    f0 = mix(f0, surface_albedo, surface_metallic);

    let surface_props = SurfaceProps(
        fs_in.world_pos,
        world_normal,
        reflection_dir,
        fs_in.uv0,
        f0,
        surface_metallic,
        surface_roughness,
        surface_albedo,
        surface_ao,
    );

    var color = lighting(surface_props, camera_props, surface_emissive);

    // HDR tonemapping
    color = color / (color + vec3f(1.0));
    // the gamma correction below is not necessary, because the swapchain format is Bgra8UnormSrgb, 
    // which will covert linear space color to sRGB.
    // gamma correction
    // color = pow(color, vec3f(1.0/2.2));

    let frag_color = vec4f(color, 1.0);
    return frag_color;
}