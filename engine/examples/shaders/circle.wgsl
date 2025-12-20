//////////////// vertex shader ////////////////
struct VSInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) uv0: vec2f,
}

struct VSOutput {
    @location(0) uv0: vec2f,
    @builtin(position) position: vec4f,
};

// _model_matrix is builtin uniform used to transform object from local space to world space.
@group(0) @binding(0)
var<uniform> _model_matrix: mat4x4<f32>;

struct ColorInfo {
    input_color: vec4f,
    final_filter: vec4f,
}

@group(1) @binding(0)
var<uniform> color: ColorInfo;

/// _time is builtin uniform.
// x: time since started, y: delta time, z: scaled delta time, w: sin(time)
@group(2) @binding(1)
var<uniform> _time: vec4f;

@vertex
fn vs_main(vs_in: VSInput) -> VSOutput {
    var result: VSOutput;
    result.uv0 = vs_in.uv0;
    let world_pos = _model_matrix * vec4f(vs_in.position, 1.0);
    result.position = world_pos;
    return result;
}

////////////// fragment shader ////////////////
struct FSIn {
    @location(0) uv0: vec2f,
};

@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    let uv = fract(fs_in.uv0 * 5.0);
    let radius = 0.5;
    var frag_color = vec4f(fs_in.uv0, 0.0, 1.0);
    let center = vec2(0.5, 0.5);
    let distance = length(uv - center);
    if distance <= radius {
        frag_color = mix(color.input_color, frag_color, distance);
    }

    let raidus2 = 0.5 * abs(_time.w);
    let distance2 = length(fs_in.uv0 - center);

    if (distance2 <= raidus2) {
        frag_color = mix(color.input_color, frag_color, distance2);
    }

    frag_color *= color.final_filter;

    return frag_color;
}