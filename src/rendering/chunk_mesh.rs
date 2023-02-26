use wgpu::util::DeviceExt;

use crate::{context::Context, rendering::Vertex, utils::as_bytes_slice};

pub struct ChunkMesh {
    pub vertices: wgpu::Buffer,
    pub vertex_count: u32,
}

impl ChunkMesh {
    pub fn new(context: &Context, label: &str, vertices: &[Vertex]) -> Self {
        ChunkMesh {
            vertices: context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    usage: wgpu::BufferUsages::VERTEX,
                    contents: as_bytes_slice(vertices),
                }),
            vertex_count: vertices.len() as u32,
        }
    }
}
