//////////////// vertex shader ////////////////
struct VSInput {
    @location(0) position: vec3f,
    // @location(1) normal: vec3f,
    @location(2) uv0: vec2f,
}

struct VSOutput {
    @location(0) uv0: vec2f,
    @builtin(position) position: vec4f,
};

@group(0) @binding(0)
var<uniform> _mvp_matrix: mat4x4<f32>;

@vertex
fn vs_main(vs_in: VSInput) -> VSOutput {
    var result: VSOutput;
    result.uv0 = vs_in.uv0;
    result.position = _mvp_matrix * vec4f(vs_in.position, 1.0);
    return result;
}

////////////// fragment shader ////////////////

struct FSIn {
    @location(0) uv0: vec2f,
};


@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    let frag_color = vec4f(fs_in.uv0, 0.0, 1.0);
    return frag_color;
}