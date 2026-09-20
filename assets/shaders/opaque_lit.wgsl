// ═══════════════════════════════════════════════════════════════════════════
//  opaque_lit.wgsl  –  Physically-Based Rendering (Cook-Torrance GGX)
//  Supports: directional lights, point lights, spot lights
//  Model: metallic-roughness workflow
// ═══════════════════════════════════════════════════════════════════════════

// ─── Bind Groups ─────────────────────────────────────────────────────────────

struct CameraUniform {
    view_proj : mat4x4f,
    eye_pos   : vec3f,
    _pad      : f32,
}

// group 0
@group(0) @binding(0) var<uniform> camera : CameraUniform;
@group(0) @binding(1) var<uniform> lights : LightsBlock;

// group 1
@group(1) @binding(0) var sample          : sampler;
@group(1) @binding(1) var base_colour_tex : texture_2d<f32>;
// AO, Roughness, Metallic Map
@group(1) @binding(2) var orm_tex         : texture_2d<f32>;
@group(1) @binding(3) var normal_tex      : texture_2d<f32>;

// ─── Vertex I/O ──────────────────────────────────────────────────────────────

struct VertexIn {
    @location(0) position : vec3f,
    @location(1) normal   : vec3f,
    @location(2) uv       : vec2f,
    @location(3) tangent  : vec4f,
}

struct VertexOut {
    @builtin(position) clip_pos    : vec4f,
    @location(0)       world_pos   : vec3f,
    @location(1)       uv          : vec2f,
    @location(2)       world_nor   : vec3f,
    @location(3)       world_tan   : vec4f,
    @location(4)       base_colour_tint : vec4f,
    @location(6)       orm_factor       : vec3f,
    @location(7)       emissive         : vec3f,
}


struct InstanceIn {
	@location(4)  transform0       : vec4f,
	@location(5)  transform1       : vec4f,
	@location(6)  transform2       : vec4f,
	@location(7)  transform3       : vec4f,
	@location(8)  normal_matrix0   : vec3f,
	@location(9)  normal_matrix1   : vec3f,
	@location(10) normal_matrix2   : vec3f,
    @location(11) base_colour_tint : vec3f,
    @location(12) orm_factor       : vec3f,
    @location(13) emissive         : vec3f,
    @location(14) uv_scale         : vec2f,
}

// ─── Vertex Shader ───────────────────────────────────────────────────────────

@vertex
fn vs_main(v: VertexIn, i: InstanceIn) -> VertexOut {
    let model = mat4x4(i.transform0, i.transform1, i.transform2, i.transform3);
    let normal_matrix = mat3x3(i.normal_matrix0.xyz, i.normal_matrix1.xyz, i.normal_matrix2.xyz);

    let world_pos = model * vec4f(v.position, 1.0);

    // Transform normal & tangent using the normal matrix (ignores non-uniform scale)
    let world_nor = (normal_matrix * v.normal).xyz;
    let world_tan = normal_matrix * v.tangent.xyz;

    var out: VertexOut;
    out.clip_pos    = camera.view_proj * world_pos;
    out.world_pos   = world_pos.xyz;
    out.world_nor   = world_nor;
    out.uv          = v.uv * i.uv_scale;
    out.world_tan   = vec4(world_tan, v.tangent.w);
    out.base_colour_tint = vec4(i.base_colour_tint, 1.0);
    out.orm_factor  = i.orm_factor;
    out.emissive    = i.emissive;
    return out;
}

// ─── Fragment Shader ─────────────────────────────────────────────────────────
const DIRECTION_LIGHT: u32 = 0;
const POINT_LIGHT: u32 = 1;
const SPOT_LIGHT: u32 = 2;

struct LightUniform {
    position  : vec3f,
    kind      : u32,      // 0=directional, 1=point, 2=spot
    direction : vec3f,
    intensity : f32,
    color     : vec3f,
    range     : f32,
    inner_cos : f32,
    outer_cos : f32,
    _pad      : vec2f,
}

const LIGHTS_CAPACITY: u32 = 1;
struct LightsBlock {
    count  : u32,
    _pad0  : u32,
    _pad1  : u32,
    _pad2  : u32,
    lights : array<LightUniform, LIGHTS_CAPACITY>,
}

// ─── PBR Math ────────────────────────────────────────────────────────────────

const PI : f32 = 3.14159265358979;

// Normal Distribution Function – GGX / Trowbridge-Reitz
fn distribution_ggx(N: vec3f, H: vec3f, roughness: f32) -> f32 {
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

fn geometry_smith(N: vec3f, V: vec3f, L: vec3f, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    return geometry_schlick_ggx(NdotV, roughness) * geometry_schlick_ggx(NdotL, roughness);
}

fn pow4(x: f32) -> f32 {
    let x2 = x * x;
    return x2 * x2;
}

fn pow5(x: f32) -> f32 {
    return pow4(x) * x;
}

// Fresnel – Schlick approximation
fn fresnel_schlick(cos_theta: f32, F0: vec3f) -> vec3f {
    return F0 + (1.0 - F0) * pow5(clamp(1.0 - cos_theta, 0.0, 1.0));
}

// Cook-Torrance BRDF contribution for one light direction
fn brdf(
    N         : vec3f,
    V         : vec3f,
    L         : vec3f,
    base_col  : vec3f,
    metallic  : f32,
    roughness : f32,
) -> vec3f {
    let H = normalize(V + L);

    let F0 = mix(vec3f(0.04), base_col, metallic);

    let D = distribution_ggx(N, H, roughness);
    let G = geometry_smith(N, V, L, roughness);
    let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

    let NdotL = max(dot(N, L), 0.0);
    let NdotV = max(dot(N, V), 0.0);

    let specular_denom = 4.0 * NdotV * NdotL + 0.00001;
    let specular = (D * G * F) / specular_denom;

    // Energy conservation: kD = (1-F) * (1-metallic)
    let kD = (vec3f(1.0) - F) * (1.0 - metallic);
    let diffuse = kD * base_col / PI;

    return (diffuse + specular) * NdotL;
}

// Attenuation for point / spot lights
fn attenuation(distance: f32, range: f32) -> f32 {
    if range <= 0.0 { return 1.0; }
    let x = max(1.0 - pow4(distance / range), 0.0);
    return x * x / (distance * distance + 1.0);
}


@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
    let normal = normalize(in.world_nor);
    var tangent = normalize(in.world_tan.xyz);
    let bitangent = cross(normal, tangent) * in.world_tan.w;
    let TBN = mat3x3(tangent, bitangent, normal);
    let N           = normalize(TBN * textureSample(normal_tex, sample, in.uv).xyz);
    let V           = normalize(camera.eye_pos - in.world_pos);

    let base_colour = in.base_colour_tint.rgb * textureSample(base_colour_tex, sample, in.uv).rgb;
    let orm         = in.orm_factor * textureSample(orm_tex, sample, in.uv).rgb;
    let ao          = orm.r;
    let roughness   = clamp(orm.g, 0.05, 1.0);
    let metallic    = orm.b;
    let emissive    = in.emissive;

    // Ambient (IBL placeholder: simple hemisphere)
    let up_factor    = max(dot(N, vec3f(0.0, 1.0, 0.0)), 0.0);
    let sky_color    = vec3f(0.3, 0.5, 0.9) * up_factor;
    let ground_color = vec3f(0.15, 0.12, 0.1) * (1.0 - up_factor);
    let ambient      = ao * (sky_color + ground_color) * base_colour;

    var Lo = vec3f(0.0);

    for (var i = 0u; i < lights.count; i++) {
        let light = lights.lights[i];
        var L        : vec3f;
        var radiance : vec3f;

        if light.kind == DIRECTION_LIGHT {
            // ── Directional ────────────────────────────────────────────
            L         = normalize(-light.direction);
            radiance  = light.color * light.intensity;

        } else if light.kind == POINT_LIGHT {
            // ── Point ──────────────────────────────────────────────────
            let diff  = light.position - in.world_pos;
            L         = normalize(diff);
            let dist  = length(diff);
            let atten = attenuation(dist, light.range);
            radiance  = light.color * light.intensity * atten;

        } else {
            // ── Spot ───────────────────────────────────────────────────
            let diff  = light.position - in.world_pos;
            L         = normalize(diff);
            let dist  = length(diff);
            let theta = dot(L, normalize(-light.direction));
            let eps   = light.inner_cos - light.outer_cos;
            let spot  = clamp((theta - light.outer_cos) / eps, 0.0, 1.0);
            let atten = attenuation(dist, light.range) * spot;
            radiance  = light.color * light.intensity * atten;
        }

        Lo += brdf(N, V, L, base_colour, metallic, roughness) * radiance;
    }

    var color = ambient + Lo + emissive;

    return vec4f(color, 1.0);
}
