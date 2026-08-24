use bytemuck::Zeroable;
use glam::Mat4;
use crate::scene::{camera::Camera, SceneObject, Light, LightKind};

pub const MAX_LIGHTS: usize = 8;

// ─── Camera Uniform ───────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: glam::Mat4,
    eye_pos:   glam::Vec3,
    _pad:      f32,
}

impl CameraUniform {
    pub fn from_camera(cam: &Camera) -> Self {
        Self {
            view_proj: cam.projection() * cam.view(),
            eye_pos:   cam.position,
            _pad:      0.0,
        }
    }
}

// ─── Light Uniform ────────────────────────────────────────────────────────────

/// GPU representation of a single light (std140 layout).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position:   [f32; 3],
    kind:       u32,       // 0=directional, 1=point, 2=spot
    direction:  [f32; 3],
    intensity:  f32,
    color:      [f32; 3],
    range:      f32,
    inner_cos:  f32,       // cos(inner_angle) for spot
    outer_cos:  f32,       // cos(outer_angle) for spot
    _pad:       [f32; 2],
}

impl LightUniform {
    pub fn from_scene_lights(lights: &[Light]) -> ([LightUniform; MAX_LIGHTS], usize) {
        let mut arr = [LightUniform::zeroed(); MAX_LIGHTS];
        let count   = lights.len().min(MAX_LIGHTS);

        for (i, light) in lights.iter().take(MAX_LIGHTS).enumerate() {
            let (kind_id, inner_cos, outer_cos) = match &light.kind {
                LightKind::Directional          => (0u32, 0.0, 0.0),
                LightKind::Point                => (1u32, 0.0, 0.0),
                LightKind::Spot { inner_angle, outer_angle } => (
                    2u32,
                    inner_angle.cos(),
                    outer_angle.cos(),
                ),
            };

            arr[i] = LightUniform {
                position:  light.position.to_array(),
                kind:      kind_id,
                direction: light.direction.to_array(),
                intensity: light.intensity,
                color:     light.color.to_array(),
                range:     light.range,
                inner_cos,
                outer_cos,
                _pad:      [0.0; 2],
            };
        }

        (arr, count)
    }
}

// ─── Object Uniform ───────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ObjectUniform {
    model:          [[f32; 4]; 4],
    normal_matrix:  [[f32; 4]; 4], // transpose(inverse(model)) – use vec4 rows for padding
    base_color:     [f32; 3],
    metallic:       f32,
    roughness:      f32,
    _pad0:          [f32; 3],
    emissive:       [f32; 3],
    _pad1:          f32,
}

impl ObjectUniform {
    pub fn from_object(obj: &SceneObject) -> Self {
        let model = obj.model();

        // Normal matrix: transpose of the inverse of the upper-left 3×3
        let normal_mat3 = glam::Mat3::from_mat4(model).inverse().transpose();
        // Pad to mat4 for std140
        let n = glam::Mat4::from_mat3(normal_mat3);

        Self {
            model:         model.to_cols_array_2d(),
            normal_matrix: n.to_cols_array_2d(),
            base_color:    obj.material.base_color.to_array(),
            metallic:      obj.material.metallic,
            roughness:     obj.material.roughness,
            _pad0:         [0.0; 3],
            emissive:      obj.material.emissive.to_array(),
            _pad1:         0.0,
        }
    }
}
