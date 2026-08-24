pub mod camera;

use glam::{Vec3, Quat, Mat4};

// ─── Handle types ───────────────────────────────────────────────────────────

/// Opaque handle referencing an uploaded GPU mesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeshHandle(pub usize);

// ─── Vertex ─────────────────────────────────────────────────────────────────

/// A single interleaved vertex for the PBR pipeline.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal:   [f32; 3],
    pub uv:       [f32; 2],
    pub tangent:  [f32; 4], // xyz = tangent, w = handedness
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

pub struct MeshBuilder;

impl MeshBuilder {
    // ── Cube ─────────────────────────────────────────────────────────────
    pub fn cube(size: f32) -> Mesh {
        let h = size * 0.5;

        // face: (normal, up, right) → 4 vertices + 6 indices
        let faces: [([f32;3], [f32;3], [f32;3]); 6] = [
            ([ 0.0,  0.0,  1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]), // front
            ([ 0.0,  0.0, -1.0], [0.0, 1.0, 0.0], [-1.0,0.0, 0.0]), // back
            ([-1.0,  0.0,  0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]), // left
            ([ 1.0,  0.0,  0.0], [0.0, 1.0, 0.0], [0.0, 0.0,-1.0]), // right
            ([ 0.0,  1.0,  0.0], [0.0, 0.0,-1.0], [1.0, 0.0, 0.0]), // top
            ([ 0.0, -1.0,  0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]), // bottom
        ];

        let uvs = [[0.0f32, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
        let mut vertices = Vec::with_capacity(24);
        let mut indices  = Vec::with_capacity(36);

        for (n, u, r) in &faces {
            let nv   = Vec3::from(*n);
            let uv   = Vec3::from(*u);
            let rv   = Vec3::from(*r);
            let base = vertices.len() as u32;

            let corners = [
                (-rv - uv) * h, // bl
                ( rv - uv) * h, // br
                ( rv + uv) * h, // tr
                (-rv + uv) * h, // tl
            ];

            for (i, pos) in corners.iter().enumerate() {
                let center = nv * h;
                vertices.push(Vertex {
                    position: (center + *pos).to_array(),
                    normal:   nv.to_array(),
                    uv:       uvs[i],
                    tangent:  [rv.x, rv.y, rv.z, 1.0],
                });
            }

            // Two triangles per face
            indices.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
        }

        Mesh { vertices, indices }
    }

    // ── UV Sphere ─────────────────────────────────────────────────────────
    pub fn sphere(radius: f32, sectors: u32, stacks: u32) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices  = Vec::new();

        for stack in 0..=stacks {
            let phi = std::f32::consts::PI * stack as f32 / stacks as f32; // 0 → π
            let y   = phi.cos();
            let r   = phi.sin();

            for sector in 0..=sectors {
                let theta = 2.0 * std::f32::consts::PI * sector as f32 / sectors as f32;
                let x = r * theta.cos();
                let z = r * theta.sin();

                let pos = Vec3::new(x, y, z);
                let tangent = Vec3::new(-theta.sin(), 0.0, theta.cos());

                vertices.push(Vertex {
                    position: (pos * radius).to_array(),
                    normal:   pos.to_array(),
                    uv:       [sector as f32 / sectors as f32, stack as f32 / stacks as f32],
                    tangent:  [tangent.x, tangent.y, tangent.z, 1.0],
                });
            }
        }

        let row = sectors + 1;
        for stack in 0..stacks {
            for sector in 0..sectors {
                let a = stack * row + sector;
                let b = a + row;
                if stack != 0        { indices.extend_from_slice(&[a, b, a+1]); }
                if stack != stacks-1 { indices.extend_from_slice(&[a+1, b, b+1]); }
            }
        }

        Mesh { vertices, indices }
    }

    // ── Torus ─────────────────────────────────────────────────────────────
    pub fn torus(major_r: f32, minor_r: f32, major_seg: u32, minor_seg: u32) -> Mesh {
        use std::f32::consts::TAU;
        let mut vertices = Vec::new();
        let mut indices  = Vec::new();

        for i in 0..=major_seg {
            let u     = TAU * i as f32 / major_seg as f32;
            let cos_u = u.cos();
            let sin_u = u.sin();
            let center = Vec3::new(major_r * cos_u, 0.0, major_r * sin_u);

            for j in 0..=minor_seg {
                let v     = TAU * j as f32 / minor_seg as f32;
                let cos_v = v.cos();
                let sin_v = v.sin();

                let pos = Vec3::new(
                    (major_r + minor_r * cos_v) * cos_u,
                     minor_r * sin_v,
                    (major_r + minor_r * cos_v) * sin_u,
                );
                let normal = (pos - center).normalize();
                let tangent = Vec3::new(-sin_u, 0.0, cos_u);

                vertices.push(Vertex {
                    position: pos.to_array(),
                    normal:   normal.to_array(),
                    uv:       [i as f32 / major_seg as f32, j as f32 / minor_seg as f32],
                    tangent:  [tangent.x, tangent.y, tangent.z, 1.0],
                });
            }
        }

        let row = minor_seg + 1;
        for i in 0..major_seg {
            for j in 0..minor_seg {
                let a = i * row + j;
                let b = a + row;
                indices.extend_from_slice(&[a, b, a+1, a+1, b, b+1]);
            }
        }

        Mesh { vertices, indices }
    }

    // ── Plane ─────────────────────────────────────────────────────────────
    pub fn plane(width: f32, depth: f32, w_segs: u32, d_segs: u32) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices  = Vec::new();

        for z in 0..=d_segs {
            for x in 0..=w_segs {
                let fx = x as f32 / w_segs as f32; // 0..1
                let fz = z as f32 / d_segs as f32;

                vertices.push(Vertex {
                    position: [(fx - 0.5) * width, 0.0, (fz - 0.5) * depth],
                    normal:   [0.0, 1.0, 0.0],
                    uv:       [fx, fz],
                    tangent:  [1.0, 0.0, 0.0, 1.0],
                });
            }
        }

        let row = w_segs + 1;
        for z in 0..d_segs {
            for x in 0..w_segs {
                let a = z * row + x;
                let b = a + row;
                indices.extend_from_slice(&[a, b, a+1, a+1, b, b+1]);
            }
        }

        Mesh { vertices, indices }
    }
}

// ─── Material ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Material {
    pub base_color: Vec3,
    pub metallic:   f32,
    pub roughness:  f32,
    pub emissive:   Vec3,
}

// ─── SceneObject ────────────────────────────────────────────────────────────

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
        * Mat4::from_scale(self.scale)
    }
} 

// ─── Camera ──────────────────────────────────────────────────────────────────

// pub struct Camera {
//     pub eye:     Vec3,
//     pub target:  Vec3,
//     pub up:      Vec3,
//     pub fovy:    f32,
//     pub aspect:  f32,
//     pub near:    f32,
//     pub far:     f32,
// }

// impl Camera {
//     pub fn new(
//         eye: Vec3, target: Vec3, up: Vec3,
//         fovy: f32, aspect: f32, near: f32, far: f32,
//     ) -> Self {
//         Self { eye, target, up, fovy, aspect, near, far }
//     }

//     pub fn view_matrix(&self) -> Mat4 {
//         Mat4::look_at_rh(self.eye, self.target, self.up)
//     }

//     pub fn projection_matrix(&self) -> Mat4 {
//         Mat4::perspective_rh(self.fovy, self.aspect, self.near, self.far)
//     }

//     pub fn view_proj(&self) -> Mat4 {
//         self.projection_matrix() * self.view_matrix()
//     }

//     pub fn update_aspect(&mut self, aspect: f32) {
//         self.aspect = aspect;
//     }
// }

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
                Vec3::new(0.0, 2.0, 5.0),
                16.0 / 9.0,
            ),
            objects: Vec::new(),
            lights:  Vec::new(),
            ambient: Vec3::new(0.05, 0.05, 0.08),
        }
    }
}
