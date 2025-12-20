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

@group(1) @binding(0)
var<uniform> _model_matrix: mat4x4<f32>;

// @group(0) @binding(1)
// var<uniform> vp_matrix: VP;

@vertex
fn vs_main(vs_in: VSInput) -> VSOutput {
    var result: VSOutput;
    result.uv0 = vs_in.uv0;
    let world_pos = _model_matrix * vec4f(vs_in.position, 1.0);
    // let world_pos = vec4f(vs_in.position, 1.0);
    // result.world_pos = world_pos.xyz;
    // result.position = vp_matrix.projection * vp_matrix.view * world_pos;
    result.position = world_pos;
    return result;
}

////////////// fragment shader ////////////////

struct FSIn {
    @location(0) world_pos: vec3f,
    @location(1) uv0: vec2f,
};

@group(0) @binding(1)
var<uniform> _albedo_color: vec4f;

@group(0) @binding(2)
var _albedo_map: texture_2d<f32>;
@group(0) @binding(3)
var _albedo_map_sampler: sampler;
@group(0) @binding(4)
var<uniform> lod: f32;


@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    let frag_color = textureSampleLevel(_albedo_map, _albedo_map_sampler, fs_in.uv0, lod) * _albedo_color;
    return frag_color;
}