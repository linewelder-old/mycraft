pub mod solid_block_renderer;
pub mod texture;
pub mod uniform;
pub mod vertex_array;

use cgmath::{Matrix4, Vector2, Vector3};

use crate::rendering::{uniform::Uniform, vertex_array::VertexArray};

#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub tex: Vector2<f32>,
    pub normal: Vector3<f32>,
}

impl Vertex {
    const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3],
    };
}

pub struct ChunkRendererTarget<'a> {
    pub output: &'a wgpu::TextureView,
    pub depth_buffer: &'a wgpu::TextureView,
}

pub struct ChunkGraphics {
    pub solid_mesh: VertexArray<Vertex>,
    pub transform: Uniform<Matrix4<f32>>,
}
