use cgmath::{Vector2, Vector3};

use crate::{
    context::Context,
    rendering::{block_renderer::Vertex, vertex_array::VertexArray},
    world::Chunk,
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

pub fn generate_chunk_mesh(context: &Context, chunk: &Chunk) -> VertexArray<Vertex> {
    let mut vertices = vec![];

    for x in 0..Chunk::SIZE.x {
        for y in 0..Chunk::SIZE.y {
            for z in 0..Chunk::SIZE.z {
                if chunk.blocks[x][y][z] {
                    let offset = Vector3 {
                        x: x as f32,
                        y: y as f32,
                        z: z as f32,
                    };

                    if z == 0 || !chunk.blocks[x][y][z - 1] {
                        emit_face(&mut vertices, 0, offset);
                    }
                    if z == Chunk::SIZE.z - 1 || !chunk.blocks[x][y][z + 1] {
                        emit_face(&mut vertices, 1, offset);
                    }
                    if y == 0 || !chunk.blocks[x][y - 1][z] {
                        emit_face(&mut vertices, 2, offset);
                    }
                    if y == Chunk::SIZE.y - 1 || !chunk.blocks[x][y + 1][z] {
                        emit_face(&mut vertices, 3, offset);
                    }
                    if x == 0 || !chunk.blocks[x - 1][y][z] {
                        emit_face(&mut vertices, 4, offset);
                    }
                    if x == Chunk::SIZE.x - 1 || !chunk.blocks[x + 1][y][z] {
                        emit_face(&mut vertices, 5, offset);
                    }
                }
            }
        }
    }

    VertexArray::new(context, "Chunk Mesh", &vertices)
}
