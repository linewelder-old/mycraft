pub mod blocks;
pub mod generation;
pub mod mesh;

use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use cgmath::{Vector2, Vector3, Zero};

use crate::{
    context::Context,
    rendering::{chunk_mesh::ChunkMesh, ChunkGraphics, ChunkGraphicsData, Face, RenderQueue},
    world::{
        blocks::{Block, BlockId},
        generation::Generator,
        mesh::ChunkMeshes,
    },
};

pub type LightLevel = u8;

#[derive(Clone, Copy)]
pub struct Cell {
    pub block_id: BlockId,
    pub light: LightLevel,
}

impl Cell {
    #[inline]
    pub fn get_block(&self) -> &'static Block {
        Block::by_id(self.block_id)
    }
}

pub struct Chunk {
    pub data: [[[Cell; Self::SIZE.z]; Self::SIZE.y]; Self::SIZE.x],
    pub graphics: Option<Rc<ChunkGraphics>>,
}

impl Chunk {
    pub const SIZE: Vector3<usize> = Vector3 {
        x: 16,
        y: 256,
        z: 16,
    };

    pub fn new() -> Self {
        Chunk {
            data: [[[Cell {
                block_id: BlockId::Air,
                light: 0,
            }; Self::SIZE.z]; Self::SIZE.y]; Self::SIZE.x],
            graphics: None,
        }
    }

    pub fn needs_graphics_update(&self) -> bool {
        if let Some(graphics) = &self.graphics {
            graphics.graphics_data.borrow().needs_update
        } else {
            true
        }
    }
}

pub type ChunkCoords = Vector2<i32>;
pub type BlockCoords = Vector3<i32>;

pub struct World {
    pub chunks: HashMap<ChunkCoords, RefCell<Chunk>>,
    generator: Generator,

    render_queue: RenderQueue,
    prev_cam_chunk_coords: ChunkCoords,
    prev_cam_block_coords: BlockCoords,
}

fn get_chunk_block_coords(position: Vector3<f32>) -> (ChunkCoords, BlockCoords) {
    let block_coords = position.map(|x| x.floor() as i32);
    let chunk_coords = World::get_chunk_coords(block_coords);

    (chunk_coords, block_coords)
}

impl World {
    pub fn new() -> Self {
        World {
            chunks: HashMap::new(),
            generator: Generator::new(0),

            render_queue: RenderQueue::new(),
            prev_cam_block_coords: Vector3::zero(),
            prev_cam_chunk_coords: Vector2::zero(),
        }
    }

    pub fn load_chunk(&mut self, coords: ChunkCoords) {
        if self.chunks.contains_key(&coords) {
            return;
        }

        let mut chunk = Chunk::new();
        self.generator.generate_chunk(&mut chunk, coords);
        self.chunks.insert(coords, RefCell::new(chunk));
    }

    fn check_what_is_to_sort(&mut self, camera_position: Vector3<f32>) {
        let (cam_chunk_coords, cam_block_coords) = get_chunk_block_coords(camera_position);
        if cam_chunk_coords != self.prev_cam_chunk_coords {
            self.render_queue.mark_unsorted();
            self.prev_cam_chunk_coords = cam_chunk_coords;
        }

        if cam_block_coords != self.prev_cam_block_coords {
            for (_, graphics) in self.render_queue.iter_for_update() {
                graphics.graphics_data.borrow_mut().water_faces_unsorted = true;
            }
            self.prev_cam_block_coords = cam_block_coords;
        }
    }

    pub fn ensure_water_geometry_is_sorted(
        &mut self,
        context: &mut Context,
        camera_position: Vector3<f32>,
    ) {
        self.check_what_is_to_sort(camera_position);

        self.render_queue.sort_if_needed(self.prev_cam_chunk_coords);
        for (coords, graphics) in self.render_queue.iter_for_update() {
            let chunk_offset = Vector3 {
                x: (coords.x * Chunk::SIZE.x as i32) as f32,
                y: 0.,
                z: (coords.y * Chunk::SIZE.z as i32) as f32,
            };
            let relative_cam_pos = camera_position - chunk_offset;

            if graphics.sort_water_faces_if_needed(context, relative_cam_pos) {
                break;
            }
        }
    }

    pub fn update_chunk_graphics(&mut self, context: &Context) {
        for (coords, chunk) in &self.chunks {
            let mut chunk = chunk.borrow_mut();

            if chunk.needs_graphics_update() {
                let meshes = ChunkMeshes::generate(self, &chunk, *coords);
                let solid_mesh = ChunkMesh::new(
                    context,
                    "Solid Chunk Mesh",
                    &meshes.solid_vertices,
                    &Face::generate_default_indices(meshes.solid_vertices.len() * 4),
                );
                let water_mesh = ChunkMesh::new(
                    context,
                    "Water Chunk Mesh",
                    &meshes.water_vertices,
                    &Face::generate_indices(&meshes.water_faces),
                );

                let graphics = Rc::new(ChunkGraphics {
                    solid_mesh,
                    water_mesh,

                    graphics_data: RefCell::new(ChunkGraphicsData {
                        water_faces: meshes.water_faces,
                        needs_update: false,
                        water_faces_unsorted: true,
                    }),
                });
                chunk.graphics = Some(graphics.clone());
                self.render_queue.insert(*coords, graphics);
            }
        }
    }

    pub fn get_chunk_coords(block_coords: BlockCoords) -> ChunkCoords {
        ChunkCoords {
            x: block_coords.x.div_euclid(Chunk::SIZE.x as i32),
            y: block_coords.z.div_euclid(Chunk::SIZE.z as i32),
        }
    }

    pub fn to_chunk_block_coords(block_coords: BlockCoords) -> (ChunkCoords, BlockCoords) {
        let chunk_coords = Self::get_chunk_coords(block_coords);
        let block_coords = BlockCoords {
            x: block_coords.x.rem_euclid(Chunk::SIZE.x as i32),
            y: block_coords.y,
            z: block_coords.z.rem_euclid(Chunk::SIZE.z as i32),
        };

        (chunk_coords, block_coords)
    }

    pub fn borrow_chunk(&self, coords: ChunkCoords) -> Option<Ref<Chunk>> {
        self.chunks.get(&coords).map(RefCell::borrow)
    }

    pub fn get_block(&self, coords: BlockCoords) -> Option<&'static Block> {
        if coords.y < 0 || coords.y >= Chunk::SIZE.y as i32 {
            return None;
        }

        let (chunk_coords, block_coords) = Self::to_chunk_block_coords(coords);
        self.chunks.get(&chunk_coords).map(|chunk| {
            let chunk = chunk.borrow();
            chunk.data[block_coords.x as usize][block_coords.y as usize][block_coords.z as usize]
                .get_block()
        })
    }

    pub fn set_block(&mut self, coords: BlockCoords, block_id: BlockId) {
        if coords.y < 0 || coords.y >= Chunk::SIZE.y as i32 {
            return;
        }

        let (chunk_coords, block_coords) = Self::to_chunk_block_coords(coords);
        if let Some(chunk) = self.chunks.get_mut(&chunk_coords) {
            let mut chunk = chunk.borrow_mut();

            chunk.data[block_coords.x as usize][block_coords.y as usize][block_coords.z as usize]
                .block_id = block_id;
            if let Some(graphics) = &chunk.graphics {
                graphics.graphics_data.borrow_mut().needs_update = true;
            }
        }
    }

    pub fn render_queue_iter(&self) -> impl Iterator<Item = &ChunkGraphics> {
        self.render_queue.iter_for_render()
    }
}
