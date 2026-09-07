use glam::{vec3, vec2, vec4, Vec3};

use crate::scene::{Mesh, Vertex};


pub const CUBE_VERTICES: [Vertex; 24] = [
    // +X
    Vertex {
        position: vec3( 0.5, -0.5, -0.5),
        normal:   Vec3::X,
        uv:       vec2(0.0, 1.0),
        tangent:  vec4(0.0, 0.0, 1.0, 1.0),
    },
    Vertex {
        position: vec3( 0.5, -0.5,  0.5),
        normal:   Vec3::X,
        uv:       vec2(1.0, 1.0),
        tangent:  vec4(0.0, 0.0, 1.0, 1.0),
    },
    Vertex {
        position: vec3( 0.5,  0.5,  0.5),
        normal:   Vec3::X,
        uv:       vec2(1.0, 0.0),
        tangent:  vec4(0.0, 0.0, 1.0, 1.0),
    },
    Vertex {
        position: vec3( 0.5,  0.5, -0.5),
        normal:   Vec3::X,
        uv:       vec2(0.0, 0.0),
        tangent:  vec4(0.0, 0.0, 1.0, 1.0),
    },

    // -X
    Vertex {
        position: vec3(-0.5, -0.5,  0.5),
        normal:   Vec3::NEG_X,
        uv:       vec2(0.0, 1.0),
        tangent:  vec4(0.0, 0.0, -1.0, 1.0),
    },
    Vertex {
        position: vec3(-0.5, -0.5, -0.5),
        normal:   Vec3::NEG_X,
        uv:       vec2(1.0, 1.0),
        tangent:  vec4(0.0, 0.0, -1.0, 1.0),
    },
    Vertex {
        position: vec3(-0.5,  0.5, -0.5),
        normal:   Vec3::NEG_X,
        uv:       vec2(1.0, 0.0),
        tangent:  vec4(0.0, 0.0, -1.0, 1.0),
    },
    Vertex {
        position: vec3(-0.5,  0.5,  0.5),
        normal:   Vec3::NEG_X,
        uv:       vec2(0.0, 0.0),
        tangent:  vec4(0.0, 0.0, -1.0, 1.0),
    },

    // +Y
    Vertex {
        position: vec3(-0.5,  0.5, -0.5),
        normal:   Vec3::Y,
        uv:       vec2(0.0, 1.0),
        tangent:  vec4(1.0, 0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3( 0.5,  0.5, -0.5),
        normal:   Vec3::Y,
        uv:       vec2(1.0, 1.0),
        tangent:  vec4(1.0, 0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3( 0.5,  0.5,  0.5),
        normal:   Vec3::Y,
        uv:       vec2(1.0, 0.0),
        tangent:  vec4(1.0, 0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3(-0.5,  0.5,  0.5),
        normal:   Vec3::Y,
        uv:       vec2(0.0, 0.0),
        tangent:  vec4(1.0, 0.0, 0.0, 1.0),
    },

    // -Y
    Vertex {
        position: vec3(-0.5, -0.5,  0.5),
        normal:   Vec3::NEG_Y,
        uv:       vec2(0.0, 1.0),
        tangent:  vec4(1.0, 0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3( 0.5, -0.5,  0.5),
        normal:   Vec3::NEG_Y,
        uv:       vec2(1.0, 1.0),
        tangent:  vec4(1.0, 0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3( 0.5, -0.5, -0.5),
        normal:   Vec3::NEG_Y,
        uv:       vec2(1.0, 0.0),
        tangent:  vec4(1.0, 0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3(-0.5, -0.5, -0.5),
        normal:   Vec3::NEG_Y,
        uv:       vec2(0.0, 0.0),
        tangent:  vec4(1.0, 0.0, 0.0, 1.0),
    },

    // +Z
    Vertex {
        position: vec3(-0.5, -0.5,  0.5),
        normal:   Vec3::Z,
        uv:       vec2(0.0, 1.0),
        tangent:  vec4(1.0, 0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3(-0.5,  0.5,  0.5),
        normal:   Vec3::Z,
        uv:       vec2(0.0, 0.0),
        tangent:  vec4(1.0, 0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3( 0.5,  0.5,  0.5),
        normal:   Vec3::Z,
        uv:       vec2(1.0, 0.0),
        tangent:  vec4(1.0, 0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3( 0.5, -0.5,  0.5),
        normal:   Vec3::Z,
        uv:       vec2(1.0, 1.0),
        tangent:  vec4(1.0, 0.0, 0.0, 1.0),
    },

    // -Z
    Vertex {
        position: vec3( 0.5, -0.5, -0.5),
        normal:   Vec3::NEG_Z,
        uv:       vec2(0.0, 1.0),
        tangent:  vec4(-1.0, 0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3( 0.5,  0.5, -0.5),
        normal:   Vec3::NEG_Z,
        uv:       vec2(0.0, 0.0),
        tangent:  vec4(-1.0, 0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3(-0.5,  0.5, -0.5),
        normal:   Vec3::NEG_Z,
        uv:       vec2(1.0, 0.0),
        tangent:  vec4(-1.0, 0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3(-0.5, -0.5, -0.5),
        normal:   Vec3::NEG_Z,
        uv:       vec2(1.0, 1.0),
        tangent:  vec4(-1.0, 0.0, 0.0, 1.0),
    },
];

pub const CUBE_INDICES: [u32; 36] = [
    // +X
    0, 2, 1,
    0, 3, 2,
    // -X
    4, 6, 5,
    4, 7, 6,
    // +Y
    8, 10, 9,
    8, 11, 10,
    // -Y
    12, 14, 13,
    12, 15, 14,
    // +Z
    16, 18, 17,
    16, 19, 18,
    // -Z
    20, 22, 21,
    20, 23, 22,
];

pub struct MeshBuilder;

impl MeshBuilder {
    /// Centered 1x1x1 cube
    pub fn unit_cube() -> Mesh {
        Mesh { vertices: CUBE_VERTICES.to_vec(), indices: CUBE_INDICES.to_vec() }
    }

    /// UV Sphere
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

                let pos = vec3(x, y, z);
                let tangent = vec3(-theta.sin(), 0.0, theta.cos());

                vertices.push(Vertex {
                    position: pos * radius,
                    normal:   pos,
                    uv:       vec2(sector as f32 / sectors as f32, stack as f32 / stacks as f32),
                    tangent:  tangent.extend(1.0),
                });
            }
        }

        let row = sectors + 1;
        for stack in 0..stacks {
            for sector in 0..sectors {
                let a = stack * row + sector;
                let b = a + row;
                if stack != 0        { indices.extend_from_slice(&[a+1, b, a]); }
                if stack != stacks-1 { indices.extend_from_slice(&[b+1, b, a+1]); }
            }
        }

        Mesh { vertices, indices }
    }

    pub fn torus(major_r: f32, minor_r: f32, major_seg: u32, minor_seg: u32) -> Mesh {
        use std::f32::consts::TAU;
        let mut vertices = Vec::new();
        let mut indices  = Vec::new();

        for i in 0..=major_seg {
            let u     = TAU * i as f32 / major_seg as f32;
            let cos_u = u.cos();
            let sin_u = u.sin();
            let center = vec3(major_r * cos_u, 0.0, major_r * sin_u);

            for j in 0..=minor_seg {
                let v     = TAU * j as f32 / minor_seg as f32;
                let cos_v = v.cos();
                let sin_v = v.sin();

                let position = vec3(
                    (major_r + minor_r * cos_v) * cos_u,
                     minor_r * sin_v,
                    (major_r + minor_r * cos_v) * sin_u,
                );
                let normal = (position - center).normalize();
                let tangent = vec3(-sin_u, 0.0, cos_u);

                
                vertices.push(Vertex {
                    position,
                    normal,
                    uv:       vec2(i as f32 / major_seg as f32, j as f32 / minor_seg as f32),
                    tangent:  tangent.extend(1.0),
                });
            }
        }

        let row = minor_seg + 1;
        for i in 0..major_seg {
            for j in 0..minor_seg {
                let a = i * row + j;
                let b = a + row;
                indices.extend_from_slice(&[b+1, b, a+1, a+1, b, a]);
            }
        }

        Mesh { vertices, indices }
    }

    pub fn plane(width: f32, depth: f32, w_segs: u32, d_segs: u32) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices  = Vec::new();

        for z in 0..=d_segs {
            for x in 0..=w_segs {
                let fx = x as f32 / w_segs as f32; // 0..1
                let fz = z as f32 / d_segs as f32;

                vertices.push(Vertex {
                    position: vec3((fx - 0.5) * width, 0.0, (fz - 0.5) * depth),
                    normal:   Vec3::Y,
                    uv:       vec2(fx, fz),
                    tangent:  vec4(1.0, 0.0, 0.0, 1.0),
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
