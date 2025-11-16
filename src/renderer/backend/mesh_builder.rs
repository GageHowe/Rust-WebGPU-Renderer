use crate::renderer::backend::definitions::{Model, PipelineType, Submesh};
// use crate::utility::string::split;
use glam::*;
use std::collections::HashMap;
use std::path::Path;
use wgpu::util::DeviceExt;

use super::definitions::{Material, ModelVertex};

// From: https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8
pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
    }
}

pub struct ObjLoader;

impl ObjLoader {
    pub fn new() -> Self {
        ObjLoader
    }

    pub fn load(
        &mut self,
        filename: &str,
        materials_out: &mut Vec<Material>,
        device: &wgpu::Device,
        pre_transform: &Mat4,
    ) -> Model {
        let obj_path = Path::new(filename);

        let (models, materials) = tobj::load_obj(
            obj_path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .expect("tobj failed");

        // convert materials
        let mtl_dir = obj_path.parent().unwrap_or(Path::new(""));

        for m in materials.unwrap_or_default() {
            let mut mat = Material::new();

            if let Some(path) = m.diffuse_texture {
                mat.pipeline_type = PipelineType::TexturedModel;
                mat.filename = Some(mtl_dir.join(path).to_string_lossy().to_string());
            } else {
                mat.pipeline_type = PipelineType::ColoredModel;
                mat.color = Some(Vec4::new(
                    m.diffuse.unwrap()[0],
                    m.diffuse.unwrap()[1],
                    m.diffuse.unwrap()[2],
                    1.0,
                ));
            }

            materials_out.push(mat);
        }

        // collect all vertices + indices + submeshes
        let mut vertex_data: Vec<ModelVertex> = Vec::new();
        let mut index_data: Vec<u32> = Vec::new();
        let mut submeshes: Vec<Submesh> = Vec::new();

        for m in &models {
            let mesh = &m.mesh;
            let first_index = index_data.len() as i32;

            for idx in &mesh.indices {
                let i = *idx as usize;

                let vx = mesh.positions[i * 3 + 0];
                let vy = mesh.positions[i * 3 + 1];
                let vz = mesh.positions[i * 3 + 2];
                let p = *pre_transform * Vec4::new(vx, vy, vz, 1.0);

                let tx = mesh.texcoords.get(i * 2).cloned().unwrap_or(0.0);
                let ty = mesh.texcoords.get(i * 2 + 1).cloned().unwrap_or(0.0);

                let nx = mesh.normals.get(i * 3).cloned().unwrap_or(0.0);
                let ny = mesh.normals.get(i * 3 + 1).cloned().unwrap_or(0.0);
                let nz = mesh.normals.get(i * 3 + 2).cloned().unwrap_or(0.0);
                let n = (*pre_transform * Vec4::new(nx, ny, nz, 0.0)).normalize();

                vertex_data.push(ModelVertex {
                    position: Vec3::new(p.x, p.y, p.z),
                    tex_coord: Vec2::new(tx, 1.0 - ty),
                    normal: Vec3::new(n.x, n.y, n.z),
                });

                index_data.push(index_data.len() as u32);
            }

            let index_count = (index_data.len() as i32 - first_index) as u32;
            let mat_id = mesh.material_id.unwrap_or(0);

            submeshes.push(Submesh {
                first_index,
                index_count,
                material_id: mat_id,
            });
        }

        // merge vertex + index data into a single buffer
        let bytes_verts: &[u8] = unsafe {
            core::slice::from_raw_parts(
                vertex_data.as_ptr() as *const u8,
                vertex_data.len() * core::mem::size_of::<ModelVertex>(),
            )
        };

        let bytes_idx: &[u8] = unsafe {
            core::slice::from_raw_parts(
                index_data.as_ptr() as *const u8,
                index_data.len() * core::mem::size_of::<u32>(),
            )
        };

        let merged = [bytes_verts, bytes_idx].concat();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model vertex & index buffer"),
            contents: &merged,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
        });

        let ebo_offset = bytes_verts.len() as u64;

        Model {
            buffer,
            ebo_offset,
            submeshes,
        }
    }
}
