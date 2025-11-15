use glam::*;

#[derive(Eq, Hash, PartialEq)]
pub enum BindScope {
    Texture,
    Color,
    UBO,
}

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum PipelineType {
    Simple,
    TexturedModel,
    ColoredModel,
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
            pipeline_type: PipelineType::Simple,
            color: None,
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

pub struct Model {
    pub buffer: wgpu::Buffer,
    pub ebo_offset: u64,
    pub submeshes: Vec<Submesh>,
}

#[repr(C)] // C-style data layout
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec3,
}

impl Vertex {
    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

#[repr(C)] // C-style data layout
pub struct ModelVertex {
    pub position: Vec3,
    pub tex_coord: Vec2,
    pub normal: Vec3,
}

impl ModelVertex {
    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x2,
            2 => Float32x3];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub struct Mesh {
    pub buffer: wgpu::Buffer,
    pub offset: u64,
}

pub struct Camera {
    pub position: Vec3,
    pub forwards: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

/// still no idea wtf this is for
pub struct Object {
    pub position: Vec3,
    pub angle: f32,
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
