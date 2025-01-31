@group(0) @binding(0)
var input_tex: texture_2d_array<f32>;
@group(0) @binding(1)
var output_tex: texture_storage_2d_array<rgba32float, write>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.z >= 6u) { return; }
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
    
    let x0y0 = textureLoad(input_tex, vec2<i32>(x0, y0), i32(id.z), 0);
    let x1y0 = textureLoad(input_tex, vec2<i32>(x0 + 1, y0), i32(id.z), 0);
    let x0y1 = textureLoad(input_tex, vec2<i32>(x0, y0 + 1), i32(id.z), 0);
    let x1y1 = textureLoad(input_tex, vec2<i32>(x0 + 1, y0 + 1), i32(id.z), 0);
    let horizontal0 = mix(x0y0, x1y0, delta_x);
    let horizontal1 = mix(x0y1, x1y1, delta_x);
    let final_color = mix(horizontal0, horizontal1, delta_y);
    
    textureStore(output_tex, vec2<i32>(id.xy), i32(id.z), final_color);
    // textureStore(output_tex, vec2<i32>(id.xy), i32(id.z), vec4f(1.0, 1.0, 0.0, 1.0));
}