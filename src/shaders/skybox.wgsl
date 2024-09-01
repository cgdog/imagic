//////////////// vertex shader ////////////////
struct VSInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) uv0: vec2f,
}

struct VSOutput {
    @location(0) world_pos: vec3f,
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
    // result.world_normal = (model_matrix * vec4f(vs_in.normal, 1.0)).xyz;
    let world_pos = model_matrix * vec4f(vs_in.position, 1.0);
    result.world_pos = world_pos.xyz;
    result.position = vp_matrix.projection * vp_matrix.view * world_pos;
    return result;
}

////////////// fragment shader ////////////////

struct FSIn {
    @location(0) world_pos: vec3f,
    @location(1) uv0: vec2f,
};

@group(2) @binding(0)
var skybox_2d_texture: texture_2d<f32>;
@group(2) @binding(1)
var skybox_2d_sampler: sampler;


@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    // let frag_color = vec4f(fs_in.world_pos, 1.0);
    let frag_color = textureSample(skybox_2d_texture, skybox_2d_sampler, fs_in.uv0);
    // let frag_color = vec4f(fs_in.uv0, 0, 1);

    return frag_color;
}