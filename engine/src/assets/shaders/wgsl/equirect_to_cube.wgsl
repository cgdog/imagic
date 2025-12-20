//////////////// vertex shader ////////////////
struct VSInput {
    @location(0) position: vec3f,
}

struct VSOutput {
    @location(0) local_pos: vec3f,
    @builtin(position) position: vec4f,
}

struct Uniforms {
    vp: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> vp: mat4x4<f32>;

@vertex
fn vs_main(vs_in: VSInput) -> VSOutput {
    var result: VSOutput;
    result.local_pos = vs_in.position;
    result.position = vp * vec4f(vs_in.position, 1.0);
    return result;
}

////////////// fragment shader ////////////////

struct FSIn {
    @location(0) local_pos: vec3f,
};

@group(0) @binding(1)
var skybox_2d_texture: texture_2d<f32>;
@group(0) @binding(2)
var skybox_2d_sampler: sampler;

const ONE_OVER_PI = 1.0 / 3.14159265359;
const ONE_OVER_TWO_PI = 1.0 / 6.28318530718;

@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    // normalized direction (cartesian coordinates)
    let dir = normalize(fs_in.local_pos);
    // convert to spherical coordinates [-PI, PI] and [-PI/2, PI/2], then convert to uv [0.0, 1.0]
    let uv = vec2(atan2(dir.z, dir.x) * ONE_OVER_TWO_PI, asin(dir.y) * ONE_OVER_PI) + 0.5;
    let color = textureSample(skybox_2d_texture, skybox_2d_sampler, uv);
    let frag_color = vec4f(color.rgb, 1.0);
    return frag_color;
}