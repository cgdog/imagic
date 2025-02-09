@group(0) @binding(0)
var input_tex: texture_2d<f32>;
@group(0) @binding(1)
var output_tex: texture_storage_2d<rgba8unorm, write>;

fn gamma_to_linear(gamma_color: vec4f) -> vec4f {
    let linear_color = vec4f(pow(gamma_color.rgb, vec3f(2.2)), gamma_color.a);
    return linear_color;
}

fn linear_to_gamma(linear_color: vec4f) -> vec4f {
    let gamma_color = vec4f(pow(linear_color.rgb, vec3f(1.0 / 2.2)), linear_color.a);
    return gamma_color;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let cur_size = textureDimensions(output_tex);
    if (id.x >= cur_size.x || id.y >= cur_size.y) { return; }

    let normalized_coord_x = f32(id.x) / f32(cur_size.x);
    let normalized_coord_y = f32(id.y) / f32(cur_size.y);

    let prev_size = textureDimensions(input_tex);
    let pre_coord_x = normalized_coord_x * f32(prev_size.x);
    let pre_coord_y = normalized_coord_y * f32(prev_size.y);
    let x0 = i32(floor(pre_coord_x));
    let y0 = i32(floor(pre_coord_y));
    let delta_x = pre_coord_x - f32(x0);
    let delta_y = pre_coord_y - f32(y0);
    
    var x0y0 = textureLoad(input_tex, vec2<i32>(x0, y0), 0);
    x0y0 = gamma_to_linear(x0y0);
    var x1y0 = textureLoad(input_tex, vec2<i32>(x0 + 1, y0), 0);
    x1y0 = gamma_to_linear(x1y0);
    var x0y1 = textureLoad(input_tex, vec2<i32>(x0, y0 + 1), 0);
    x0y1 = gamma_to_linear(x0y1);
    var x1y1 = textureLoad(input_tex, vec2<i32>(x0 + 1, y0 + 1), 0);
    x1y1 = gamma_to_linear(x1y1);
    let horizontal0 = mix(x0y0, x1y0, delta_x);
    let horizontal1 = mix(x0y1, x1y1, delta_x);
    var final_color = mix(horizontal0, horizontal1, delta_y);
    final_color = linear_to_gamma(final_color);
    textureStore(output_tex, vec2<i32>(id.xy), final_color);
    // textureStore(output_tex, vec2<i32>(id.xy), i32(id.z), vec4f(1.0, 1.0, 0.0, 1.0));
}