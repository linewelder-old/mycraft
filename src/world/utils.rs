use std::cell::{Ref, RefMut};

use cgmath::{ElementWise, Vector3};

use super::{BlockCoords, Cell, Chunk, ChunkCoords, World};

fn get_chunk_coords(block_coords: BlockCoords) -> ChunkCoords {
    block_coords.map(|x| x.div_euclid(Chunk::SIZE))
}

pub fn get_chunk_and_block_coords(position: Vector3<f32>) -> (ChunkCoords, BlockCoords) {
    let block_coords = position.map(|x| x.floor() as i32);
    let chunk_coords = get_chunk_coords(block_coords);

    (chunk_coords, block_coords)
}

/// Returns the chunk coords of the given block, and the coords within the chunk
pub fn to_local_chunk_coords(block_coords: BlockCoords) -> (ChunkCoords, BlockCoords) {
    let chunk_coords = get_chunk_coords(block_coords);
    let block_coords = block_coords.map(|x| x.rem_euclid(Chunk::SIZE));

    (chunk_coords, block_coords)
}

pub fn to_chunk_offset(chunk_coords: ChunkCoords) -> Vector3<f32> {
    chunk_coords.mul_element_wise(Chunk::SIZE).map(|x| x as f32)
}

/// Borrows a 3x3x3 chunk region
pub struct ChunkNeighborhood<'a> {
    chunk: &'a Chunk,
    neighbors: [[[Option<Ref<'a, Chunk>>; 3]; 3]; 3],
}

const fn empty_neighbor_array<T>() -> [[[Option<T>; 3]; 3]; 3] {
    [
        [[None, None, None], [None, None, None], [None, None, None]],
        [[None, None, None], [None, None, None], [None, None, None]],
        [[None, None, None], [None, None, None], [None, None, None]],
    ]
}

fn borrow_neighborhood<'a, T: 'a>(
    world: &'a World,
    chunk_coords: ChunkCoords,
    f: impl Fn(&'a World, ChunkCoords) -> Option<T>,
) -> [[[Option<T>; 3]; 3]; 3] {
    let mut neighbors = empty_neighbor_array();
    #[allow(clippy::needless_range_loop)]
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                if x != 1 || y != 1 || z != 1 {
                    let offset = ChunkCoords {
                        x: x as i32 - 1,
                        y: y as i32 - 1,
                        z: z as i32 - 1,
                    };
                    neighbors[x][y][z] = f(world, chunk_coords + offset);
                }
            }
        }
    }

    neighbors
}

impl<'a> ChunkNeighborhood<'a> {
    pub fn new(world: &'a World, chunk: &'a Chunk, chunk_coords: ChunkCoords) -> Self {
        let neighbors = borrow_neighborhood(world, chunk_coords, World::borrow_chunk);
        ChunkNeighborhood { chunk, neighbors }
    }

    // Coords are relative to middle chunk in chunks array
    pub fn get_cell(&self, coords: Vector3<i32>) -> Option<Cell> {
        if (coords.x >= 0 && coords.x < Chunk::SIZE)
            && (coords.y >= 0 && coords.y < Chunk::SIZE)
            && (coords.z >= 0 && coords.z < Chunk::SIZE)
        {
            return Some(self.chunk[coords]);
        }

        let (chunk_coords, block_coords) = to_local_chunk_coords(coords);
        let array_coords = chunk_coords.map(|x| (x + 1) as usize);
        self.neighbors[array_coords.x][array_coords.y][array_coords.z]
            .as_ref()
            .map(|chunk| chunk[block_coords])
    }
}

/// Borrows a 3x3x3 chunk region mutably
pub struct ChunkNeighborhoodMut<'a> {
    chunk: &'a mut Chunk,
    neighbors: [[[Option<RefMut<'a, Chunk>>; 3]; 3]; 3],
}

impl<'a> ChunkNeighborhoodMut<'a> {
    pub fn new(world: &'a World, chunk: &'a mut Chunk, chunk_coords: ChunkCoords) -> Self {
        let neighbors = borrow_neighborhood(world, chunk_coords, World::borrow_mut_chunk);
        ChunkNeighborhoodMut { chunk, neighbors }
    }

    pub fn get_cell(&self, coords: Vector3<i32>) -> Option<Cell> {
        if (coords.x >= 0 && coords.x < Chunk::SIZE)
            && (coords.y >= 0 && coords.y < Chunk::SIZE)
            && (coords.z >= 0 && coords.z < Chunk::SIZE)
        {
            return Some(self.chunk[coords]);
        }

        let (chunk_coords, block_coords) = to_local_chunk_coords(coords);
        let array_coords = chunk_coords.map(|x| (x + 1) as usize);
        self.neighbors[array_coords.x][array_coords.y][array_coords.z]
            .as_ref()
            .map(|chunk| chunk[block_coords])
    }

    pub fn get_cell_mut(&mut self, coords: Vector3<i32>) -> Option<&mut Cell> {
        if (coords.x >= 0 && coords.x < Chunk::SIZE)
            && (coords.y >= 0 && coords.y < Chunk::SIZE)
            && (coords.z >= 0 && coords.z < Chunk::SIZE)
        {
            return Some(&mut self.chunk[coords]);
        }

        let (chunk_coords, block_coords) = to_local_chunk_coords(coords);
        let array_coords = chunk_coords.map(|x| (x + 1) as usize);
        self.neighbors[array_coords.x][array_coords.y][array_coords.z]
            .as_mut()
            .map(|chunk| &mut chunk[block_coords])
    }
}
