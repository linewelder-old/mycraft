pub mod chunk_mesh;
pub mod frustrum;
mod solid_block_pipeline;
pub mod texture;
pub mod uniform;
mod water_pipeline;
pub mod world_renderer;

use std::{cell::RefCell, rc::Rc};

use cgmath::{MetricSpace, Vector2, Vector3};

use self::{chunk_mesh::ChunkMesh, frustrum::Frustrum};
use crate::{
    utils::aabb::AABB,
    world::{Chunk, ChunkCoords},
};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub tex: Vector2<f32>,
    pub light: f32,
}

impl Vertex {
    const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32],
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

pub struct ChunkGraphicsData {
    pub water_faces: Vec<Face>,
    pub outdated: bool,
    pub water_faces_unsorted: bool,
}

pub struct ChunkGraphics {
    pub solid_mesh: ChunkMesh,
    pub water_mesh: ChunkMesh,

    pub graphics_data: RefCell<ChunkGraphicsData>,
}

impl ChunkGraphics {
    pub fn needs_water_faces_sorting(&self) -> bool {
        let data = self.graphics_data.borrow();
        data.water_faces_unsorted
    }

    pub fn sort_water_faces(&self, relative_cam_pos: Vector3<f32>) {
        let mut data = self.graphics_data.borrow_mut();
        data.water_faces_unsorted = false;

        for face in data.water_faces.iter_mut() {
            face.distance = relative_cam_pos.distance2(face.center);
        }

        data.water_faces
            .sort_by(|x, y| y.distance.total_cmp(&x.distance));
        self.water_mesh
            .write_indices(&Face::generate_indices(&data.water_faces));
    }
}

struct RenderQueueItem {
    coords: ChunkCoords,
    graphics: Rc<ChunkGraphics>,
    in_frustrum: bool,
}

pub struct RenderQueue {
    queue: Vec<RenderQueueItem>,
    outdated: bool,
}

fn chunk_aabb(coords: ChunkCoords) -> AABB {
    AABB {
        start: Vector3 {
            x: (coords.x * Chunk::SIZE.x) as f32,
            y: 0.,
            z: (coords.y * Chunk::SIZE.z) as f32,
        },
        size: Chunk::SIZE.map(|x| x as f32),
    }
}

impl RenderQueue {
    pub fn new() -> RenderQueue {
        RenderQueue {
            queue: vec![],
            outdated: false,
        }
    }

    pub fn load_from_iter<'a>(
        &mut self,
        iter: impl Iterator<Item = (ChunkCoords, Rc<ChunkGraphics>)>,
    ) {
        self.queue.clear();
        iter.map(|(coords, graphics)| RenderQueueItem {
            coords,
            graphics,
            in_frustrum: false,
        })
        .for_each(|x| self.queue.push(x));
    }

    pub fn mark_outdated(&mut self) {
        self.outdated = true;
    }

    pub fn is_outdated(&self) -> bool {
        self.outdated
    }

    pub fn clip_to_frustrum(&mut self, frustrum: &Frustrum) {
        for item in &mut self.queue {
            item.in_frustrum = frustrum.intersects_with_aabb(&chunk_aabb(item.coords));
        }
    }

    pub fn iter_for_render(&self) -> impl Iterator<Item = &ChunkGraphics> + Clone {
        self.queue.iter().filter_map(|x| {
            if x.in_frustrum {
                Some(x.graphics.as_ref())
            } else {
                None
            }
        })
    }

    pub fn iter_for_update(&self) -> impl Iterator<Item = (ChunkCoords, &ChunkGraphics)> {
        self.queue.iter().map(|x| (x.coords, x.graphics.as_ref()))
    }
}
