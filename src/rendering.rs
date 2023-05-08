pub mod chunk_mesh;
pub mod frustrum;
pub mod sky_renderer;
mod solid_block_pipeline;
pub mod texture;
pub mod uniform;
mod water_pipeline;
pub mod world_renderer;

use std::cell::RefCell;

use cgmath::{MetricSpace, Vector2, Vector3};

use self::{chunk_mesh::ChunkMesh, uniform::Uniform};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pos: Vector3<f32>,
    tex: Vector2<f32>,
    light: u32,
}

pub struct VertexDesc {
    pub pos: Vector3<f32>,
    pub tex: Vector2<f32>,
    pub diffused_light: u8,
    pub sun_light: u8,
    pub block_light: u8,
}

impl Vertex {
    const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Uint32],
    };

    #[inline]
    pub fn new(desc: VertexDesc) -> Self {
        Vertex {
            pos: desc.pos,
            tex: desc.tex,
            light: ((desc.diffused_light as u32) << 8)
                | ((desc.sun_light as u32) << 4)
                | (desc.block_light as u32),
        }
    }
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
    pub water_faces_unsorted: bool,
}

pub struct ChunkGraphics {
    pub solid_mesh: ChunkMesh,
    pub water_mesh: ChunkMesh,
    pub offset: Uniform<Vector3<f32>>,

    pub graphics_data: RefCell<ChunkGraphicsData>,
}

impl ChunkGraphics {
    pub fn needs_water_faces_sorting(&self) -> bool {
        let data = self.graphics_data.borrow();
        data.water_faces_unsorted
    }

    pub fn sort_water_faces(&self, relative_cam_pos: Vector3<f32>) {
        puffin::profile_function!();

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
