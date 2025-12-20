//////////////// vertex shader ////////////////
struct VSInput {
    @location(0) position: vec3f,
}

struct VSOutput {
    @location(0) uv: vec3f,
    @builtin(position) position: vec4f,
};

struct VP {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> _v_p_matrices: VP;

@vertex
fn vs_main(vs_in: VSInput) -> VSOutput {
    var result: VSOutput;
    result.uv = vs_in.position;
    let view = mat3x3<f32>(
        _v_p_matrices.view[0].xyz,
        _v_p_matrices.view[1].xyz,
        _v_p_matrices.view[2].xyz,
    );

    result.position = _v_p_matrices.projection * vec4f(view * vs_in.position, 1.0);
    result.position = result.position.xyww;
    return result;
}

////////////// fragment shader ////////////////

struct FSIn {
    @location(0) uv: vec3f,
};

@group(1) @binding(0)
var skybox_cube_texture: texture_cube<f32>;
@group(1) @binding(1)
var skybox_cube_sampler: sampler;
@group(1) @binding(2)
var<uniform> lod: f32;


@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    let frag_color = textureSampleLevel(skybox_cube_texture, skybox_cube_sampler, fs_in.uv, lod);
    return frag_color;
}