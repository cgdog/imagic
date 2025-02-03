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
    result.position = vp_matrix.projection * vp_matrix.view * vec4f(vs_in.position, 1.0);
    return result;
}

////////////// fragment shader ////////////////

struct FSIn {
    @location(0) uv: vec3f,
};

@group(2) @binding(0)
var skybox_cube_texture: texture_cube<f32>;
@group(2) @binding(1)
var skybox_cube_sampler: sampler;


@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    var sample_dir = fs_in.uv;
    sample_dir.y = -sample_dir.y;
    let frag_color = textureSample(skybox_cube_texture, skybox_cube_sampler, sample_dir);
    // let frag_color = textureSampleLevel(skybox_cube_texture, skybox_cube_sampler, fs_in.uv, 4.0);
    // let frag_color = vec4f(1.0, 0.0, 1.0, 1.0);
    return frag_color;
}