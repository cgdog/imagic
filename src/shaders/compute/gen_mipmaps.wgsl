/// Generate mipmaps.
/// Capture the scene rendered by babylonjs https://playground.babylonjs.com/#NDQJBS,
/// we can change the fragment shader and show the specific lod.
/// It seems that openGL's or webgl's glGenerateMipmap() generates mipmaps similar something between 'main_bilinear()' and 'main_gaussian' below.

@group(0) @binding(0)
var input_tex: texture_2d_array<f32>;
@group(0) @binding(1)
var output_tex: texture_storage_2d_array<rgba32float, write>;

// Bilinear filter.
@compute @workgroup_size(8, 8, 1)
fn main_bilinear(@builtin(global_invocation_id) id: vec3<u32>) {
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

// Gaussian filter(4x4)
@compute @workgroup_size(8, 8, 1)
fn main_gaussian(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.z >= 6u) { return; }

    let output_size = textureDimensions(output_tex).xy;
    if (id.x >= output_size.x || id.y >= output_size.y) { return; }

    // last mip coord
    let prev_size = textureDimensions(input_tex).xy;
    let prev_coord = vec2<f32>(id.xy) * 2.0 + 0.5;

    // gassian kernel（4x4）
    let weights = array<f32, 16>(
        0.05, 0.1,  0.1,  0.05,
        0.1,  0.2,  0.2,  0.1,
        0.1,  0.2,  0.2,  0.1,
        0.05, 0.1,  0.1,  0.05
    );

    var sum = vec4<f32>(0.0);
    var total_weight = 0.0;

    // sample 4x4 region
    for (var i = 0; i < 4; i++) {
        for (var j = 0; j < 4; j++) {
            let offset = vec2<i32>(i, j) - vec2<i32>(2, 2);
            let sample_coord = vec2<i32>(prev_coord) + offset;
            
            let clamped_coord = clamp(
                sample_coord,
                vec2<i32>(0, 0),
                vec2<i32>(prev_size) - vec2<i32>(1, 1)
            );

            let color = textureLoad(input_tex, clamped_coord, i32(id.z), 0);
            let weight = weights[i * 4 + j];
            sum += color * weight;
            total_weight += weight;
        }
    }

    let result = sum / total_weight;
    textureStore(output_tex, vec2<i32>(id.xy), i32(id.z), result);
}

