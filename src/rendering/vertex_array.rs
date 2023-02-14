use std::marker::PhantomData;

use wgpu::util::DeviceExt;

use crate::{context::Context, utils::as_bytes_slice};

pub struct VertexArray<T> {
    pub vertices: wgpu::Buffer,
    pub vertex_count: u32,
    phantom: PhantomData<T>,
}

impl<T> VertexArray<T> {
    pub fn new(context: &Context, label: &str, vertices: &[T]) -> Self {
        VertexArray {
            vertices: context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    usage: wgpu::BufferUsages::VERTEX,
                    contents: as_bytes_slice(vertices),
                }),
            vertex_count: vertices.len() as u32,
            phantom: PhantomData::default(),
        }
    }
}
