use cgmath::{Vector2, Vector3};

use crate::rendering::block_renderer::Vertex;

#[rustfmt::skip]
const FACES: [[Vector3<f32>; 4]; 6] = [
    [
        Vector3 { x: 1., y: 0., z: 0. },
        Vector3 { x: 0., y: 0., z: 0. },
        Vector3 { x: 1., y: 1., z: 0. },
        Vector3 { x: 0., y: 1., z: 0. },
    ],
    [
        Vector3 { x: 0., y: 0., z: 1. },
        Vector3 { x: 1., y: 0., z: 1. },
        Vector3 { x: 0., y: 1., z: 1. },
        Vector3 { x: 1., y: 1., z: 1. },
    ],
    [
        Vector3 { x: 0., y: 0., z: 0. },
        Vector3 { x: 1., y: 0., z: 0. },
        Vector3 { x: 0., y: 0., z: 1. },
        Vector3 { x: 1., y: 0., z: 1. },
    ],
    [
        Vector3 { x: 1., y: 1., z: 0. },
        Vector3 { x: 0., y: 1., z: 0. },
        Vector3 { x: 1., y: 1., z: 1. },
        Vector3 { x: 0., y: 1., z: 1. },
    ],
    [
        Vector3 { x: 0., y: 0., z: 0. },
        Vector3 { x: 0., y: 0., z: 1. },
        Vector3 { x: 0., y: 1., z: 0. },
        Vector3 { x: 0., y: 1., z: 1. },
    ],
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

pub fn create_cube_vertices() -> Vec<Vertex> {
    FACES
        .iter()
        .flat_map(|face| {
            let mut vertices: Vec<Vertex> = face
                .iter()
                .zip(TEX_COORDS)
                .map(|(&pos, tex)| Vertex { pos, tex })
                .collect();

            // Split into triangles
            vertices.insert(3, vertices[2]);
            vertices.insert(4, vertices[1]);
            vertices.into_iter()
        })
        .collect()
}
