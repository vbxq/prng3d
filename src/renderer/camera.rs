use glam::{Mat4, Vec2, Vec3};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    Free,
    Orbital,
}

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,

    pub target: Vec3,
    pub orbital_distance: f32,

    pub mode: CameraMode,

    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,

    pub move_speed: f32,
    pub mouse_sensitivity: f32,
    pub zoom_speed: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 800.0),
            yaw: -90.0_f32.to_radians(),
            pitch: 0.0,

            target: Vec3::ZERO,
            orbital_distance: 800.0,

            mode: CameraMode::Free,

            fov: 60.0_f32.to_radians(),
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 50000.0,

            move_speed: 500.0,
            mouse_sensitivity: 0.002,
            zoom_speed: 50.0,
        }
    }
}

impl Camera {
    pub fn front(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    pub fn right(&self) -> Vec3 {
        self.front().cross(Vec3::Y).normalize()
    }

    pub fn view_matrix(&self) -> Mat4 {
        match self.mode {
            CameraMode::Free => {
                Mat4::look_at_rh(self.position, self.position + self.front(), Vec3::Y)
            }
            CameraMode::Orbital => Mat4::look_at_rh(self.position, self.target, Vec3::Y),
        }
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    pub fn process_keyboard(&mut self, forward: f32, right: f32, up: f32, dt: f32) {
        if self.mode != CameraMode::Free {
            return;
        }

        let speed = self.move_speed * dt;
        let front = self.front();
        let right_vec = self.right();

        self.position += front * forward * speed;
        self.position += right_vec * right * speed;
        self.position.y += up * speed;
    }

    pub fn process_mouse_movement(&mut self, delta: Vec2) {
        let dx = delta.x * self.mouse_sensitivity;
        let dy = delta.y * self.mouse_sensitivity;

        self.yaw += dx;
        self.pitch -= dy;

        let max_pitch = 89.0_f32.to_radians();
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);

        if self.mode == CameraMode::Orbital {
            self.update_orbital_position();
        }
    }

    pub fn process_scroll(&mut self, delta: f32) {
        match self.mode {
            CameraMode::Free => {
                self.move_speed = (self.move_speed + delta * self.zoom_speed).clamp(10.0, 5000.0);
            }
            CameraMode::Orbital => {
                self.orbital_distance =
                    (self.orbital_distance - delta * self.zoom_speed).clamp(10.0, 10000.0);
                self.update_orbital_position();
            }
        }
    }

    pub fn set_mode(&mut self, mode: CameraMode) {
        if self.mode == mode {
            return;
        }

        match mode {
            CameraMode::Free => {
                let dir = (self.target - self.position).normalize();
                self.yaw = dir.z.atan2(dir.x);
                self.pitch = dir.y.asin();
                self.mode = CameraMode::Free;
            }
            CameraMode::Orbital => {
                self.mode = CameraMode::Orbital;
                self.orbital_distance = self.position.distance(self.target);

                let dir = (self.position - self.target).normalize();
                self.yaw = dir.z.atan2(dir.x);
                self.pitch = dir.y.asin();

                self.update_orbital_position();
            }
        }
    }

    fn update_orbital_position(&mut self) {
        self.position = self.target
            + Vec3::new(
                self.orbital_distance * self.yaw.cos() * self.pitch.cos(),
                self.orbital_distance * self.pitch.sin(),
                self.orbital_distance * self.yaw.sin() * self.pitch.cos(),
            );
    }

    pub fn set_aspect(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub _padding: f32,
}

impl CameraUniform {
    pub fn from_camera(camera: &Camera) -> Self {
        Self {
            view_proj: camera.view_projection_matrix().to_cols_array_2d(),
            camera_pos: camera.position.to_array(),
            _padding: 0.0,
        }
    }
}
