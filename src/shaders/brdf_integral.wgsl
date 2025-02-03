//////////////// vertex shader ////////////////
struct VSInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) uv0: vec2f,
}

struct VSOutput {
    @location(0) uv: vec2f,
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
    result.uv = vs_in.uv0;

    // result.position = vp_matrix.projection * vp_matrix.view * vec4f(vs_in.position, 1.0);
    result.position = vec4f(vs_in.position, 1.0);
    return result;
}

////////////// fragment shader ////////////////

const PI: f32 = 3.14159265359;
// ----------------------------------------------------------------------------
// http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
// efficient VanDerCorpus calculation.
fn RadicalInverse_VdC(sample_index: u32) -> f32 
{
    var bits = sample_index;
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
	var H: vec3f;
	H.x = cos(phi) * sinTheta;
	H.y = sin(phi) * sinTheta;
	H.z = cosTheta;
	
	// from tangent-space H vector to world-space sample vector
	// let up: vec3f = abs(N.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
    let up: vec3f = select(vec3f(1.0, 0.0, 0.0), vec3f(0.0, 0.0, 1.0), abs(N.z) < 0.999);
	let tangent: vec3f = normalize(cross(up, N));
	let bitangent: vec3f = cross(N, tangent);
	
	let sampleVec: vec3f = tangent * H.x + bitangent * H.y + N * H.z;
	return normalize(sampleVec);
}
// ----------------------------------------------------------------------------
fn GeometrySchlickGGX(NdotV: f32, roughness: f32) -> f32
{
    // note that we use a different k for IBL
    let a = roughness;
    let k = (a * a) / 2.0;

    let nom   = NdotV;
    let denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}
// ----------------------------------------------------------------------------
fn GeometrySmith(N: vec3f, V: vec3f, L: vec3f, roughness: f32) -> f32
{
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = GeometrySchlickGGX(NdotV, roughness);
    let ggx1 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}
// ----------------------------------------------------------------------------
fn IntegrateBRDF(NdotV: f32, roughness: f32) -> vec2f
{
    var V: vec3f;
    V.x = sqrt(1.0 - NdotV*NdotV);
    V.y = 0.0;
    V.z = NdotV;

    var A = 0.0;
    var B = 0.0; 

    let N = vec3f(0.0, 0.0, 1.0);
    
    const SAMPLE_COUNT: u32 = 1024u;
    for(var i = 0u; i < SAMPLE_COUNT; i++)
    {
        // generates a sample vector that's biased towards the
        // preferred alignment direction (importance sampling).
        let Xi = Hammersley(i, SAMPLE_COUNT);
        let H = ImportanceSampleGGX(Xi, N, roughness);
        let L = normalize(2.0 * dot(V, H) * H - V);

        let NdotL = max(L.z, 0.0);
        let NdotH = max(H.z, 0.0);
        let VdotH = max(dot(V, H), 0.0);

        if(NdotL > 0.0)
        {
            let G = GeometrySmith(N, V, L, roughness);
            let G_Vis = (G * VdotH) / (NdotH * NdotV);
            let Fc = pow(1.0 - VdotH, 5.0);

            A += (1.0 - Fc) * G_Vis;
            B += Fc * G_Vis;
        }
    }
    A /= f32(SAMPLE_COUNT);
    B /= f32(SAMPLE_COUNT);
    return vec2f(A, B);
}

struct FSIn {
    @location(0) uv: vec2f,
};

@fragment
fn fs_main(fs_in: FSIn) -> @location(0) vec4f {
    // It seems that the result of "1.0 - fs_in.uv.y" is more similar to OpenGL's result.
    // I do not know the exact reason. Maybe it's caused by some "flip y"?
    // var integratedBRDF = IntegrateBRDF(fs_in.uv.x, fs_in.uv.y);
    var integratedBRDF = IntegrateBRDF(fs_in.uv.x, 1.0 - fs_in.uv.y);
    // Open the line below, you will get the same brdf lut as openGL.
    // integratedBRDF = pow(integratedBRDF, vec2f(2.2));
    return vec4f(integratedBRDF, 0.0, 1.0);
}