struct VSOutput {
    @builtin(position) pos: vec4f,
    @location(0) uv: vec2f,
}

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VSOutput {
    let x = f32(i32((vid & 2) << 1) - 1);
    let y = f32(i32((vid & 1) << 2) - 1);
    let u = f32(vid & 2);
    let v = f32(1 - i32((vid & 1) << 1));
    return VSOutput(vec4f(x, y, 0, 1), vec2f(u, v));
}

struct FSIn {
    @location(0) uv: vec2f,
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    return textureSample(t_diffuse, s_diffuse, fs_in.uv);
}
