pub mod solid_block_renderer;
pub mod texture;
pub mod uniform;
pub mod chunk_mesh;
pub mod water_renderer;

use std::{cmp::Reverse, rc::Rc};

use cgmath::{Matrix4, MetricSpace, Vector2, Vector3};

use crate::{
    rendering::{uniform::Uniform, chunk_mesh::ChunkMesh},
    world::{ChunkCoords, World},
};

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
    pub solid_mesh: ChunkMesh,
    pub water_mesh: ChunkMesh,
    pub transform: Uniform<Matrix4<f32>>,
}

pub struct RenderQueue(Vec<(ChunkCoords, Rc<ChunkGraphics>)>);

impl RenderQueue {
    pub fn new(cam_chunk_coords: ChunkCoords, world: &World) -> RenderQueue {
        let mut queue = RenderQueue(
            world
                .chunk_graphics
                .iter()
                .map(|(coords, graphics)| (*coords, graphics.clone()))
                .collect(),
        );
        queue.sort(cam_chunk_coords);
        queue
    }

    pub fn sort(&mut self, cam_chunk_coords: ChunkCoords) {
        self.0
            .sort_unstable_by_key(|x| Reverse(cam_chunk_coords.distance2(x.0)));
    }

    pub fn iter(&self) -> impl Iterator<Item = &ChunkGraphics> {
        self.0.iter().map(|x| x.1.as_ref())
    }
}
