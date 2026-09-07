pub mod camera;
pub mod mesh_builder;

use glam::{Mat3, Mat3A, Mat4, Quat, Vec3, vec2, vec3, vec4};

// ─── Handle types ───────────────────────────────────────────────────────────

/// Opaque handle referencing an uploaded GPU mesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeshHandle(pub usize);

// ─── Vertex ─────────────────────────────────────────────────────────────────

/// A single interleaved vertex for the PBR pipeline.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vec3,
    pub normal:   Vec3,
    pub uv:       Vec2,
    pub tangent:  Vec4, // xyz = tangent, w = handedness
}

impl Vertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // uv
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // tangent
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

// ─── CPU-side mesh ──────────────────────────────────────────────────────────

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices:  Vec<u32>,
}

// ─── MeshBuilder ─────────────────────────────────────────────────────────────

use glam::{Vec2, Vec4};

#[derive(Clone, Debug)]
pub struct Material {
    pub base_color: Vec3,
    pub metallic:   f32,
    pub roughness:  f32,
    pub emissive:   Vec3,
}

pub struct SceneObject {
    pub mesh:     MeshHandle,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale:    Vec3,
    pub material: Material,
}

impl SceneObject {
    pub fn model(&self) -> Mat4 {
        Mat4::from_translation(self.position) 
        * Mat4::from_quat(self.rotation)
        .mul_diagonal_scale(self.scale.extend(1.0))
    }

    pub fn normal_matrix(&self) -> Mat3A {
        Mat3A::from_quat(self.rotation).mul_diagonal_scale(self.scale.recip())
    }
} 

// ─── Light ──────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum LightKind {
    Directional,
    Point,
    Spot { inner_angle: f32, outer_angle: f32 },
}

#[derive(Clone, Debug)]
pub struct Light {
    pub kind:      LightKind,
    pub position:  Vec3,
    pub direction: Vec3,
    pub color:     Vec3,
    pub intensity: f32,
    pub range:     f32,
}

// ─── Scene ──────────────────────────────────────────────────────────────────

pub struct Scene {
    pub camera:  camera::Camera,
    pub objects: Vec<SceneObject>,
    pub lights:  Vec<Light>,
    pub ambient: Vec3,
} 

impl Scene {
    pub fn new() -> Self {
        Self {
            camera:  camera::Camera::new(
                vec3(0.0, 2.0, 5.0),
                16.0 / 9.0,
            ),
            objects: Vec::new(),
            lights:  Vec::new(),
            ambient: vec3(0.05, 0.05, 0.08),
        }
    }
}
