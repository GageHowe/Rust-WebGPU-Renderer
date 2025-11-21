use glam::*;
use glfw::*;

#[derive(Eq, Hash, PartialEq)]
pub enum BindScope {
    Texture,
    Color,
}

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum PipelineType {
    TexturedModel, // if the model has a texture
    ColoredModel,  // fallback
}

pub struct Material {
    pub pipeline_type: PipelineType,
    pub color: Option<Vec4>,
    pub filename: Option<String>,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl Material {
    pub fn new() -> Self {
        Material {
            pipeline_type: PipelineType::ColoredModel,
            color: Some(Vec4::new(0.5, 0.0, 0.5, 1.0)),
            filename: None,
            bind_group: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Submesh {
    pub first_index: i32,
    pub index_count: u32,
    pub material_id: usize,
}

/// 3d models
pub struct Model {
    pub buffer: wgpu::Buffer,
    ///location where the Element Buffer Object (index buffer) starts in `buffer`
    pub ebo_offset: u64,
    pub submeshes: Vec<Submesh>,
}

/// describes a vertex with its position, texture coordinates, and normal
#[repr(C)] // C-style data layout
pub struct VertexData {
    pub position: Vec3,
    pub tex_coord: Vec2,
    pub normal: Vec3,
}

impl VertexData {
    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x2,
            2 => Float32x3];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<VertexData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

/// This describes information needed to send information about multiple instances
/// of a model to the GPU for batching/instancing.
/// https://sotrh.github.io/learn-wgpu/beginner/tutorial7-instancing/
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub model: [[f32; 4]; 4],
}

impl InstanceData {
    pub fn from_pos_rot(pos: glam::Vec3, rot: glam::Quat, scale: f32) -> Self {
        let model = glam::Mat4::from_scale_rotation_translation(glam::Vec3::splat(scale), rot, pos);

        Self {
            model: model.to_cols_array_2d(),
        }
    }
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

    pub fn update(&mut self, dt: f32, window: &mut glfw::Window) {
        if !window.is_focused() {
            // TODO: make character trait and make this check in the "super" call
            return;
        }

        let speed = 0.5 * dt;

        let mouse_pos = window.get_cursor_pos();
        window.set_cursor_pos(400.0, 300.0);
        let dx = (-40.0 * (mouse_pos.0 - 400.0) / 400.0) as f32;
        let dy = (-40.0 * (mouse_pos.1 - 300.0) / 300.0) as f32;
        self.look(dx, dy);

        if window.get_key(Key::W) == Action::Press {
            self.position += self.forwards * speed;
        }
        if window.get_key(Key::S) == Action::Press {
            self.position -= self.forwards * speed;
        }
        if window.get_key(Key::A) == Action::Press {
            self.position -= self.right * speed;
        }
        if window.get_key(Key::D) == Action::Press {
            self.position += self.right * speed;
        }
        if window.get_key(Key::Space) == Action::Press {
            self.position += self.up * speed;
        }
        if window.get_key(Key::LeftShift) == Action::Press {
            self.position -= self.up * speed;
        }
    }

    fn look(&mut self, d_yaw: f32, d_pitch: f32) {
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
