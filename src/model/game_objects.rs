use glam::{Mat3, Quat, Vec3};
// use glm::{Vec3, cos, cross, max, min, normalize, radians, sin};

/// still no idea wtf this is for
pub struct Object {
    pub position: Vec3,
    pub angle: f32,
}

pub struct Camera {
    pub position: Vec3,
    pub forwards: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new() -> Self {
        let position = Vec3::new(-5.0, 0.0, 2.0);
        let yaw = 0.0;
        let pitch = 0.0;
        let forwards = Vec3::new(1.0, 0.0, 0.0);
        let right = Vec3::new(0.0, -1.0, 0.0);
        let up = Vec3::new(0.0, 0.0, 1.0);
        Camera {
            position,
            forwards,
            right,
            up,
            yaw,
            pitch,
        }
    }

    pub fn spin(&mut self, d_yaw: f32, d_pitch: f32) {
        self.yaw = (self.yaw + d_yaw) % 360.0;
        if self.yaw < 0.0 {
            self.yaw += 360.0;
        }
        self.pitch = self.pitch + d_pitch;
        if self.pitch > 89.0 {
            self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        let c = self.yaw.to_radians().cos();
        let s = self.yaw.to_radians().sin();
        let c2 = self.pitch.to_radians().cos();
        let s2 = self.pitch.to_radians().sin();

        self.forwards = Vec3::new(c * c2, s * c2, s2);
        self.up = Vec3::new(0.0, 0.0, 1.0);
        self.right = self.forwards.cross(self.up).normalize();
        self.up = self.right.cross(self.forwards).normalize();
    }
}
