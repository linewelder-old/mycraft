use crate::world::{Chunk, ChunkCoords};

pub fn generate_chunk(chunk: &mut Chunk, _chunk_coords: ChunkCoords) {
    for x in 0..Chunk::SIZE.x {
        for z in 0..Chunk::SIZE.z {
            for y in 0..3 {
                chunk.blocks[x][y][z] = true;
            }
        }
    }
}
