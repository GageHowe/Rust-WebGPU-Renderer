// use wgpu::util::DeviceExt;
// use wgpu::{Buffer, Device, Queue};

// pub struct StorageBuffer<T> {
//     pub buffer: Buffer,
//     pub len: usize,
//     pub alignment: usize,
//     pub element_size: usize,
// }

// impl<T: bytemuck::Pod> StorageBuffer<T> {
//     pub fn new(device: &Device, initial: &[T], usage: wgpu::BufferUsages) -> Self {
//         // Optionally: ensure size respects limits (pad as needed for std430)
//         let element_size = std::mem::size_of::<T>();
//         let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Storage (SSBO) Buffer"),
//             contents: bytemuck::cast_slice(initial),
//             usage: usage | wgpu::BufferUsages::STORAGE,
//         });
//         Self {
//             buffer,
//             len: initial.len(),
//             alignment: element_size, // For simple usage; see device.limits for strict requirements.
//             element_size,
//         }
//     }

//     pub fn write(&self, queue: &Queue, data: &[T]) {
//         assert!(
//             data.len() <= self.len,
//             "Write data size exceeds buffer capacity"
//         );
//         queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
//     }
// }
