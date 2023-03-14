use std::cell::{Ref, RefMut};

use cgmath::Vector3;

use super::{BlockCoords, Cell, Chunk, ChunkCoords, World};

fn get_chunk_coords(block_coords: BlockCoords) -> ChunkCoords {
    ChunkCoords {
        x: block_coords.x.div_euclid(Chunk::SIZE.x),
        y: block_coords.z.div_euclid(Chunk::SIZE.z),
    }
}

pub fn get_chunk_and_block_coords(position: Vector3<f32>) -> (ChunkCoords, BlockCoords) {
    let block_coords = position.map(|x| x.floor() as i32);
    let chunk_coords = get_chunk_coords(block_coords);

    (chunk_coords, block_coords)
}

/// Returns the chunk coords of the given block, and the coords within the chunk
pub fn to_local_chunk_coords(block_coords: BlockCoords) -> (ChunkCoords, BlockCoords) {
    let chunk_coords = get_chunk_coords(block_coords);
    let block_coords = BlockCoords {
        x: block_coords.x.rem_euclid(Chunk::SIZE.x),
        y: block_coords.y,
        z: block_coords.z.rem_euclid(Chunk::SIZE.z),
    };

    (chunk_coords, block_coords)
}

/// Borrows a 3x3 chunk region
pub struct ChunkNeighborhood<'a> {
    chunk: &'a Chunk,
    neighbors: [[Option<Ref<'a, Chunk>>; 3]; 3],
}

impl<'a> ChunkNeighborhood<'a> {
    pub fn new(world: &'a World, chunk: &'a Chunk, chunk_coords: ChunkCoords) -> Self {
        let mut neighbors = [[None, None, None], [None, None, None], [None, None, None]];
        #[allow(clippy::needless_range_loop)]
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

        let (chunk_coords, block_coords) = to_local_chunk_coords(coords);
        self.neighbors[(chunk_coords.x + 1) as usize][(chunk_coords.y + 1) as usize]
            .as_ref()
            .map(|chunk| chunk[block_coords])
    }
}

/// Borrows a 3x3 chunk region mutably
pub struct ChunkNeighborhoodMut<'a> {
    chunk: &'a mut Chunk,
    neighbors: [[Option<RefMut<'a, Chunk>>; 3]; 3],
}

impl<'a> ChunkNeighborhoodMut<'a> {
    pub fn new(world: &'a World, chunk: &'a mut Chunk, chunk_coords: ChunkCoords) -> Self {
        let mut neighbors = [[None, None, None], [None, None, None], [None, None, None]];
        #[allow(clippy::needless_range_loop)]
        for x in 0..3 {
            for y in 0..3 {
                if x != 1 || y != 1 {
                    let offset = ChunkCoords {
                        x: x as i32 - 1,
                        y: y as i32 - 1,
                    };
                    neighbors[x][y] = world.borrow_mut_chunk(chunk_coords + offset);
                }
            }
        }

        ChunkNeighborhoodMut { chunk, neighbors }
    }

    pub fn get_cell(&self, coords: Vector3<i32>) -> Option<Cell> {
        if coords.y >= Chunk::SIZE.y || coords.y < 0 {
            return None;
        }

        if coords.x >= 0 && coords.x < Chunk::SIZE.x && coords.z >= 0 && coords.z < Chunk::SIZE.z {
            return Some(self.chunk[coords]);
        }

        let (chunk_coords, block_coords) = to_local_chunk_coords(coords);
        self.neighbors[(chunk_coords.x + 1) as usize][(chunk_coords.y + 1) as usize]
            .as_ref()
            .map(|chunk| chunk[block_coords])
    }

    pub fn get_cell_mut(&mut self, coords: Vector3<i32>) -> Option<&mut Cell> {
        if coords.y >= Chunk::SIZE.y || coords.y < 0 {
            return None;
        }

        if coords.x >= 0 && coords.x < Chunk::SIZE.x && coords.z >= 0 && coords.z < Chunk::SIZE.z {
            return Some(&mut self.chunk[coords]);
        }

        let (chunk_coords, block_coords) = to_local_chunk_coords(coords);
        self.neighbors[(chunk_coords.x + 1) as usize][(chunk_coords.y + 1) as usize]
            .as_mut()
            .map(|chunk| &mut chunk[block_coords])
    }
}
