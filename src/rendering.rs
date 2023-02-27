pub mod chunk_mesh;
pub mod solid_block_renderer;
pub mod texture;
pub mod uniform;
pub mod water_renderer;

use std::{cell::RefCell, cmp::Reverse, rc::Rc};

use cgmath::{Matrix4, MetricSpace, Vector2, Vector3};

use crate::{
    context::Context,
    rendering::{chunk_mesh::ChunkMesh, uniform::Uniform},
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

pub struct Face {
    pub base_index: u32,
    pub center: Vector3<f32>,
    pub distance: f32,
}

impl Face {
    const VERTEX_INDICES: [u32; 6] = [0, 1, 2, 2, 1, 3];

    pub fn generate_default_indices(face_count: usize) -> Vec<u32> {
        Self::VERTEX_INDICES
            .iter()
            .cycle()
            .enumerate()
            .map(|(i, x)| x + (i as u32 / 6) * 4)
            .take(face_count * 6)
            .collect()
    }

    pub fn generate_indices(faces: &[Face]) -> Vec<u32> {
        faces
            .iter()
            .flat_map(|face| Self::VERTEX_INDICES.iter().map(|x| x + face.base_index))
            .collect()
    }
}

pub struct ChunkRendererTarget<'a> {
    pub output: &'a wgpu::TextureView,
    pub depth_buffer: &'a wgpu::TextureView,
}

pub struct ChunkGraphics {
    pub solid_mesh: ChunkMesh,
    pub water_mesh: ChunkMesh,
    pub transform: Uniform<Matrix4<f32>>,

    pub water_faces: RefCell<Vec<Face>>,
}

impl ChunkGraphics {
    pub fn sort_water_geometry(&self, context: &mut Context, relative_cam_pos: Vector3<f32>) {
        let mut water_faces = self.water_faces.borrow_mut();
        for face in water_faces.iter_mut() {
            face.distance = relative_cam_pos.distance2(face.center);
        }

        water_faces.sort_by(|x, y| y.distance.total_cmp(&x.distance));
        self.water_mesh
            .write_indices(context, &Face::generate_indices(&water_faces));
    }
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

    pub fn iter_with_coords(&self) -> impl Iterator<Item = (ChunkCoords, &ChunkGraphics)> {
        self.0.iter().map(|x| (x.0, x.1.as_ref()))
    }
}
