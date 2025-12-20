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

// @group(0) @binding(0)
// var<uniform> model_matrix: mat4x4f;

@group(0) @binding(0)
var<uniform> vp: mat4x4<f32>;;

@vertex
fn vs_main(vs_in: VSInput) -> VSOutput {
    var result: VSOutput;
    result.local_pos = vs_in.position;
    // result.position = vp.projection * vp.view * model_matrix * vec4f(vs_in.position, 1.0);
    result.position = vp * vec4f(vs_in.position, 1.0);
    return result;
}

////////////// fragment shader ////////////////

struct FSIn {
   @location(0) local_pos: vec3f,
};

// struct FragmentUnifoms {
//     roughness: f32,
// }

@group(0) @binding(1)
var input_cube_texture: texture_cube<f32>;
@group(0) @binding(2)
var cube_sampler: sampler;
@group(0) @binding(3)
var<uniform> roughness: f32;

const PI: f32 = 3.14159265359;

// ----------------------------------------------------------------------------
fn DistributionGGX(N: vec3f, H: vec3f, roughness: f32) -> f32
{
    let a = roughness*roughness;
    let a2 = a*a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH*NdotH;

    let nom   = a2;
    var denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}
// ----------------------------------------------------------------------------
// http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
// efficient VanDerCorpus calculation.
fn RadicalInverse_VdC(input_bits: u32) -> f32
{
    var bits = input_bits;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return f32(bits) * 2.3283064365386963e-10; // / 0x100000000
}
// ----------------------------------------------------------------------------
fn Hammersley(i: u32, N: u32) -> vec2f
{
	return vec2f(f32(i)/f32(N), RadicalInverse_VdC(i));
}
// ----------------------------------------------------------------------------
fn ImportanceSampleGGX(Xi: vec2f, N: vec3f, roughness: f32) -> vec3f
{
	let a = roughness*roughness;
	
	let phi = 2.0 * PI * Xi.x;
	let cosTheta = sqrt((1.0 - Xi.y) / (1.0 + (a*a - 1.0) * Xi.y));
	let sinTheta = sqrt(1.0 - cosTheta*cosTheta);
	
	// from spherical coordinates to cartesian coordinates - halfway vector
	var H = vec3(0.0, 0.0 ,0.0);
	H.x = cos(phi) * sinTheta;
	H.y = sin(phi) * sinTheta;
	H.z = cosTheta;
	
	// from tangent-space H vector to world-space sample vector
	let up        = select(vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), abs(N.z) < 0.999);
	let tangent   = normalize(cross(up, N));
	let bitangent = cross(N, tangent);
	
	let sampleVec = tangent * H.x + bitangent * H.y + N * H.z;
	return normalize(sampleVec);
}
// ----------------------------------------------------------------------------
// @fragment
// fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
//     return vec4f(1.0, 1.0, 0.0, 1.0);
// }
@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    let WorldPos = fs_in.local_pos;
    let N = normalize(WorldPos);
    
    // make the simplifying assumption that V equals R equals the normal 
    let R = N;
    let V = R;

    const SAMPLE_COUNT: u32 = 1024u;
    var prefilteredColor = vec3(0.0);
    var totalWeight: f32 = 0.0;

    // let roughness = fragment_uniforms.roughness;
    
    for(var i = 0u; i < SAMPLE_COUNT; i++)
    {
        // generates a sample vector that's biased towards the preferred alignment direction (importance sampling).
        let Xi = Hammersley(i, SAMPLE_COUNT);
        let H = ImportanceSampleGGX(Xi, N, roughness);
        let L  = normalize(2.0 * dot(V, H) * H - V);

        let NdotL = max(dot(N, L), 0.0);
        if(NdotL > 0.0)
        {
            // sample from the environment's mip level based on roughness/pdf
            let D   = DistributionGGX(N, H, roughness);
            let NdotH = max(dot(N, H), 0.0);
            let HdotV = max(dot(H, V), 0.0);
            let pdf = D * NdotH / (4.0 * HdotV) + 0.0001; 

            let resolution = 512.0; // resolution of source cubemap (per face)
            let saTexel  = 4.0 * PI / (6.0 * resolution * resolution);
            let saSample = 1.0 / (f32(SAMPLE_COUNT) * pdf + 0.0001);

            // let mipLevel = roughness == 0.0 ? 0.0 : 0.5 * log2(saSample / saTexel); 
            let mipLevel = select(0.5 * log2(saSample / saTexel), 0.0, roughness == 0.0);
            
            // Note: the line below is used to correct cube texture upside down problem.
            // I do not know the problem reason now.
            let sample_dir = vec3f(L.x, -L.y, L.z);
            prefilteredColor += textureSampleLevel(input_cube_texture, cube_sampler, sample_dir, mipLevel).rgb * NdotL;
            totalWeight      += NdotL;
        }
    }

    prefilteredColor = prefilteredColor / totalWeight;

    let frag_color = vec4f(prefilteredColor, 1.0);
    // let frag_color = vec4f(1.0, 1.0, 1.0, 1.0);
    return frag_color;
}