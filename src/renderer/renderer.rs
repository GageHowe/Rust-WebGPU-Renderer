// use crate::model::game_objects::{Camera, Object};
use crate::renderer::backend::definitions::{Camera, InstanceData, Model};
use crate::renderer::backend::{
    bind_group_layout,
    mesh_builder::ObjLoader,
    pipeline,
    texture::{Texture, new_color, new_depth_texture, new_texture},
    // ubo::UBOGroup,
};
use glam::*;
use glfw::Window;
use std::collections::HashMap;

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
    // pub ubo_group: Option<UBOGroup>,
    bind_group_layouts: HashMap<BindScope, wgpu::BindGroupLayout>,
    models: Vec<Model>, // convert to map of string to Model?
    materials: Vec<Material>,
    depth_buffer: Texture,
    pub object_instances: Vec<InstanceData>,
    pub instance_buffer: wgpu::Buffer,
    pub instance_count: u32,
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

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: 1, // resized later
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instance_count = 0;

        Self {
            instance,
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipelines,
            // ubo_group: None,
            bind_group_layouts: bind_group_layouts,
            models: Vec::new(),
            materials: Vec::new(),
            depth_buffer,
            object_instances: Vec::new(),
            instance_buffer,
            instance_count,
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
        scope = BindScope::UBO;
        layout = builder.build("UBO Bind Group Layout");
        layouts.insert(scope, layout);

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
        // don't add UBO bind group
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
        // don't add UBO bind group
        pipelines.insert(
            PipelineType::TexturedModel,
            pb.build("Textured Model Pipeline"),
        );

        return pipelines;
    }

    pub fn update_instance_buffer(&mut self, instances: &Vec<InstanceData>) {
        self.instance_count = instances.len() as u32;

        // Reallocate if needed
        let size = (instances.len() * std::mem::size_of::<InstanceData>()) as u64;
        if self.instance_buffer.size() < size {
            self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer"),
                size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        self.queue
            .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
    }

    pub fn load_assets(&mut self, filepath: &str) {
        let mut loader = ObjLoader::new();

        self.models.push(loader.load(
            filepath,
            &mut self.materials,
            &self.device,
            &glam::Mat4::IDENTITY,
        ));

        for material in &mut self.materials {
            material.bind_group = match material.pipeline_type {
                PipelineType::ColoredModel => Some(new_color(
                    &(material.color.unwrap()),
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
            }
        }
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

    // pub fn build_ubos_for_objects(&mut self, object_count: usize) {
    //     self.ubo_group = Some(UBOGroup::new(
    //         &self.device,
    //         object_count,
    //         &self.bind_group_layouts[&BindScope::UBO],
    //     ));
    // }

    fn update_projection(&mut self, camera: &Camera) -> Mat4 {
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

    pub fn render(
        &mut self,
        instances: &Vec<InstanceData>,
        camera: &Camera,
    ) -> Result<(), wgpu::SurfaceError> {
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });

        self.update_projection(camera);
        self.update_instance_buffer(instances);

        // housekeeping
        _ = self.queue.submit([]);
        _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        let drawable = self.surface.get_current_texture()?;
        let image_view_descriptor = wgpu::TextureViewDescriptor::default();
        let image_view = drawable.texture.create_view(&image_view_descriptor);
        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        };
        let mut command_encoder = self
            .device
            .create_command_encoder(&command_encoder_descriptor);
        let depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_buffer.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        };

        // render
        {
            let mut renderpass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(
                    wgpu::RenderPassColorAttachment /* index 0 (@location(0)) */{
                    view: &image_view,
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
                },
                )],
                depth_stencil_attachment: Some(depth_stencil_attachment),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            let view_proj = self.update_projection(camera);

            // RENDER THE MODEL INSTANCES
            // self.render_model(instances, &self.models[0], &mut renderpass);
            // TODO: iterate through all models. For each model, draw all instances in one pass
            let model = &self.models[0];
            renderpass.set_vertex_buffer(0, model.buffer.slice(0..model.ebo_offset));
            renderpass.set_index_buffer(
                model.buffer.slice(model.ebo_offset..),
                wgpu::IndexFormat::Uint32,
            );

            for submesh in &model.submeshes {
                let material = &self.materials[submesh.material_id];
                renderpass.set_pipeline(&self.render_pipelines[&material.pipeline_type]);
                renderpass.set_push_constants(
                    wgpu::ShaderStages::VERTEX,
                    0,
                    mat4_as_bytes(&view_proj),
                );

                renderpass.set_bind_group(0, (material.bind_group).as_ref().unwrap(), &[]);

                renderpass.set_vertex_buffer(0, model.buffer.slice(0..model.ebo_offset));
                renderpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                renderpass.draw_indexed(
                    0..submesh.index_count,
                    submesh.first_index,
                    0..self.instance_count,
                );
            }
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        drawable.present();
        Ok(())
    }
}

pub fn mat4_as_bytes(m: &glam::Mat4) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts((m as *const Mat4) as *const u8, std::mem::size_of::<Mat4>())
    }
}
