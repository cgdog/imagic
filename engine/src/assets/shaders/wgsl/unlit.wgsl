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

// Model-View-Projection matrices as a single uniform block.
// This is an alternative to using separate uniforms for model, view, and projection matrices.
// struct MVP {
//     model: mat4x4<f32>,
//     view: mat4x4<f32>,
//     projection: mat4x4<f32>,
// }

// @group(0) @binding(0)
// var<uniform> _m_v_p_matrices: MVP;

@group(0) @binding(0)
var<uniform> _model_matrix: mat4x4<f32>;
@group(1) @binding(1)
var<uniform> _vp_matrix: mat4x4<f32>;


@vertex
fn vs_main(vs_in: VSInput) -> VSOutput {
    var result: VSOutput;
    result.uv0 = vs_in.uv0;
    let world_pos = _vp_matrix * _model_matrix * vec4f(vs_in.position, 1.0);
    // let world_pos = _m_v_p_matrices.projection * _m_v_p_matrices.view * _m_v_p_matrices.model * vec4f(vs_in.position, 1.0);
    result.position = world_pos;
    return result;
}

////////////// fragment shader ////////////////

struct FSIn {
    @location(0) world_pos: vec3f,
    @location(1) uv0: vec2f,
};

@group(2) @binding(3)
var<uniform> _albedo_color: vec4f;

@group(2) @binding(4)
var _albedo_map: texture_2d<f32>;
@group(2) @binding(5)
var _albedo_map_sampler: sampler;
@group(2) @binding(6)
var<uniform> _material_features: vec4<u32>;

const FEATURE_FLAG_ALBEDO_MAP: u32 = 1u;

fn is_albedo_map_enabled() -> bool {
    return (_material_features.x & FEATURE_FLAG_ALBEDO_MAP) != 0u;
}

@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    var frag_color = _albedo_color;
    if (is_albedo_map_enabled()) {
        frag_color *= textureSample(_albedo_map, _albedo_map_sampler, fs_in.uv0);
    }
    return frag_color;
}