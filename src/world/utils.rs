use std::cell::Ref;

use cgmath::Vector3;

use crate::world::{BlockCoords, Cell, Chunk, ChunkCoords, World};

/// Borrows a 3x3 chunk region
pub struct ChunkNeighborhood<'a> {
    chunk: &'a Chunk,
    neighbors: [[Option<Ref<'a, Chunk>>; 3]; 3],
}

impl<'a> ChunkNeighborhood<'a> {
    pub fn new(world: &'a World, chunk: &'a Chunk, chunk_coords: ChunkCoords) -> Self {
        let mut neighbors: [[Option<Ref<Chunk>>; 3]; 3] =
            [[None, None, None], [None, None, None], [None, None, None]];
        for x in 0..3 {
            for y in 0..3 {
                if x != 1 || y != 1 {
                    let offset = ChunkCoords {
                        x: x as i32 - 1,
                        y: y as i32 - 1,
                    };
                    neighbors[x][y] = world.borrow_chunk(chunk_coords + offset);
                }
            }
        }

        ChunkNeighborhood { chunk, neighbors }
    }

    // Coords are relative to middle chunk in chunks array
    pub fn get_cell(&self, coords: Vector3<i32>) -> Option<Cell> {
        if coords.y >= Chunk::SIZE.y || coords.y < 0 {
            return None;
        }

        if coords.x >= 0 && coords.x < Chunk::SIZE.x && coords.z >= 0 && coords.z < Chunk::SIZE.z {
            return Some(self.chunk[coords]);
        }

        // Coords relative to chunks array start
        let relative_x = coords.x + Chunk::SIZE.x;
        let relative_z = coords.z + Chunk::SIZE.z;

        let chunk_x = relative_x / Chunk::SIZE.x;
        let chunk_z = relative_z / Chunk::SIZE.z;

        if let Some(chunk) = &self.neighbors[chunk_x as usize][chunk_z as usize] {
            let block_x = relative_x % Chunk::SIZE.x;
            let block_z = relative_z % Chunk::SIZE.z;

            Some(
                chunk[BlockCoords {
                    x: block_x,
                    y: coords.y,
                    z: block_z,
                }],
            )
        } else {
            None
        }
    }
}
