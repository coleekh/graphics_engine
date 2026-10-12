use bytemuck::Zeroable;
use glam::{Mat3, Mat3A, Mat4, Vec2, Vec3, vec2, vec3};
use crate::scene::{camera::Camera, Aabb, SceneObject, Light, LightKind};

// Camera Uniform

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

// Light Uniform

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
    pub fn from_scene_lights(lights: &[Light]) -> ([LightUniform; crate::Engine::MAX_LIGHTS], usize) {
        let mut arr = [LightUniform::zeroed(); crate::Engine::MAX_LIGHTS];
        let count   = lights.len().min(crate::Engine::MAX_LIGHTS);

        for (i, light) in lights.iter().take(crate::Engine::MAX_LIGHTS).enumerate() {
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

// Object Uniform



#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    model:            Mat4,
    normal_matrix:    Mat3,
    base_colour_tint: Vec3,
    orm_factor:       Vec3,
    emissive:         Vec3,
    uv_scale:             Vec2,
}

impl InstanceData {
    pub fn from_object(obj: &SceneObject) -> Self {
        let model = obj.model();
        let normal_matrix = obj.normal_matrix();

        Self {
            model,
            normal_matrix,
            base_colour_tint: obj.material.base_colour,
            orm_factor:       vec3(0.04, obj.material.roughness, obj.material.metallic),
            emissive:         obj.material.emissive,
            uv_scale:         vec2(10.0, 10.0),
        }
    }

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
            4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4, // model matrix 
            8 => Float32x3, 9 => Float32x3, 10 => Float32x3, // normal matrix
            11 => Float32x3, // base_colour_tint
            12 => Float32x3, // orm_factor
            13 => Float32x3, // emissive
            14 => Float32x2, // _pad
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode:    wgpu::VertexStepMode::Instance,
            attributes:   &ATTRIBUTES,
        }
    }

    /// The same instance buffer, described for the depth-only shadow pass: it
    /// reads nothing but the four rows of the model matrix, so the buffer
    /// uploaded for the lit pass can be reused verbatim.
    pub fn shadow_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
            4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4, // model matrix
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode:    wgpu::VertexStepMode::Instance,
            attributes:   &ATTRIBUTES,
        }
    }
}

// Shadow Uniform

/// A fitted light view-projection plus the biases that suit its volume.
struct ShadowFit {
    transform:  Mat4,
    depth_bias: f32,
    slope_bias: f32,
}

/// GPU state for the single shadow-casting directional light.
///
/// The byte layout mirrors `ShadowUniform` in `assets/shaders/opaque_lit.wgsl`;
/// the explicit padding keeps the f32 fields where the shader expects them.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowUniform {
    /// Light view-projection. This is the same matrix the depth pass renders
    /// with, which is what makes the stored depths comparable.
    pub transform:   Mat4,
    /// Index into the light array of the casting light, or [`ShadowUniform::NO_CASTER`].
    pub light_index: u32,
    _pad0:           u32,
    /// Constant bias, removes acne on surfaces facing the light. Computed from
    /// the fitted volume, not fixed, so it survives retuning the fit.
    pub depth_bias:  f32,
    /// Additional bias proportional to the surface's slope to the light.
    pub slope_bias:  f32,
    /// `1.0 / shadow map resolution`, spreads the PCF taps by one texel.
    pub texel_size:  Vec2,
    _pad1:           [f32; 2],
}

impl ShadowUniform {
    /// Special `light_index` meaning nothing is shadowed.
    pub const NO_CASTER: u32 = u32::MAX;

    /// Safety factor on the geometric minimum bias, covering little depth precision
    /// and error from the normal map's perturbation of a geometry's normals.
    pub const BIAS_SAFETY: f32 = 2.0;
    /// The constant term, as a share of the slope term.
    pub const DEPTH_BIAS_FRACTION: f32 = 0.5;

    /// How far in front of the camera the fitted shadow volume reaches.
    pub const SHADOW_DISTANCE: f32 = 40.0;
    /// Extra room on the light's side of the volume, so casters that sit
    /// off-screen or behind the camera still land in the map.
    pub const CASTER_PAD: f32 = 30.0;

    /// Picks the first directional light as the caster and fits the shadow
    /// volume around the camera's view, clipped to the casters that actually
    /// exist. With no directional light the result leaves every surface lit.
    ///
    /// `caster_bounds` is the world-space bounds of everything that can cast.
    /// Supplying it is what keeps the map's resolution useful. Otherwise the 
    /// fit is to the camera's view volume.
    pub fn from_camera(
        camera: &Camera,
        lights: &[Light],
        caster_bounds: Option<Aabb>,
        map_size: u32,
    ) -> Self {
        assert!(map_size.is_power_of_two(), "shadow map size must be a power of two");
        assert!(map_size > 0, "shadow map size must be non-zero");
        let map_size = map_size.max(1) as f32;

        let mut uniform = Self {
            transform:   Mat4::IDENTITY,
            light_index: Self::NO_CASTER,
            _pad0:       0,
            // Only meaningful alongside a caster; the fit overwrites both.
            depth_bias:  0.0,
            slope_bias:  0.0,
            texel_size:  Vec2::splat(1.0 / map_size),
            _pad1:       [0.0; 2],
        };

        if let Some(index) = lights
            .iter()
            .position(|light| matches!(light.kind, LightKind::Directional))
        {
            uniform.light_index = index as u32;
            let fit = Self::fit_directional(camera, &lights[index], caster_bounds, map_size);
            uniform.transform  = fit.transform;
            uniform.depth_bias = fit.depth_bias;
            uniform.slope_bias = fit.slope_bias;
        }

        uniform
    }

    /// Fits an orthographic box around the slice of the camera frustum the map
    /// must cover, clipped to `caster_bounds`, and returns the light's
    /// view-projection for it along with biases scaled to that volume.
    ///
    /// Fitting to what the camera can actually see is what keeps the map's
    /// resolution useful; a fixed extent would waste texels and/or clip
    /// shadows as the camera moves.
    fn fit_directional(
        camera: &Camera,
        light: &Light,
        caster_bounds: Option<Aabb>,
        map_size: f32,
    ) -> ShadowFit {
        const EPSILON: f32 = 1e-8;
        // Direction towards the light; the shader lights from -light.direction.
        let to_light = if light.direction.length_squared() > EPSILON {
            -light.direction.normalize()
        } else {
            Vec3::Y
        };

        let forward = camera.forward();
        let right   = camera.right();
        let up      = camera.up();

        let half_h = (camera.fov_y * 0.5).tan();
        let half_w = half_h * camera.aspect;

        // The camera's view volume, truncated at SHADOW_DISTANCE. The near end
        // collapses to the eye - nothing closer to the camera than the camera
        // itself can be seen - so the shape is a wedge rather than a box: an
        // apex at the eye and a quad face at the far end.
        let mut frustum_corners = [Vec3::ZERO; 8];
        let signs = [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)];
        for (i, (sx, sy)) in signs.into_iter().enumerate() {
            let ray = (forward + right * (half_w * sx) + up * (half_h * sy)).normalize();
            frustum_corners[i]     = camera.position;
            frustum_corners[i + 4] = camera.position + ray * Self::SHADOW_DISTANCE;
        }

        // Bounding box of the camera's view frustum. Only boxes get intersected below, so the
        // clip is a superset of the true view_volume-to-caster overlap, which is what
        // we want: a bound must never cut a caster.
        let view_bounds = Aabb::from_points(frustum_corners);

        // With casters to clip to, the clipped box bounds the fit. Without, or
        // if the camera sees none of the casters, the view volume's own corners pin
        // it far more tightly than the corners of its bounding box, which stick
        // out well past the view volume.
        let region = match caster_bounds.filter(Aabb::is_finite) {
            Some(caster) => {
                let clipped = view_bounds.intersection(&caster);
                if clipped.has_volume() {
                    clipped
                } else {
                    view_bounds
                }
            }
            None => view_bounds,
        };

        let centre = region.center();
        let radius = region.radius();

        // A light pointing straight down would make look_at degenerate.
        let up_hint = if to_light.dot(Vec3::Y).abs() > 0.99999 { Vec3::Z } else { Vec3::Y };
        // Stand the light at volume position - radius - pad, so the whole
        // of it lands in front of the eye; ortho needs its near plane above zero.
        let eye  = centre + to_light * (radius + Self::CASTER_PAD + 1.0);
        let view = Mat4::look_at_rh(eye, centre, up_hint);

        // Bound the region in light space, where the ortho box is aligned.
        let light_space = region.transform_affine(view);
        let extents = light_space.extents();

        // A little slack so casters just outside the view still contribute.
        let pad        = extents * 0.05;
        let mut left   = light_space.min.x - pad.x;
        let mut bottom = light_space.min.y - pad.y;
        let width      = extents.x + pad.x * 2.0;
        let height     = extents.y + pad.y * 2.0;

        // The fit shifts continuously as the camera moves, which makes shadow
        // edges crawl; snapping the origin to whole texels stops the shimmer.
        let texel = width / map_size;
        if texel > 0.0 {
            left   = (left / texel).floor() * texel;
            bottom = (bottom / texel).floor() * texel;
        }
        let right = left + width;
        let top   = bottom + height;

        // View space looks down -z, so depth towards the light is +z. Pulling
        // the near plane back by the pad is what actually admits casters: the
        // standoff above cancels out of `near` and `far` alike, so without this
        // the volume holds nothing but the camera's own view volume, and anything
        // reaching past it (e.g. the top of an object beside a low camera)
        // gets clipped and silently stops casting.
        let near = (-light_space.max.z - Self::CASTER_PAD).max(0.1);
        let far  = (-light_space.min.z).max(near + 1.0);

        // A texel covers `texel_world_size` units across the light, and depth varies
        // across that texel by `texel_world_size * tan(theta)` on a surface tilted
        // `theta` from facing the light. Over the depth span that variation is
        // the NDC error to cancel, which is the shader's slope term.
        let depth_span  = (far - near).max(1e-4);
        let texel_world_size = width.max(height) / map_size;
        let slope_bias  = texel_world_size / depth_span * Self::BIAS_SAFETY;
        let depth_bias  = slope_bias * Self::DEPTH_BIAS_FRACTION;

        ShadowFit {
            transform: Mat4::orthographic_rh(left, right, bottom, top, near, far) * view,
            depth_bias,
            slope_bias,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Quat;

    /// The demo scene: a 10x10 ground plane at y = -1 with objects on it.
    const SCENE_BOUNDS: Aabb = Aabb {
        min: Vec3::new(-5.0, -1.0, -5.0),
        max: Vec3::new( 5.0,  2.2,  5.0),
    };

    fn demo_light() -> Light {
        Light {
            kind:      LightKind::Directional,
            position:  Vec3::ZERO,
            direction: Vec3::new(-1.0, -2.0, -1.0).normalize(),
            color:     Vec3::ONE,
            intensity: 1.0,
            range:     0.0,
        }
    }

    fn demo_camera() -> Camera {
        Camera::new(Vec3::new(0.0, 2.0, 5.0), 16.0 / 9.0)
    }

    /// Fits a shadow volume for the demo camera and light.
    fn fit(bounds: Option<Aabb>, map_size: u32) -> ShadowUniform {
        let light = demo_light();
        ShadowUniform::from_camera(&demo_camera(), std::slice::from_ref(&light), bounds, map_size)
    }

    /// Where a point lands in the shadow map's normalised space.
    fn project(uniform: &ShadowUniform, point: Vec3) -> Vec3 {
        let clip = uniform.transform * point.extend(1.0);
        clip.truncate() / clip.w
    }

    /// Fails when a point falls outside the fitted volume.
    fn assert_inside(ndc: Vec3, what: &str) {
        assert!(
            ndc.x.abs() <= 1.0 && ndc.y.abs() <= 1.0 && (0.0..=1.0).contains(&ndc.z),
            "{what} left the shadow volume: ndc = {ndc:?}",
        );
    }

    /// Span a box occupies in the map, where 2.0 covers the whole of NDC.
    fn ndc_span(uniform: &ShadowUniform, bounds: Aabb) -> f32 {
        let projected = Aabb::from_points(bounds.corners().map(|c| project(uniform, c)));
        projected.extents().truncate().max_element()
    }

    #[test]
    fn shadow_uniform_matches_shader_layout() {
        use std::mem::{offset_of, size_of};
        assert_eq!(size_of::<ShadowUniform>(), 96);
        assert_eq!(offset_of!(ShadowUniform, transform), 0);
        assert_eq!(offset_of!(ShadowUniform, light_index), 64);
        assert_eq!(offset_of!(ShadowUniform, depth_bias), 72);
        assert_eq!(offset_of!(ShadowUniform, slope_bias), 76);
        assert_eq!(offset_of!(ShadowUniform, texel_size), 80);
    }

    #[test]
    fn fitted_volume_covers_the_visible_frustum() {
        let camera  = demo_camera();
        let uniform = fit(None, 2048);
        assert_eq!(uniform.light_index, 0);

        let forward = camera.forward();
        let right   = camera.right();
        let up      = camera.up();
        let half_h  = (camera.fov_y * 0.5).tan();
        let half_w  = half_h * camera.aspect;

        for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            let ray = (forward + right * (half_w * sx) + up * (half_h * sy)).normalize();
            for t in [0.0, ShadowUniform::SHADOW_DISTANCE * 0.5, ShadowUniform::SHADOW_DISTANCE] {
                let ndc = project(&uniform, camera.position + ray * t);
                // The nearest corner lands exactly on the near plane, so allow
                // rounding to dip a hair below zero.
                let eps = 1e-3;
                assert!(ndc.x.abs() <= 1.0 + eps, "x escaped the map: {}", ndc.x);
                assert!(ndc.y.abs() <= 1.0 + eps, "y escaped the map: {}", ndc.y);
                assert!(ndc.z >= -eps && ndc.z <= 1.0 + eps, "z escaped 0..1: {}", ndc.z);
            }
        }
    }

    #[test]
    fn casters_behind_the_camera_stay_in_the_volume() {
        let camera  = demo_camera();
        let uniform = fit(None, 2048);

        // The camera looks down -z and this light comes from +z, so a caster a
        // few units behind the eye sits between the light and everything the
        // camera can see. It has to be in the map or it stops casting.
        for behind in [1.0f32, 3.0, 6.0] {
            let ndc = project(&uniform, camera.position + Vec3::Z * behind);
            assert!(
                (0.0..=1.0).contains(&ndc.z),
                "a caster {behind} units behind the camera was clipped (z = {})",
                ndc.z,
            );
        }
    }

    #[test]
    fn casters_near_a_low_camera_stay_in_the_volume() {
        // Standing near the ground the view wedge is almost flat, so its
        // highest point is the eye itself. Anything poking above that - the top
        // of a cube beside the camera, say - still has to be in the map, or the
        // object stops casting the very shadows the camera is looking at.
        let mut camera = demo_camera();
        camera.position    = Vec3::new(0.5, -0.85, 1.5);
        camera.orientation = Quat::from_rotation_x(-0.4);

        let light = demo_light();
        let uniform =
            ShadowUniform::from_camera(&camera, std::slice::from_ref(&light), None, 2048);

        let cube = Aabb { min: Vec3::splat(-0.5), max: Vec3::splat(0.5) };
        for corner in cube.corners() {
            assert_inside(project(&uniform, corner), &format!("cube corner {corner:?}"));
        }
    }

    #[test]
    fn volume_tightens_to_the_caster_bounds() {
        // Clipping to the bounds must not drop any part of them.
        let fitted = fit(Some(SCENE_BOUNDS), 2048);
        for corner in SCENE_BOUNDS.corners() {
            assert_inside(project(&fitted, corner), &format!("caster corner {corner:?}"));
        }

        // After clipping the casters should fill nearly the whole map: the
        // measured span is ~1.8 of the 2.0 that NDC covers.
        let tight = ndc_span(&fitted, SCENE_BOUNDS);
        assert!(
            tight > 1.7,
            "clipped fit leaves the scene in only {:.0}% of the map",
            tight / 2.0 * 100.0,
        );

        // Without clipping the same casters occupy under half of it, which is
        // the waste the bounds are there to remove.
        let loose = ndc_span(&fit(None, 2048), SCENE_BOUNDS);
        assert!(
            loose < 0.8,
            "the unclipped fit was expected to waste the map, got {loose:.2}",
        );
        assert!(
            tight > loose * 2.0,
            "clipping only gained {loose:.2} -> {tight:.2} NDC span",
        );
    }

    #[test]
    fn bias_follows_the_shadow_map_resolution() {
        // Bias has to track texel size: halving it halves the bias. Fixed
        // constants cannot, which is how the over-bias crept in - the fit was
        // retuned for resolution and the constants stayed where they were.
        let coarse = fit(Some(SCENE_BOUNDS), 1024);
        let fine   = fit(Some(SCENE_BOUNDS), 4096);

        for (what, coarse, fine) in [
            ("slope_bias", coarse.slope_bias, fine.slope_bias),
            ("depth_bias", coarse.depth_bias, fine.depth_bias),
        ] {
            let ratio = coarse / fine;
            assert!(
                (ratio - 4.0).abs() < 1e-3,
                "{what} did not follow the texel size: 1024/4096 ratio {ratio}",
            );
        }
    }

    #[test]
    fn bias_stays_small_in_world_units() {
        // The biases are NDC fractions, so their meaning depends on the fitted
        // depth span. Pinning the world distance down is what keeps them from
        // silently growing into tens of centimetres and eroding shadow edges.
        let uniform = fit(Some(SCENE_BOUNDS), 2048);

        // Recover the depth span by stepping one world unit along the light.
        let to_light = -demo_light().direction.normalize();
        let span = 1.0 / (project(&uniform, Vec3::ZERO).z - project(&uniform, to_light).z).abs();

        // tan is clamped at 4 in the shader, so that is the worst case.
        let worst = (uniform.depth_bias + uniform.slope_bias * 4.0) * span;
        assert!(
            worst < 0.25,
            "worst-case bias is {worst:.4} world units, which erodes shadow edges",
        );
        assert!(worst > 0.0, "bias must not be zero or shadows will acne");
    }

    #[test]
    fn no_directional_light_means_no_caster() {
        let camera = Camera::new(Vec3::ZERO, 1.0);
        let point = Light {
            kind:      LightKind::Point,
            position:  Vec3::ZERO,
            direction: Vec3::ZERO,
            color:     Vec3::ONE,
            intensity: 1.0,
            range:     1.0,
        };
        let uniform =
            ShadowUniform::from_camera(&camera, std::slice::from_ref(&point), None, 1024);
        assert_eq!(uniform.light_index, ShadowUniform::NO_CASTER);
    }

    #[test]
    fn aabb_round_trips_through_its_corners() {
        let bounds = Aabb { min: Vec3::new(-1.0, 2.0, -3.0), max: Vec3::new(4.0, 5.0, 6.0) };
        assert_eq!(Aabb::from_points(bounds.corners()), bounds);

        // The empty box has to stay empty, so accumulation has a valid start.
        assert!(Aabb::EMPTY.is_empty());
        assert!(Aabb::from_points(std::iter::empty()).is_empty());
        assert!(!Aabb::EMPTY.is_finite());
    }
}