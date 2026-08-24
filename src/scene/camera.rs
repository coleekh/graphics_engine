use glam::{vec3, Mat3, Mat4, Quat, Vec3};

#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    pub scale: Vec3,
    pub orientation: Quat,
    up: Vec3,

    // technically fov_x
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,

    pub aspect: f32,
}

const DEFAULT_WORLD_UP: Vec3 = Vec3::Y;

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            position: Vec3::ZERO,
            up: DEFAULT_WORLD_UP,
            scale: Vec3::ONE,
            // fov_y: Radian::EIGTH_TURN, // aproximate human fov
            fov_y: std::f32::consts::FRAC_PI_4, // aproximate human fov
            z_near: 0.1,
            z_far: 500.0,
            orientation: Quat::IDENTITY,
            aspect: 16.0 / 9.0,
        }
    }
}

impl Camera {
    // creates new camera facing in the negative z
    pub fn new(position: Vec3, aspect: f32) -> Self {
        Camera {
                position,
                up: DEFAULT_WORLD_UP,
                scale: Vec3::ONE,
                // fov_y: Radian::EIGTH_TURN, // aproximate human fov
                fov_y: std::f32::consts::FRAC_PI_4, // aproximate human fov
                z_near: 0.1,
                z_far: 100.0,
                aspect,
                orientation: Quat::IDENTITY,
                // orientation: Quat::look_at_rh(position, Vec3::ZERO, DEFAULT_WORLD_UP),
        }
    }

    // Returns (bottom left corner frustum ray, top right corner frustum ray)
    /// the other corners of the frustum can be calculated from the returned ray
    pub fn ray(&self) -> Vec3 {
        let aspect = self.aspect;
        let half_fov_y = self.fov_y / 2.0;
        let half_fov_x = aspect * half_fov_y;
        // let rotation = self.orientation
        //     * Quat::axis_angle(self.fov_y / 4.0, Vec3::x())
        //     * Quat::axis_angle(aspect * self.fov_y / 4.0, Vec3::y());
        // let left_corner = rotation * Vec3::z() * rotation.conjugate();
        // left_corner
        // vec3(
        //     half_fov_x.sin() * half_fov_y.sin(),
        //     -half_fov_x.sin() * half_fov_y.sin(),
        //     half_fov_x.cos() * half_fov_y.cos(),
        // )
        // // vec3(
        // //     half_fov_x.cos() * half_fov_y.cos(),
        // //     half_fov_y.sin(),
        // //     half_fov_x.sin() * half_fov_y.cos(),
        // // )
        // .normalized()
        //     * self.orientation
        self.orientation.mul_vec3(vec3(0.43, -0.50, 0.75))

        // (A + B)/2
    }

    /// Returns the view matrix for transforming external objects into camera space.
    pub fn view(&self) -> Mat4 {
        Mat4::from_quat(self.orientation.conjugate())
        * Mat4::from_translation(-self.position)  
        * Mat4::from_scale(self.scale)
    }

    pub fn projection(&self) -> Mat4 {
        Mat4::perspective_rh(
            self.fov_y,
            self.aspect,
            self.z_near,
            self.z_far,
        )
    }
    
    pub fn rotation_projection(&self) -> Mat4 {
        let view = Mat4::from_quat(self.orientation);

        let projection = Mat4::perspective_rh(
            self.fov_y,
            self.aspect,
            self.z_near,
            self.z_far,
        );

        projection * view
    }

    pub fn inverse_rotation(&self) -> Mat3 {
        Mat3::from_quat(self.orientation.conjugate())
    }

    pub fn inverse_projection(&self) -> Mat4 {
        Mat4::perspective_rh(
            self.fov_y,
            self.aspect,
            self.z_near,
            self.z_far,
        )
    }

    /// Rotates the camera around the global y-axis (side to side).
    /// This is useful for first person style camera movement
    pub fn global_yaw(&mut self, angle: f32) {
        self.orientation = Quat::from_axis_angle(DEFAULT_WORLD_UP, angle) * self.orientation;
    }

    /// Rotates the camera around the local y-axis (side to side).
    pub fn yaw(&mut self, angle: f32) {
        self.orientation *= Quat::from_axis_angle(self.up, angle);
    }
    
    /// Rotates the camera around the local x-axis (up and down).
    pub fn pitch(&mut self, angle: f32) {
        self.orientation *= Quat::from_axis_angle(Vec3::X, angle);
    }

    pub fn scale(&mut self, factor: &Vec3) {
        self.scale.x *= factor.x;
        self.scale.y *= factor.y;
        self.scale.z *= factor.z;
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
    }

    /// changes position absolutely (relative to nothing)
    pub fn change_position(&mut self, translation: Vec3) {
        self.position += translation;
    }

    /// changes position relative to orientation of camera
    pub fn translate(&mut self, translation: Vec3) {
        self.position += self.orientation.mul_vec3(translation);
    }

    // pub fn set_viewport(&mut self, device: &wgpu::Device, viewport: AABB2D<i32>) {
    //     let dimensions = viewport.dimensions();
    //     self.width = dimensions.x as f32;
    //     self.height = dimensions.y as f32;

    //     // device.viewport(
    //     //     viewport.neg_corner.x,
    //     //     viewport.neg_corner.y,
    //     //     viewport.pos_corner.x,
    //     //     viewport.pos_corner.y,
    //     // );
    // }

    pub fn reset_viewport(&mut self) {
        // viewport(0, 0, self.width as i32, self.height as i32);
    }

    pub fn is_sphere_in_view(&self, position: Vec3, radius: f32) -> bool {
        let point_camera = self.orientation.conjugate().mul_vec3(position - self.position);

        let height = point_camera.z * 2.0 * (self.fov_y * 0.5).tan();
        let width = height * self.aspect;
        let distance_y = radius / (self.fov_y * 0.5).cos();
        let distance_x = radius / (self.aspect * self.fov_y * 0.5).cos();

        point_camera.x > -height * 0.5 - distance_y
            && point_camera.x < height * 0.5 + distance_y
            && point_camera.y > -width * 0.5 - distance_x
            && point_camera.y < width * 0.5 + distance_x
            && point_camera.z > self.z_near - radius
            && point_camera.z < self.z_far + radius
    }

    pub fn is_point_in_view(&self, point: Vec3) -> bool {
        let point_camera = self.orientation.conjugate().mul_vec3(point - self.position);

        let height = point_camera.z * 2.0 * (self.fov_y * 0.5).tan();
        let width = height * self.aspect;
        2.0 * point_camera.x > -width
            && 2.0 * point_camera.x < width
            && 2.0 * point_camera.y > -height
            && 2.0 * point_camera.y < height
            && point_camera.z > self.z_near
            && point_camera.z < self.z_far
    }

    // pub fn is_aabb_in_view(&self, aabb: &AABB3D<f32>) -> bool {
    //     let points = aabb.points();
    //     self.is_point_in_view(&points[0])
    //         || self.is_point_in_view(&points[1])
    //         || self.is_point_in_view(&points[2])
    //         || self.is_point_in_view(&points[3])
    //         || self.is_point_in_view(&points[4])
    //         || self.is_point_in_view(&points[5])
    //         || self.is_point_in_view(&points[6])
    //         || self.is_point_in_view(&points[7])
    // }
}
