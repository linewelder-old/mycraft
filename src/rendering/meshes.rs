use std::rc::Rc;

use cgmath::Vector3;
use wgpu::util::DeviceExt;

use super::Vertex;
use crate::{context::Context, utils::as_bytes_slice};

pub struct ChunkMesh {
    context: Rc<Context>,
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub index_count: u32,
}

impl ChunkMesh {
    pub fn new(context: Rc<Context>, label: &str, vertices: &[Vertex], indices: &[u32]) -> Self {
        let index_count = indices.len() as u32;
        let vertices = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Vertices", label)),
                usage: wgpu::BufferUsages::VERTEX,
                contents: as_bytes_slice(vertices),
            });
        let indices = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Indices", label)),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                contents: as_bytes_slice(indices),
            });

        ChunkMesh {
            context,
            vertices,
            indices,
            index_count,
        }
    }

    pub fn write_indices(&self, indices: &[u32]) {
        assert_eq!(indices.len() as u32, self.index_count);
        self.context
            .queue
            .write_buffer(&self.indices, 0, as_bytes_slice(indices));
    }
}
