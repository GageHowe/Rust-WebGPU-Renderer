// use crate::model::game_objects::{Camera, Object};
use crate::renderer::backend::definitions::{Camera, InstanceData, Model};
use crate::renderer::backend::{
    bind_group_layout,
    mesh_builder::ObjLoader,
    pipeline,
    texture::{Texture, new_color, new_depth_texture, new_texture},
    ubo::{UBO, UBOGroup},
};
use glam::*;
use glfw::Window;
use std::collections::HashMap;
use wgpu::VertexBufferLayout;
use wgpu::util::DeviceExt;

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
    pub ubo_group: Option<UBOGroup>,
    projection_ubo: UBO,
    bind_group_layouts: HashMap<BindScope, wgpu::BindGroupLayout>,
    models: Vec<Model>,
    materials: Vec<Material>,
    depth_buffer: Texture,
    pub object_instances: Vec<InstanceData>,
    // /// currently only supports one kind of object
    // instance_buffer: Option<wgpu::Buffer>,
    // instance_count: usize,
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
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
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

        let projection_ubo = UBO::new(&device, &bind_group_layouts[&BindScope::UBO]);

        let depth_buffer = new_depth_texture(&device, &config, "Depth Buffer");

        // let i_b = Some(device.create_buffer_init(VertexBufferLayout{
        // }));
        Self {
            instance,
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipelines,
            ubo_group: None,
            projection_ubo: projection_ubo,
            bind_group_layouts: bind_group_layouts,
            models: Vec::new(),
            materials: Vec::new(),
            depth_buffer,
            object_instances: Vec::new(),
            // i_b,
            // 0,
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
        let mut builder = pipeline::Builder::new(device);

        // Colored Model Pipeline
        builder.set_shader_module("shaders/colored_model_shader.wgsl", "vs_main", "fs_main");
        builder.set_pixel_format(config.format);
        builder.add_vertex_buffer_layout(ModelVertex::get_layout());
        builder.add_bind_group_layout(&bind_group_layouts[&BindScope::Color]);
        builder.add_bind_group_layout(&bind_group_layouts[&BindScope::UBO]);
        builder.add_bind_group_layout(&bind_group_layouts[&BindScope::UBO]);
        pipelines.insert(
            PipelineType::ColoredModel,
            builder.build("Colored Model Pipeline"),
        );

        // Textured Model Pipeline
        builder.set_shader_module("shaders/textured_model_shader.wgsl", "vs_main", "fs_main");
        builder.set_pixel_format(config.format);
        builder.add_vertex_buffer_layout(ModelVertex::get_layout());
        builder.add_bind_group_layout(&bind_group_layouts[&BindScope::Texture]);
        builder.add_bind_group_layout(&bind_group_layouts[&BindScope::UBO]);
        builder.add_bind_group_layout(&bind_group_layouts[&BindScope::UBO]);
        pipelines.insert(
            PipelineType::TexturedModel,
            builder.build("Textured Model Pipeline"),
        );

        pipelines
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

    pub fn build_ubos_for_objects(&mut self, object_count: usize) {
        self.ubo_group = Some(UBOGroup::new(
            &self.device,
            object_count,
            &self.bind_group_layouts[&BindScope::UBO],
        ));
    }

    fn update_projection(&mut self, camera: &Camera) {
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
        let z_far = 1000.0;
        let projection = Mat4::perspective_rh(fov_y, aspect, z_near, z_far);

        let view_proj = projection * view;

        self.projection_ubo.upload(&view_proj, &self.queue);
    }

    fn update_transforms(&mut self, objects: &Vec<InstanceData>) {
        for (i, obj) in objects.iter().enumerate() {
            // let rotation = Mat4::from_rotation_z(obj.angle);
            let rotation = Mat4::from_quat(obj.rotation);
            let translation = Mat4::from_translation(obj.position);
            let matrix = rotation * translation;
            self.ubo_group
                .as_mut()
                .unwrap()
                .upload(i as u64, &matrix, &self.queue);
        }
    }

    // pub fn update_instance_buffer(&mut self, instances: &[InstanceData]) {
    //     let instance_data: Vec<InstanceRaw> =
    //         instances.iter().map(InstanceRaw::from_instance).collect();

    //     let buffer = self
    //         .device
    //         .create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //             label: Some("Instance Buffer"),
    //             contents: bytemuck::cast_slice(&instance_data),
    //             usage: wgpu::BufferUsages::VERTEX,
    //         });
    //     // self.instance_buffer = Some(buffer);
    //     // self.instance_count = instances.len();
    // }

    fn render_model(
        &self,
        objs: &Vec<InstanceData>,
        model: &Model,
        renderpass: &mut wgpu::RenderPass,
    ) {
        // Bind vertex and index buffer
        renderpass.set_vertex_buffer(0, model.buffer.slice(0..model.ebo_offset));
        renderpass.set_index_buffer(
            model.buffer.slice(model.ebo_offset..),
            wgpu::IndexFormat::Uint32,
        );

        // Transforms
        renderpass.set_bind_group(1, &(self.ubo_group.as_ref().unwrap()).bind_groups[0], &[]);
        //renderpass.set_bind_group(2, &self.proselfjection_ubo.bind_group, &[]);

        for submesh in &model.submeshes {
            let material = &self.materials[submesh.material_id];
            renderpass.set_pipeline(&self.render_pipelines[&material.pipeline_type]);
            renderpass.set_bind_group(0, (material.bind_group).as_ref().unwrap(), &[]);
            // renderpass.draw_indexed(0..submesh.index_count, submesh.first_index, 0..1);
            renderpass.draw_indexed(
                0..submesh.index_count,
                submesh.first_index,
                0..objs.len() as u32,
            );
        }
    }

    pub fn render(
        &mut self,
        instances: &Vec<InstanceData>,
        camera: &Camera,
    ) -> Result<(), wgpu::SurfaceError> {
        // self.device.poll(wgpu::MaintainBase::Wait).ok();
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });

        self.update_projection(camera);
        // self.update_transforms(quads, tris);
        self.update_transforms(instances); // still don't know why this is necessary to render the cube

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

        {
            let mut renderpass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
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
                })],
                depth_stencil_attachment: Some(depth_stencil_attachment),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            renderpass.set_bind_group(2, &self.projection_ubo.bind_group, &[]);

            // RENDER THE MODEL INSTANCES
            // self.render_model(instances, &self.models[0], &mut renderpass);
            let model = &self.models[0]; // inlined
            renderpass.set_vertex_buffer(0, model.buffer.slice(0..model.ebo_offset));
            renderpass.set_index_buffer(
                model.buffer.slice(model.ebo_offset..),
                wgpu::IndexFormat::Uint32,
            );

            renderpass.set_bind_group(1, &(self.ubo_group.as_ref().unwrap()).bind_groups[0], &[]);
            //renderpass.set_bind_group(2, &self.proselfjection_ubo.bind_group, &[]);

            for submesh in &model.submeshes {
                let material = &self.materials[submesh.material_id];
                renderpass.set_pipeline(&self.render_pipelines[&material.pipeline_type]);
                renderpass.set_bind_group(0, (material.bind_group).as_ref().unwrap(), &[]);
                renderpass.draw_indexed(0..submesh.index_count, submesh.first_index, 0..1);
                // renderpass.draw_indexed(
                //     0..submesh.index_count,
                //     submesh.first_index,
                //     0..self.object_instances.len() as u32,
                // );
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
