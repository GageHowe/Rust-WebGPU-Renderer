use super::bind_group;
use super::mesh_builder::any_as_u8_slice;
use glam::*;

/// for managing multiple uniforms (matrices) in one large buffer with proper alignment and per-object bind groups, useful when you have many objects.
pub struct UBOGroup {
    pub buffer: wgpu::Buffer,
    /// A vector of wgpu::BindGroup, each representing a bind group used to bind a slice of the buffer to a shader.
    pub bind_groups: Vec<wgpu::BindGroup>,
    /// The required alignment size for the buffer offset to ensure hardware/GPU compatibility.
    alignment: u64,
}

impl UBOGroup {
    pub fn new(device: &wgpu::Device, object_count: usize, layout: &wgpu::BindGroupLayout) -> Self {
        let al = u64::max(
            device.limits().min_storage_buffer_offset_alignment as u64,
            std::mem::size_of::<Mat4>() as u64,
        );

        let buffer_descriptor = wgpu::BufferDescriptor {
            label: Some("UBO"),
            size: object_count as u64 * al,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let buffer = device.create_buffer(&buffer_descriptor);

        // build bind groups
        let mut bind_groups: Vec<wgpu::BindGroup> = Vec::new();
        for i in 0..object_count {
            let mut builder = bind_group::Builder::new(device);
            builder.set_layout(layout);
            builder.add_buffer(&buffer, i as u64 * al);
            bind_groups.push(builder.build("Matrix"));
        }

        Self {
            buffer,
            bind_groups,
            alignment: al,
        }
    }

    pub fn upload(&mut self, i: u64, matrix: &Mat4, queue: &wgpu::Queue) {
        let offset = i * self.alignment;
        let data: &[u8] = unsafe { any_as_u8_slice(matrix) };
        queue.write_buffer(&self.buffer, offset, data);
    }
}

/// Uniform Buffer Object - used for efficiently sending data to shaders
pub struct UBO {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl UBO {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let buffer_descriptor = wgpu::BufferDescriptor {
            label: Some("UBO"),
            size: std::mem::size_of::<Mat4>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let buffer = device.create_buffer(&buffer_descriptor);

        // build bind groups
        let bind_group: wgpu::BindGroup;
        {
            let mut builder = bind_group::Builder::new(device);
            builder.set_layout(layout);
            builder.add_buffer(&buffer, 0);
            bind_group = builder.build("Matrix");
        }

        Self { buffer, bind_group }
    }

    pub fn upload(&mut self, matrix: &Mat4, queue: &wgpu::Queue) {
        let offset = 0;
        let data: &[u8] = unsafe { any_as_u8_slice(matrix) };
        queue.write_buffer(&self.buffer, offset, data);
    }
}
