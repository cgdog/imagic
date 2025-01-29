//////////////// vertex shader ////////////////
struct VSInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) uv0: vec2f,
}

struct VSOutput {
    @location(0) local_pos: vec3f,
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
    result.local_pos = vs_in.position;
    result.position = vp_matrix.projection * vp_matrix.view * model_matrix * vec4f(vs_in.position, 1.0);
    return result;
}

////////////// fragment shader ////////////////

struct FSIn {
    @location(0) local_pos: vec3f,
};

@group(2) @binding(0)
var t_cube: texture_cube<f32>;
@group(2) @binding(1)
var s_sampler: sampler;
const PI: f32 = 3.14159265359;
const TWO_PI: f32 = 3.14159265359 * 2.0;
const HALF_PI: f32 = 3.14159265359 * 0.5;

@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    let normal = normalize(fs_in.local_pos);
    var irradiance = vec3f(0.0);

    var up    = vec3f(0.0, 1.0, 0.0);
    let right = normalize(cross(up, normal));
    up         = normalize(cross(normal, right));

    // Note: if phiSteps and thetaSteps were set to 251 and 63 respectively, the result may be black in some machine. 
    // I do not know the reason.
    // I try to set phiSteps and thetaSteps to be 200, 200 respectively, the result may be also balck.
    // It seems that the times the the double for loop execute should not greater than some value in WGSL.
    // Oh, shit! (120, 120) or even (110, 110) may be aslo black.
    // Finally, I decide to use (100, 100) here, it always works.
    // TODO: figure out the real reason in the future.

    // let sampleDelta: f32 = 0.025;
    var nrSamples: f32 = 0.0; 
    let phiSteps: i32 = 100;//i32(TWO_PI / sampleDelta);//251
    let thetaSteps: i32 = 100;//i32(HALF_PI / sampleDelta);//63
    let phiSampleDelta = TWO_PI / f32(phiSteps);
    let thetaSampleDelta = HALF_PI / f32(thetaSteps);

    for(var i: i32 = 0; i < phiSteps; i = i + 1) {
        let phi: f32 = f32(i) * phiSampleDelta;
        for(var j: i32 = 0; j < thetaSteps; j = j + 1) {
            let theta: f32 = f32(j) * thetaSampleDelta;
            // spherical to cartesian (in tangent space)
            let tangentSample = vec3f(sin(theta) * cos(phi),  sin(theta) * sin(phi), cos(theta));
            // tangent space to world
            var direction = tangentSample.x * right + tangentSample.y * up + tangentSample.z * normal; 
            direction = normalize(direction);
            irradiance += textureSample(t_cube, s_sampler, direction).rgb * cos(theta) * sin(theta);
            nrSamples += 1.0;
        }
    }
    irradiance = PI * irradiance * (1.0 / nrSamples);

    let frag_color = vec4f(irradiance, 1.0);
    return frag_color;
}