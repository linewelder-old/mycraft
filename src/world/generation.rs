use cgmath::Vector2;
use noise::{NoiseFn, Perlin};

use crate::world::{blocks::BLOCKS, Chunk, ChunkCoords};

pub fn generate_chunk(chunk: &mut Chunk, chunk_coords: ChunkCoords) {
    let noise = Perlin::new(0);

    for x in 0..Chunk::SIZE.x {
        for z in 0..Chunk::SIZE.z {
            let offset = Vector2 {
                x: (x as i32 + chunk_coords.x * Chunk::SIZE.x as i32) as f64,
                y: (z as i32 + chunk_coords.y * Chunk::SIZE.z as i32) as f64,
            };

            let height = noise.get((offset / 20.).into()) as f32 * 3. + 10.;
            let height = height as usize;

            for y in 0..(height - 3) {
                chunk.blocks[x][y][z] = 1;
            }

            for y in (height - 3)..height {
                chunk.blocks[x][y][z] = 3;
            }

            chunk.blocks[x][height][z] = 2;
        }
    }

    if chunk_coords == (ChunkCoords { x: 0, y: 0 }) {
        for id in 0..BLOCKS.len() {
            chunk.blocks[0][15 + 2 * id][0] = id;
        }
    }
}
