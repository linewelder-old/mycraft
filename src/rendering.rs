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

use crate::context::Context;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex(u32, u32);

pub struct VertexDesc {
    pub pos: Vector3<u16>,
    pub texture_id: u16,
    pub texture_coords: Vector2<u8>,
    pub diffused_light: u8,
    pub sun_light: u8,
    pub block_light: u8,
}

impl Vertex {
    const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Uint32x2],
    };

    #[inline]
    pub fn new(desc: VertexDesc) -> Self {
        Vertex(
            ((desc.pos.x as u32) & 0x1FF)
                | (((desc.pos.y as u32) & 0x1FF) << 9)
                | (((desc.pos.z as u32) & 0x1FF) << 18)
                | (((desc.texture_coords.x as u32) & 0x1F) << 27),
            ((desc.texture_coords.y as u32) & 0x1F)
                | ((desc.texture_id as u32) << 5)
                | (((desc.sun_light as u32) & 0xF) << 21)
                | (((desc.block_light as u32) & 0xF) << 25)
                | (((desc.diffused_light as u32) & 0x3) << 29),
        )
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

pub trait Bindable {
    fn create_bind_group_layout(context: &Context) -> wgpu::BindGroupLayout;
    fn get_bind_group(&self) -> &wgpu::BindGroup;
}

#[derive(Clone, Copy)]
pub struct RenderTargetWithDepth<'a> {
    pub color: &'a wgpu::TextureView,
    pub depth: &'a wgpu::TextureView,
}
