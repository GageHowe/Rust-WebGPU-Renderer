// use crate::model::game_objects::{Camera, Object};
use crate::renderer::backend::definitions::{Camera, InstanceData, Model};
use crate::renderer::backend::{
    bind_group_layout,
    mesh_builder::ObjLoader,
    pipeline,
    texture::{Texture, new_color, new_depth_texture, new_texture},
};
use glam::*;
use glfw::Window;
use std::collections::HashMap;
use std::hash::Hash;

use super::backend::definitions::*;

pub struct RendererState<'a> {
    /// a handle to our GPU
    instance: wgpu::Instance,
    /// the part of the window that we draw to
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    /// executes recorded CommandBuffer objects and provides convenience methods for writing to buffers
    queue: wgpu::Queue,
    /// screen size, max latency, etc
    config: wgpu::SurfaceConfiguration,
    pub size: (i32, i32),
    /// struct that wraps a *GLFWWindow handle
    pub window: &'a mut Window,
    /// map of pre-defined types to wgpu::RenderPipelines
    render_pipelines: HashMap<PipelineType, wgpu::RenderPipeline>,
    bind_group_layouts: HashMap<BindScope, wgpu::BindGroupLayout>,
    materials: Vec<Material>,
    depth_buffer: Texture,

    // models: Vec<Model>, // convert to map of string to Model?
    // pub object_instances: Vec<InstanceData>,
    // pub instance_buffer: wgpu::Buffer,
    // pub instance_count: u32,
    models: HashMap<String, Vec<Model>>,
    pub instances: HashMap<String, Vec<InstanceData>>,
    instance_buffers: HashMap<String, wgpu::Buffer>,
    pub instance_counts: HashMap<String, u32>,
}

impl<'a> RendererState<'a> {
    pub async fn new(window: &'a mut Window) -> Self {
        let size = window.get_framebuffer_size();

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        };

        let instance = wgpu::Instance::new(&instance_descriptor);
        let surface = instance.create_surface(window.render_context()).unwrap();

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        let adapter = instance.request_adapter(&adapter_descriptor).await.unwrap();

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::PUSH_CONSTANTS,
            required_limits: wgpu::Limits {
                max_push_constant_size: 64,
                ..wgpu::Limits::default()
            },
            memory_hints: wgpu::MemoryHints::Performance,
            label: Some("Device"),
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::default(),
        };

        let (device, queue) = adapter.request_device(&device_descriptor).await.unwrap();
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0 as u32,
            height: size.1 as u32,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let bind_group_layouts = Self::build_bind_group_layouts(&device);
        let render_pipelines = Self::build_pipelines(&device, &config, &bind_group_layouts);
        let depth_buffer = new_depth_texture(&device, &config, "Depth Buffer");

        // let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Instance Buffer"),
        //     size: 1, // resized later
        //     usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });

        Self {
            instance,
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipelines,
            bind_group_layouts: bind_group_layouts,
            materials: Vec::new(),
            depth_buffer,

            models: HashMap::new(),
            instances: HashMap::new(),
            instance_buffers: HashMap::new(),
            instance_counts: HashMap::new(), // initialize with 0?
        }
    }

    fn build_bind_group_layouts(
        device: &wgpu::Device,
    ) -> HashMap<BindScope, wgpu::BindGroupLayout> {
        let mut layouts: HashMap<BindScope, wgpu::BindGroupLayout> = HashMap::new();
        let mut layout: wgpu::BindGroupLayout;
        let mut scope = BindScope::Texture;
        let mut builder = bind_group_layout::Builder::new(device);
        builder.add_texture();
        layout = builder.build("Texture Bind Group Layout");
        layouts.insert(scope, layout);

        builder.add_vec4();
        scope = BindScope::Color;
        layout = builder.build("Color Group Layout");
        layouts.insert(scope, layout);

        builder.add_mat4();

        layouts
    }

    fn build_pipelines(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        bind_group_layouts: &HashMap<BindScope, wgpu::BindGroupLayout>,
    ) -> HashMap<PipelineType, wgpu::RenderPipeline> {
        let mut pipelines: HashMap<PipelineType, wgpu::RenderPipeline> = HashMap::new();
        let mut pb = pipeline::Builder::new(device);

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 32,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 48,
                    shader_location: 6,
                },
            ],
        };

        // Colored pipeline
        pb.set_shader_module("shaders/instanced_colored.wgsl", "vs_main", "fs_main");
        pb.set_pixel_format(config.format);
        pb.add_vertex_buffer_layout(VertexData::get_layout());
        pb.add_vertex_buffer_layout(instance_layout.clone());
        pb.add_bind_group_layout(&bind_group_layouts[&BindScope::Color]);
        pipelines.insert(
            PipelineType::ColoredModel,
            pb.build("Colored Model Pipeline"),
        );

        // Textured pipeline
        pb.set_shader_module("shaders/instanced_textured.wgsl", "vs_main", "fs_main");
        pb.set_pixel_format(config.format);
        pb.add_vertex_buffer_layout(VertexData::get_layout());
        pb.add_vertex_buffer_layout(instance_layout);
        pb.add_bind_group_layout(&bind_group_layouts[&BindScope::Texture]);
        pipelines.insert(
            PipelineType::TexturedModel,
            pb.build("Textured Model Pipeline"),
        );

        return pipelines;
    }

    // pub fn update_instance_buffer(&mut self, instances: &Vec<InstanceData>) {
    //     self.instance_count = instances.len() as u32;

    //     // Reallocate if needed
    //     let size = (instances.len() * std::mem::size_of::<InstanceData>()) as u64;
    //     if self.instance_buffer.size() < size {
    //         self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
    //             label: Some("Instance Buffer"),
    //             size,
    //             usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    //             mapped_at_creation: false,
    //         });
    //     }

    //     self.queue
    //         .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
    // }

    pub fn update_instance_buffer(&mut self) {
        for (key, instances) in &self.instances {
            let instance_count = instances.len() as u32;
            self.instance_counts.insert(key.clone(), instance_count);

            // Compute required buffer size
            let size = (instances.len() * std::mem::size_of::<InstanceData>()) as u64;

            // Check if a buffer exists AND if it is large enough
            let need_new_buffer = match self.instance_buffers.get(key) {
                Some(buf) => buf.size() < size,
                None => true,
            };

            // Reallocate if needed
            if need_new_buffer {
                let new_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("Instance Buffer: {}", key)),
                    size,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                self.instance_buffers.insert(key.clone(), new_buffer);
            }

            // Now safe to unwrap—buffer definitely exists
            let buffer = self.instance_buffers.get(key).unwrap();

            // Write the instance data into the GPU buffer
            if !instances.is_empty() {
                self.queue
                    .write_buffer(buffer, 0, bytemuck::cast_slice(instances));
            }
        }
    }

    pub fn load_assets(&mut self, id: &str, filepath: &str) {
        let mut loader = ObjLoader::new();

        let model: Model = loader.load(
            filepath,
            &mut self.materials,
            &self.device,
            &glam::Mat4::IDENTITY,
        );

        self.models.insert(id.to_string(), vec![model]);

        // build bindgroups for all materials
        for material in &mut self.materials {
            material.bind_group = match material.pipeline_type {
                PipelineType::ColoredModel => Some(new_color(
                    material.color.as_ref().unwrap(),
                    &self.device,
                    "Color",
                    &self.bind_group_layouts[&BindScope::Color],
                )),

                PipelineType::TexturedModel => Some(new_texture(
                    material.filename.as_ref().unwrap().as_str(),
                    &self.device,
                    &self.queue,
                    "Texture",
                    &self.bind_group_layouts[&BindScope::Texture],
                )),

                _ => None,
            };
        }

        self.instances.entry(id.to_string()).or_insert(Vec::new());
        self.instance_counts.entry(id.to_string()).or_insert(0);

        let placeholder_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Instance Buffer Placeholder: {}", id)),
            size: 1,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.instance_buffers
            .insert(id.to_string(), placeholder_buffer);
    }

    pub fn resize(&mut self, new_size: (i32, i32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            self.config.width = new_size.0 as u32;
            self.config.height = new_size.1 as u32;
            self.surface.configure(&self.device, &self.config);

            self.depth_buffer.texture.destroy();
            self.depth_buffer = new_depth_texture(&self.device, &self.config, "Depth Buffer");
        }
    }

    pub fn update_surface(&mut self) {
        self.surface = self
            .instance
            .create_surface(self.window.render_context())
            .unwrap();
    }

    fn update_projection(&self, camera: &Camera) -> Mat4 {
        // Vectors for view matrix columns
        let c0 = Vec4::new(camera.right.x, camera.up.x, -camera.forwards.x, 0.0);
        let c1 = Vec4::new(camera.right.y, camera.up.y, -camera.forwards.y, 0.0);
        let c2 = Vec4::new(camera.right.z, camera.up.z, -camera.forwards.z, 0.0);
        let a: f32 = -camera.right.dot(camera.position);
        let b: f32 = -camera.up.dot(camera.position);
        let c: f32 = camera.forwards.dot(camera.position);
        let c3 = Vec4::new(a, b, c, 1.0);

        let view = Mat4::from_cols(c0, c1, c2, c3);

        let fov_y: f32 = 80.0_f32.to_radians();
        let aspect = 4.0 / 3.0;
        let z_near = 0.5;
        let z_far = 10000.0;
        let projection = Mat4::perspective_rh(fov_y, aspect, z_near, z_far);

        projection * view
    }

    /// draws all objects in an instanced way.
    /// runs an instanced draw on each submesh/mat in each model
    pub fn render(&mut self, camera: &Camera) -> Result<(), wgpu::SurfaceError> {
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        self.update_instance_buffer();

        let drawable = self.surface.get_current_texture()?;
        let view = drawable
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.01,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_buffer.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        let view_proj = self.update_projection(camera);

        // draw loop
        for (id, model_list) in &self.models {
            let instance_count = match self.instance_counts.get(id) {
                Some(count) if *count > 0 => *count,
                _ => continue,
            };
            let instance_buffer = match self.instance_buffers.get(id) {
                Some(b) => b,
                None => continue,
            };

            for model in model_list {
                renderpass.set_vertex_buffer(0, model.buffer.slice(0..model.ebo_offset));
                renderpass.set_index_buffer(
                    model.buffer.slice(model.ebo_offset..),
                    wgpu::IndexFormat::Uint32,
                );
                renderpass.set_vertex_buffer(1, instance_buffer.slice(..));
                // draw each submesh with its own material
                for submesh in &model.submeshes {
                    let material = &self.materials[submesh.material_id];

                    renderpass.set_pipeline(&self.render_pipelines[&material.pipeline_type]);
                    renderpass.set_push_constants(
                        wgpu::ShaderStages::VERTEX,
                        0,
                        mat4_as_bytes(&view_proj),
                    );
                    renderpass.set_bind_group(0, material.bind_group.as_ref().unwrap(), &[]);

                    renderpass.draw_indexed(
                        0..submesh.index_count,
                        submesh.first_index,
                        0..instance_count,
                    );
                }
            }
        }

        drop(renderpass);

        self.queue.submit(Some(encoder.finish()));
        drawable.present();

        Ok(())
    }
}

pub fn mat4_as_bytes(m: &glam::Mat4) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts((m as *const Mat4) as *const u8, std::mem::size_of::<Mat4>())
    }
}
