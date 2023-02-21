use cgmath::{Vector2, Vector3};

use crate::{
    context::Context,
    rendering::{chunk_renderer::Vertex, vertex_array::VertexArray},
    world::{
        blocks::{Block, BLOCKS},
        Chunk, ChunkCoords, World,
    },
};

#[rustfmt::skip]
const FACES: [[Vector3<f32>; 4]; 6] = [
    // Neg Z
    [
        Vector3 { x: 1., y: 0., z: 0. },
        Vector3 { x: 0., y: 0., z: 0. },
        Vector3 { x: 1., y: 1., z: 0. },
        Vector3 { x: 0., y: 1., z: 0. },
    ],
    // Pos Z
    [
        Vector3 { x: 0., y: 0., z: 1. },
        Vector3 { x: 1., y: 0., z: 1. },
        Vector3 { x: 0., y: 1., z: 1. },
        Vector3 { x: 1., y: 1., z: 1. },
    ],
    // Neg Y
    [
        Vector3 { x: 0., y: 0., z: 0. },
        Vector3 { x: 1., y: 0., z: 0. },
        Vector3 { x: 0., y: 0., z: 1. },
        Vector3 { x: 1., y: 0., z: 1. },
    ],
    // Pos Y
    [
        Vector3 { x: 1., y: 1., z: 0. },
        Vector3 { x: 0., y: 1., z: 0. },
        Vector3 { x: 1., y: 1., z: 1. },
        Vector3 { x: 0., y: 1., z: 1. },
    ],
    // Neg X
    [
        Vector3 { x: 0., y: 0., z: 0. },
        Vector3 { x: 0., y: 0., z: 1. },
        Vector3 { x: 0., y: 1., z: 0. },
        Vector3 { x: 0., y: 1., z: 1. },
    ],
    // Pos X
    [
        Vector3 { x: 1., y: 0., z: 1. },
        Vector3 { x: 1., y: 0., z: 0. },
        Vector3 { x: 1., y: 1., z: 1. },
        Vector3 { x: 1., y: 1., z: 0. },
    ],
];

#[rustfmt::skip]
const NEIGHBOR_OFFSETS: [Vector3<i32>; 6] = [
    Vector3 { x:  0, y:  0, z: -1 },
    Vector3 { x:  0, y:  0, z:  1 },
    Vector3 { x:  0, y: -1, z:  0 },
    Vector3 { x:  0, y:  1, z:  0 },
    Vector3 { x: -1, y:  0, z:  0 },
    Vector3 { x:  1, y:  0, z:  0 },
];

const TEX_COORDS: [Vector2<f32>; 4] = [
    Vector2 { x: 0., y: 1. },
    Vector2 { x: 1., y: 1. },
    Vector2 { x: 0., y: 0. },
    Vector2 { x: 1., y: 0. },
];

struct MeshGenerationContext<'a> {
    neighbor_chunks: [[Option<&'a Chunk>; 3]; 3],
    vertices: Vec<Vertex>,
}

impl<'a> MeshGenerationContext<'a> {
    fn new(world: &'a World, chunk_coords: ChunkCoords) -> Self {
        // Cache neighboring chunks
        let mut neighbor_chunks: [[Option<&Chunk>; 3]; 3] = [[None; 3]; 3];
        for x in 0..3 {
            for y in 0..3 {
                let offset = ChunkCoords {
                    x: x as i32 - 1,
                    y: y as i32 - 1,
                };
                neighbor_chunks[x][y] = world.chunks.get(&(chunk_coords + offset));
            }
        }

        MeshGenerationContext {
            neighbor_chunks,
            vertices: vec![],
        }
    }

    // Coords are relative to middle chunk in chunks array
    fn is_transparent(&self, coords: Vector3<i32>) -> bool {
        if coords.y > Chunk::SIZE.y as i32 || coords.y < 0 {
            return true;
        }

        // Coords relative to chunks array start
        let relative_x = (coords.x + Chunk::SIZE.x as i32) as usize;
        let relative_z = (coords.z + Chunk::SIZE.z as i32) as usize;

        let chunk_x = relative_x / Chunk::SIZE.x;
        let chunk_z = relative_z / Chunk::SIZE.z;
        let chunk = self.neighbor_chunks[chunk_x][chunk_z];

        if let Some(chunk) = chunk {
            let block_x = relative_x % Chunk::SIZE.x;
            let block_z = relative_z % Chunk::SIZE.z;

            let block_id = chunk.blocks[block_x][coords.y as usize][block_z];
            BLOCKS[block_id].transparent
        } else {
            true
        }
    }

    fn emit_face(&mut self, i: usize, texture_id: u32, offset: Vector3<f32>) {
        let base_texture_coords = Vector2 {
            x: (texture_id % 4) as f32,
            y: (texture_id / 4) as f32,
        };

        FACES[i]
            .iter()
            .zip(TEX_COORDS)
            .map(|(&pos, tex)| Vertex {
                pos: pos + offset,
                tex: (base_texture_coords + tex) / 4.,
                normal: NEIGHBOR_OFFSETS[i].map(|x| x as f32),
            })
            .for_each(|x| self.vertices.push(x));

        // Split into triangles
        self.vertices.insert(
            self.vertices.len() - 1,
            self.vertices[self.vertices.len() - 2],
        );
        self.vertices.insert(
            self.vertices.len() - 1,
            self.vertices[self.vertices.len() - 4],
        );
    }

    fn emit_block(&mut self, x: i32, y: i32, z: i32, block: &Block) {
        if let Some(textures) = &block.texture_ids {
            let block_offset = Vector3 {
                x: x as f32,
                y: y as f32,
                z: z as f32,
            };

            for (i, neighbor_offset) in NEIGHBOR_OFFSETS.iter().enumerate() {
                let neighbor_coords = Vector3 { x, y, z } + neighbor_offset;
                if self.is_transparent(neighbor_coords) {
                    self.emit_face(i, textures[i], block_offset);
                }
            }
        }
    }
}

pub fn generate_chunk_mesh(
    context: &Context,
    world: &World,
    chunk: &Chunk,
    chunk_coords: ChunkCoords,
) -> VertexArray<Vertex> {
    let mut generation_context = MeshGenerationContext::new(world, chunk_coords);

    for x in 0..Chunk::SIZE.x as i32 {
        for y in 0..Chunk::SIZE.y as i32 {
            for z in 0..Chunk::SIZE.z as i32 {
                let block_id = chunk.blocks[x as usize][y as usize][z as usize];
                let block = &BLOCKS[block_id];

                generation_context.emit_block(x, y, z, block);
            }
        }
    }

    VertexArray::new(context, "Chunk Mesh", &generation_context.vertices)
}
