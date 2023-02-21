pub mod blocks;
pub mod generation;
pub mod mesh;

use std::collections::HashMap;

use cgmath::{Matrix4, Vector2, Vector3};

use crate::{
    context::Context,
    rendering::{chunk_renderer::ChunkGraphics, uniform::Uniform},
    world::{generation::generate_chunk, mesh::generate_chunk_mesh},
};

pub struct Chunk {
    pub blocks: [[[usize; Self::SIZE.z]; Self::SIZE.y]; Self::SIZE.x],
    needs_update: bool,
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
            needs_update: true,
        }
    }
}

pub type ChunkCoords = Vector2<i32>;

pub struct World {
    pub chunks: HashMap<ChunkCoords, Chunk>,
    pub chunk_graphics: HashMap<ChunkCoords, ChunkGraphics>,
}

impl World {
    pub fn new() -> Self {
        World {
            chunks: HashMap::new(),
            chunk_graphics: HashMap::new(),
        }
    }

    pub fn load_chunk(&mut self, coords: ChunkCoords) {
        if self.chunks.contains_key(&coords) {
            return;
        }

        let mut chunk = Chunk::new();
        generate_chunk(&mut chunk, coords);
        self.chunks.insert(coords, chunk);
    }

    pub fn update_chunk_graphics(&mut self, context: &Context) {
        for (coords, chunk) in &self.chunks {
            if chunk.needs_update {
                let translation = Vector3 {
                    x: coords.x as f32 * Chunk::SIZE.x as f32,
                    y: 0.,
                    z: coords.y as f32 * Chunk::SIZE.z as f32,
                };

                let mesh = generate_chunk_mesh(context, self, &chunk, *coords);
                let transform = Uniform::new(
                    context,
                    "Chunk Transform",
                    Matrix4::from_translation(translation),
                );
                self.chunk_graphics.insert(*coords, ChunkGraphics { mesh, transform });
            }
        }
    }
}
