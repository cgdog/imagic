//! Custom skybox material which support to sample specific lod of the given CubeTexture.

//////////////// vertex shader ////////////////
struct VSInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) uv0: vec2f,
}

struct VSOutput {
    @location(0) uv: vec3f,
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
    result.uv = vs_in.position;
    // result.position = vp_matrix.projection * vp_matrix.view * vec4f(vs_in.position, 1.0);
    let view = mat3x3f(
        vp_matrix.view[0].xyz,
        vp_matrix.view[1].xyz,
        vp_matrix.view[2].xyz,
    );

    result.position = vp_matrix.projection * vec4f(view * vs_in.position, 1.0);
    result.position = result.position.xyww;
    return result;
}

////////////// fragment shader ////////////////

struct CustomUniforms {
    lod: f32,
}

struct FSIn {
    @location(0) uv: vec3f,
};

@group(2) @binding(0)
var skybox_cube_texture: texture_cube<f32>;
@group(2) @binding(1)
var skybox_cube_sampler: sampler;
@group(2) @binding(2)
var<uniform> custom_uniforms: CustomUniforms;


@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    // Note: if custom_uniforms.lod does not exist, it will sample lod 0.
    // if custom_uniforms.lod exists, but without data, sample result will be black.
    let frag_color = textureSampleLevel(skybox_cube_texture, skybox_cube_sampler, fs_in.uv, custom_uniforms.lod);
    return frag_color;
}