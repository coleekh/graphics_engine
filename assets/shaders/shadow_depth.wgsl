//  shadow_depth.wgsl  –  Shadow map depth pass
//  Renders scene geometry from the light's point of view, writing depth only.
//  Build this as a fragment-less pipeline targeting a depth texture, e.g.
//  with `fragment: None`, `Depth32Float` and per-light depth bias.

// Bind Groups

struct ShadowUniform {
    transform : mat4x4f,
}

// group 0
@group(0) @binding(0) var<uniform> shadow : ShadowUniform;

// Vertex I/O

struct VertexIn {
    @location(0) position : vec3f,
    @location(1) normal   : vec3f,
    @location(2) uv       : vec2f,
    @location(3) tangent  : vec4f,
}

// Instance buffer: four attributes, one per row of the 4x4 model matrix.
struct InstanceIn {
    @location(4) transform0 : vec4f,
    @location(5) transform1 : vec4f,
    @location(6) transform2 : vec4f,
    @location(7) transform3 : vec4f,
}

// Vertex Shader

@vertex
fn vs_main(v: VertexIn, i: InstanceIn) -> @builtin(position) vec4f {
    let model = mat4x4(i.transform0, i.transform1, i.transform2, i.transform3);
    return shadow.transform * model * vec4f(v.position, 1.0);
}