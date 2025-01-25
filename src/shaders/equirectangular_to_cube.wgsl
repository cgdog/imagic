//////////////// vertex shader ////////////////
struct VSInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) uv0: vec2f,
}

struct VSOutput {
    @location(0) local_pos: vec3f,
    // TODO: remove uv0
    @location(1) uv0: vec2f,
    @builtin(position) position: vec4f,
};

struct VP {
    view: mat4x4f,
    projection: mat4x4f,
}

@group(0) @binding(0)
var<uniform> model_matrix: mat4x4f;

@group(1) @binding(0)
var<uniform> vp_matrix: VP;

@vertex
fn vs_main(vs_in: VSInput) -> VSOutput {
    var result: VSOutput;
    result.uv0 = vs_in.uv0;
    result.local_pos = vs_in.position;
    result.position = vp_matrix.projection * vp_matrix.view * model_matrix * vec4f(vs_in.position, 1.0);
    return result;
}

////////////// fragment shader ////////////////

struct FSIn {
    @location(0) local_pos: vec3f,
    @location(1) uv0: vec2f,
};

@group(2) @binding(0)
var skybox_2d_texture: texture_2d<f32>;
@group(2) @binding(1)
var skybox_2d_sampler: sampler;

const invAtan = vec2f(0.1591, 0.3183);

fn SampleSphericalMap(v: vec3f) -> vec2f
{
    var uv = vec2(atan2(v.z, v.x), asin(v.y));
    uv *= invAtan;
    uv += 0.5;
    return uv;
}

@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    let uv = SampleSphericalMap(normalize(fs_in.local_pos)); // make sure to normalize localPos
    let color = textureSample(skybox_2d_texture, skybox_2d_sampler, uv);
    let frag_color = vec4f(color.rgb, 1.0);
    return frag_color;
}