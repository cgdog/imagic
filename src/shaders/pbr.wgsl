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

// struct MVP {
//     model: mat4x4f,
//     view: mat4x4f,
//     projection: mat4x4f,
// }

struct VP {
    view: mat4x4f,
    projection: mat4x4f,
}

@group(0) @binding(0)
var<uniform> model_matrix: mat4x4f;

@group(1) @binding(0)
var<uniform> vp_matrix: VP;

@vertex
fn vs_main(
    vs_in: VSInput
) -> VSOutput {
    var result: VSOutput;
    result.uv0 = vs_in.uv0;
    result.world_normal = (model_matrix * vec4f(vs_in.normal, 1.0)).xyz;
    let world_pos = model_matrix * vec4f(vs_in.position, 1.0);
    result.world_pos = world_pos.xyz;
    result.position = vp_matrix.projection * vp_matrix.view * world_pos;
    return result;
}

//////////////// fragment shader ////////////////
struct FragmentUniforms {
    albedo: vec4f,
    metallic_roughness_ao: vec4f,
    // w: 0, perspective; <0, orthographic
    // camera_pos: vec4f,
}

struct DirLightInfo {
    dir: vec4f,
    color: vec4f,
}

struct PointLightInfo {
    position: vec4f,
    color: vec4f,
}

struct SpotLightInfo {
    position: vec4f,
    color: vec4f,
    // x: inner angle, y: outter angle, z: angle decay, w: distance decay
    angel_decay: vec4f,
}

struct LightsInfo {
    // x,y,z: dir light, point light, spotlight
    light_count: vec4<u32>,
    lights_info: array<vec4f>,
}

@group(1) @binding(1)
var<uniform> camera_pos: vec4f;

@group(2) @binding(0)
var<uniform> fragment_uniforms: FragmentUniforms;
@group(2) @binding(1)
var s_sampler_0: sampler;
@group(2) @binding(2)
var t_albedo: texture_2d<f32>;
@group(2) @binding(3)
var t_normal: texture_2d<f32>;
@group(2) @binding(4)
var t_metallic: texture_2d<f32>;
@group(2) @binding(5)
var t_roughness: texture_2d<f32>;
@group(2) @binding(6)
var t_ao: texture_2d<f32>;

@group(3) @binding(0)
var<storage, read> lighting_infos: LightsInfo;

const PI = 3.1415926;

struct SurfaceProps {
    world_pos: vec3f,
    world_normal: vec3f,
    uv0: vec2f,
    f0: vec3f,
    metallic: f32,
    roughness: f32,
    albedo: vec3f,
}

struct CameraProps {
    view_dir: vec3f,
}

struct FSIn {
    @location(0) world_pos: vec3f,
    @location(1) world_normal: vec3f,
    @location(2) uv0: vec2f,
}

fn get_normal_from_map(v_world_normal: vec3f, world_pos: vec3f, uv: vec2f) -> vec3f {
    let tangent_normal = textureSample(t_normal, s_sampler_0, uv).xyz * 2.0 - 1.0;

    // https://www.w3.org/TR/WGSL/#dpdx-builtin
    let Q1  = dpdx(world_pos);
    let Q2  = dpdy(world_pos);
    let st1 = dpdx(uv);
    let st2 = dpdy(uv);

    let N   = normalize(v_world_normal);
    let T  = normalize(Q1*st2.y - Q2*st1.y);
    let B  = -normalize(cross(N, T));
    let TBN = mat3x3f(T, B, N);

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

fn lighting_point(light_pos: vec3f, light_color: vec3f, surface_props: SurfaceProps, camera_props: CameraProps) -> vec3f {
    let light_dir = normalize(light_pos - surface_props.world_pos);
    let half_dir = normalize(light_dir + camera_props.view_dir);

    let distance = length(light_pos - surface_props.world_pos);
    let attenuation = 1.0 / (distance * distance);
    let radiance = light_color * attenuation;

    // Cook-Torrance BRDF
    let NDF = distribution_ggx(surface_props.world_normal, half_dir, surface_props.roughness);   
    let G   = geometry_smith(surface_props.world_normal, camera_props.view_dir, light_dir, surface_props.roughness);      
    let F   = fresnel_schlick(clamp(dot(half_dir, camera_props.view_dir), 0.0, 1.0), surface_props.f0);
        
    let numerator    = NDF * G * F; 
    let denominator = 4.0 * max(dot(surface_props.world_normal, camera_props.view_dir), 0.0) * max(dot(surface_props.world_normal, light_dir), 0.0) + 0.0001;
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
    let NdotL = max(dot(surface_props.world_normal, light_dir), 0.0);        

    // add to outgoing radiance Lo
    return (kD * surface_props.albedo / PI + specular) * radiance * NdotL;
}

@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    
    //////////////////////////
    // let world_normal = normalize(fs_in.world_normal);
    let world_normal = get_normal_from_map(fs_in.world_normal, fs_in.world_pos, fs_in.uv0);
    // let camera_pos = fragment_uniforms.camera_pos.xyz;
    let view_dir = normalize(camera_pos.xyz - fs_in.world_pos);

    let camera_props = CameraProps(
        view_dir,
    );
    let metallic = fragment_uniforms.metallic_roughness_ao.x;
    let roughness = fragment_uniforms.metallic_roughness_ao.y;
    let ao = fragment_uniforms.metallic_roughness_ao.z;
    let surface_metallic  = metallic * textureSample(t_metallic, s_sampler_0, fs_in.uv0).r;
    let surface_roughness = roughness * textureSample(t_roughness, s_sampler_0, fs_in.uv0).r;
    let surface_ao        = ao * textureSample(t_ao, s_sampler_0, fs_in.uv0).r;

    let albedo_texl = textureSample(t_albedo, s_sampler_0, fs_in.uv0);
    let surface_albedo = albedo_texl.rgb * fragment_uniforms.albedo.rgb;
    var f0 = vec3f(0.04);
    f0 = mix(f0, surface_albedo, surface_metallic);

    let surface_props = SurfaceProps(
        fs_in.world_pos,
        world_normal,
        fs_in.uv0,
        f0,
        surface_metallic,
        surface_roughness,
        surface_albedo,
    );

    var lo = vec3f(0.0);
    var light_info_index: u32 = 0;
    // compute dir lighting
    // compute point lighting
    let point_light_count = lighting_infos.light_count.y * 2;
    for (; light_info_index < point_light_count; light_info_index = light_info_index + 2) {
        let cur_point_light_pos = lighting_infos.lights_info[light_info_index];
        let cur_point_light_color = lighting_infos.lights_info[light_info_index+1];
        lo += lighting_point(cur_point_light_pos.xyz, cur_point_light_color.rgb, surface_props, camera_props);
    }
    // compute spot lighting

    // let ambient = vec3f(0.03) * albedo * surface_ao;
    let ambient = vec3f(0.0);
    var color = ambient + lo;

    // HDR tonemapping
    color = color / (color + vec3f(1.0));
    // gamma correct
    color = pow(color, vec3f(1.0/2.2));

    let frag_color = vec4f(color, 1.0);
    return frag_color;
}