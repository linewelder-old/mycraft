use cgmath::{Vector2, Vector3};

use crate::{
    context::Context,
    rendering::{chunk_renderer::Vertex, vertex_array::VertexArray},
    world::{Chunk, ChunkCoords, World},
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

fn emit_face(vertices: &mut Vec<Vertex>, i: usize, offset: Vector3<f32>) {
    FACES[i]
        .iter()
        .zip(TEX_COORDS)
        .map(|(&pos, tex)| Vertex {
            pos: pos + offset,
            tex,
        })
        .for_each(|x| vertices.push(x));

    // Split into triangles
    vertices.insert(vertices.len() - 1, vertices[vertices.len() - 2]);
    vertices.insert(vertices.len() - 1, vertices[vertices.len() - 4]);
}

pub fn generate_chunk_mesh(
    context: &Context,
    world: &World,
    chunk: &Chunk,
    chunk_coords: ChunkCoords,
) -> VertexArray<Vertex> {
    // Cache neighboring chunks
    let mut chunks: [[Option<&Chunk>; 3]; 3] = [[None; 3]; 3];
    for x in 0..3 {
        for y in 0..3 {
            let offset = ChunkCoords { x: x as i32- 1, y: y as i32 - 1 };
            chunks[x][y] = world.chunks.get(&(chunk_coords + offset));
        }
    }

    // Coords are relative to middle chunk in chunks array
    fn get_block(chunks: &[[Option<&Chunk>; 3]; 3], coords: Vector3<i32>) -> bool {
        if coords.y > Chunk::SIZE.y as i32 || coords.y < 0 {
            return false;
        }

        // Coords relative to chunks array start
        let relative_x = (coords.x + Chunk::SIZE.x as i32) as usize;
        let relative_z = (coords.z + Chunk::SIZE.z as i32) as usize;

        let chunk_x = relative_x / Chunk::SIZE.x;
        let chunk_z = relative_z / Chunk::SIZE.z;
        let chunk = chunks[chunk_x][chunk_z];

        if let Some(chunk) = chunk {
            let block_x = relative_x % Chunk::SIZE.x;
            let block_z = relative_z % Chunk::SIZE.z;

            chunk.blocks[block_x][coords.y as usize][block_z]
        } else {
            false
        }
    }

    let mut vertices = vec![];

    for x in 0..Chunk::SIZE.x as i32 {
        for y in 0..Chunk::SIZE.y as i32 {
            for z in 0..Chunk::SIZE.z as i32 {
                if chunk.blocks[x as usize][y as usize][z as usize] {
                    let block_offset = Vector3 {
                        x: x as f32,
                        y: y as f32,
                        z: z as f32,
                    };

                    for (i, neighbor_offset) in NEIGHBOR_OFFSETS.iter().enumerate() {
                        let neighbor_coords = Vector3 { x, y, z } + neighbor_offset;
                        if !get_block(&chunks, neighbor_coords) {
                            emit_face(&mut vertices, i, block_offset);
                        }
                    }
                }
            }
        }
    }

    VertexArray::new(context, "Chunk Mesh", &vertices)
}
