pub mod blocks;
pub mod generation;
pub mod mesh;

use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use cgmath::{Matrix4, Vector2, Vector3};

use crate::{
    context::Context,
    rendering::{uniform::Uniform, ChunkGraphics, ChunkGraphicsData, RenderQueue},
    world::{
        blocks::{Block, BLOCKS},
        generation::Generator,
        mesh::ChunkMeshes,
    },
};

pub struct Chunk {
    pub blocks: [[[usize; Self::SIZE.z]; Self::SIZE.y]; Self::SIZE.x],
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
            blocks: [[[0; Self::SIZE.z]; Self::SIZE.y]; Self::SIZE.x],
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
    pub render_queue: RenderQueue,

    generator: Generator,
}

impl World {
    pub fn new() -> Self {
        World {
            chunks: HashMap::new(),
            render_queue: RenderQueue::new(),
            generator: Generator::new(0),
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

    pub fn update_chunk_graphics(&mut self, context: &Context) {
        for (coords, chunk) in &self.chunks {
            let mut chunk = chunk.borrow_mut();

            if chunk.needs_graphics_update() {
                let translation = Vector3 {
                    x: coords.x as f32 * Chunk::SIZE.x as f32,
                    y: 0.,
                    z: coords.y as f32 * Chunk::SIZE.z as f32,
                };

                let meshes = ChunkMeshes::generate(context, self, &chunk, *coords);
                let transform = Uniform::new(
                    context,
                    "Chunk Transform",
                    Matrix4::from_translation(translation),
                );

                let graphics = Rc::new(ChunkGraphics {
                    solid_mesh: meshes.solid_mesh,
                    water_mesh: meshes.water_mesh,
                    transform,

                    graphics_data: RefCell::new(ChunkGraphicsData {
                        water_faces: meshes.water_faces,
                        needs_update: false,
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
        if coords.y < 0 || coords.y > Chunk::SIZE.y as i32 {
            return None;
        }

        let (chunk_coords, block_coords) = Self::to_chunk_block_coords(coords);
        self.chunks.get(&chunk_coords).map(|chunk| {
            let chunk = chunk.borrow();
            let block_id = chunk.blocks[block_coords.x as usize][block_coords.y as usize]
                [block_coords.z as usize];
            &BLOCKS[block_id]
        })
    }

    pub fn set_block(&mut self, coords: BlockCoords, block_id: usize) {
        if coords.y < 0 || coords.y > Chunk::SIZE.y as i32 {
            return;
        }

        let (chunk_coords, block_coords) = Self::to_chunk_block_coords(coords);
        if let Some(chunk) = self.chunks.get_mut(&chunk_coords) {
            let mut chunk = chunk.borrow_mut();

            chunk.blocks[block_coords.x as usize][block_coords.y as usize]
                [block_coords.z as usize] = block_id;
            if let Some(graphics) = &chunk.graphics {
                graphics.graphics_data.borrow_mut().needs_update = true;
            }
        }
    }
}
