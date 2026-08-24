// ═══════════════════════════════════════════════════════════════════════════
//  pbr.wgsl  –  Physically-Based Rendering (Cook-Torrance GGX)
//  Supports: directional lights, point lights, spot lights
//  Model: metallic-roughness workflow
// ═══════════════════════════════════════════════════════════════════════════

// ─── Bind Groups ─────────────────────────────────────────────────────────────

struct CameraUniform {
    view_proj : mat4x4<f32>,
    eye_pos   : vec3<f32>,
    _pad      : f32,
}

struct LightUniform {
    position  : vec3<f32>,
    kind      : u32,      // 0=directional, 1=point, 2=spot
    direction : vec3<f32>,
    intensity : f32,
    color     : vec3<f32>,
    range     : f32,
    inner_cos : f32,
    outer_cos : f32,
    _pad      : vec2<f32>,
}

struct LightsBlock {
    count  : u32,
    _pad0  : u32,
    _pad1  : u32,
    _pad2  : u32,
    lights : array<LightUniform, 8>,
}

struct ObjectUniform {
    model          : mat4x4<f32>,
    normal_matrix  : mat4x4<f32>,
    base_color     : vec3<f32>,
    metallic       : f32,
    roughness      : f32,
    _pad0          : f32,
    _pad1          : f32,
    _pad2          : f32,
    emissive       : vec3<f32>,
    _pad3          : f32,
}

@group(0) @binding(0) var<uniform> camera  : CameraUniform;
@group(0) @binding(1) var<uniform> lights  : LightsBlock;
@group(1) @binding(0) var<uniform> object  : ObjectUniform;

// ─── Vertex I/O ──────────────────────────────────────────────────────────────

struct VertexIn {
    @location(0) position : vec3<f32>,
    @location(1) normal   : vec3<f32>,
    @location(2) uv       : vec2<f32>,
    @location(3) tangent  : vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_pos  : vec4<f32>,
    @location(0)       world_pos : vec3<f32>,
    @location(1)       world_nor : vec3<f32>,
    @location(2)       uv        : vec2<f32>,
    @location(3)       tangent   : vec3<f32>,
    @location(4)       bitangent : vec3<f32>,
}

// ─── Vertex Shader ───────────────────────────────────────────────────────────

@vertex
fn vs_main(v: VertexIn) -> VertexOut {
    let world_pos = object.model * vec4<f32>(v.position, 1.0);

    // Transform normal & tangent using the normal matrix (ignores non-uniform scale)
    let world_nor = normalize((object.normal_matrix * vec4<f32>(v.normal, 0.0)).xyz);
    let world_tan = normalize((object.model         * vec4<f32>(v.tangent.xyz, 0.0)).xyz);
    let world_bit = cross(world_nor, world_tan) * v.tangent.w;

    var out: VertexOut;
    out.clip_pos  = camera.view_proj * world_pos;
    out.world_pos = world_pos.xyz;
    out.world_nor = world_nor;
    out.uv        = v.uv;
    out.tangent   = world_tan;
    out.bitangent = world_bit;
    return out;
}

// ─── PBR Math ────────────────────────────────────────────────────────────────

const PI : f32 = 3.14159265358979;

// Normal Distribution Function – GGX / Trowbridge-Reitz
fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let denom = (NdotH * NdotH) * (a2 - 1.0) + 1.0;
    return a2 / max(PI * denom * denom, 0.00001);
}

// Geometry – Smith's Schlick-GGX
fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return NdotV / (NdotV * (1.0 - k) + k);
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    return geometry_schlick_ggx(NdotV, roughness) * geometry_schlick_ggx(NdotL, roughness);
}

fn pow5(x: f32) -> f32 {
    let x2 = x * x;
    return x2 * x2 * x;
}

// Fresnel – Schlick approximation
fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow5(clamp(1.0 - cos_theta, 0.0, 1.0));
}

// Cook-Torrance BRDF contribution for one light direction
fn brdf(
    N:         vec3<f32>,
    V:         vec3<f32>,
    L:         vec3<f32>,
    base_col:  vec3<f32>,
    metallic:  f32,
    roughness: f32,
) -> vec3<f32> {
    let H = normalize(V + L);

    let F0 = mix(vec3<f32>(0.04), base_col, metallic);

    let D = distribution_ggx(N, H, roughness);
    let G = geometry_smith(N, V, L, roughness);
    let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

    let NdotL = max(dot(N, L), 0.0);
    let NdotV = max(dot(N, V), 0.0);

    let specular_denom = 4.0 * NdotV * NdotL + 0.00001;
    let specular = (D * G * F) / specular_denom;

    // Energy conservation: kD = (1-F) * (1-metallic)
    let kD = (vec3<f32>(1.0) - F) * (1.0 - metallic);
    let diffuse = kD * base_col / PI;

    return (diffuse + specular) * NdotL;
}

// Attenuation for point / spot lights
fn distance_attenuation(dist: f32, range: f32) -> f32 {
    if range <= 0.0 { return 1.0; }
    let x = max(1.0 - pow(dist / range, 4.0), 0.0);
    return x * x / (dist * dist + 1.0);
}

fn eval_directional_light(
    light: LightUniform,
    N:     vec3<f32>,
    V:     vec3<f32>,
    base_col:  vec3<f32>,
    metallic:  f32,
    roughness: f32,
) -> vec3<f32> {
    let L = normalize(-light.direction);
    let radiance = light.color * light.intensity;
    return brdf(N, V, L, base_col, metallic, roughness) * radiance;
}

fn eval_point_light(
    light: LightUniform,
    N:     vec3<f32>,
    V:     vec3<f32>,
    world_pos: vec3<f32>,
    base_col:  vec3<f32>,
    metallic:  f32,
    roughness: f32,
) -> vec3<f32> {
    let enlonged_L = light.position - world_pos;
    let distance = length(enlonged_L);
    let L = enlonged_L / distance;
    let attenuation = distance_attenuation(distance, light.range);
    let radiance = light.color * light.intensity * attenuation;
    return brdf(N, V, L, base_col, metallic, roughness) * radiance;
}

// ─── Fragment Shader ─────────────────────────────────────────────────────────

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let N          = normalize(in.world_nor);
    let V          = normalize(camera.eye_pos - in.world_pos);
    let base_color = object.base_color;
    let metallic   = object.metallic;
    let roughness  = clamp(object.roughness, 0.05, 1.0);

    // Ambient (IBL placeholder: simple hemisphere)
    let ambient_intensity = 0.04;
    let up_factor  = max(dot(N, vec3<f32>(0.0, 1.0, 0.0)), 0.0);
    let sky_color  = vec3<f32>(0.3, 0.5, 0.9) * up_factor;
    let grnd_color = vec3<f32>(0.15, 0.12, 0.1) * (1.0 - up_factor);
    let ambient    = (sky_color + grnd_color) * ambient_intensity * base_color;

    var Lo = vec3<f32>(0.0);

    for (var i = 0u; i < lights.count; i++) {
        let light = lights.lights[i];
        var L        : vec3<f32>;
        var radiance : vec3<f32>;

        if light.kind == 0u {
            // ── Directional ────────────────────────────────────────────
            L         = normalize(-light.direction);
            radiance  = light.color * light.intensity;

        } else if light.kind == 1u {
            // ── Point ──────────────────────────────────────────────────
            let diff  = light.position - in.world_pos;
            L         = normalize(diff);
            let dist  = length(diff);
            let atten = distance_attenuation(dist, light.range);
            radiance  = light.color * light.intensity * atten;

        } else {
            // ── Spot ───────────────────────────────────────────────────
            let diff  = light.position - in.world_pos;
            L         = normalize(diff);
            let dist  = length(diff);
            let theta = dot(L, normalize(-light.direction));
            let eps   = light.inner_cos - light.outer_cos;
            let spot  = clamp((theta - light.outer_cos) / eps, 0.0, 1.0);
            let atten = distance_attenuation(dist, light.range) * spot;
            radiance  = light.color * light.intensity * atten;
        }

        Lo += brdf(N, V, L, base_color, metallic, roughness) * radiance;
    }

    var color = ambient + Lo + object.emissive;

    // Reinhard tone-mapping
    color = color / (color + vec3<f32>(1.0));

    // Gamma correction (linear → sRGB approximation)
    color = pow(color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, 1.0);
}
