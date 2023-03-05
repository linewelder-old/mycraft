use std::cell::Ref;

use cgmath::{Vector2, Vector3, Zero};

use crate::{
    rendering::{Face, Vertex},
    world::{blocks::Block, BlockCoords, Cell, Chunk, ChunkCoords, World},
};

#[rustfmt::skip]
const SOLID_BLOCK_FACES: [[Vector3<f32>; 4]; 6] = [
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
const FLUID_BLOCK_FACES: [[Vector3<f32>; 4]; 6] = [
    // Neg Z
    [
        Vector3 { x: 1., y: 0.,      z: 0. },
        Vector3 { x: 0., y: 0.,      z: 0. },
        Vector3 { x: 1., y: 14./16., z: 0. },
        Vector3 { x: 0., y: 14./16., z: 0. },
    ],
    // Pos Z
    [
        Vector3 { x: 0., y: 0.,      z: 1. },
        Vector3 { x: 1., y: 0.,      z: 1. },
        Vector3 { x: 0., y: 14./16., z: 1. },
        Vector3 { x: 1., y: 14./16., z: 1. },
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
        Vector3 { x: 1., y: 14./16., z: 0. },
        Vector3 { x: 0., y: 14./16., z: 0. },
        Vector3 { x: 1., y: 14./16., z: 1. },
        Vector3 { x: 0., y: 14./16., z: 1. },
    ],
    // Neg X
    [
        Vector3 { x: 0., y: 0.,      z: 0. },
        Vector3 { x: 0., y: 0.,      z: 1. },
        Vector3 { x: 0., y: 14./16., z: 0. },
        Vector3 { x: 0., y: 14./16., z: 1. },
    ],
    // Pos X
    [
        Vector3 { x: 1., y: 0.,      z: 1. },
        Vector3 { x: 1., y: 0.,      z: 0. },
        Vector3 { x: 1., y: 14./16., z: 1. },
        Vector3 { x: 1., y: 14./16., z: 0. },
    ],
];

#[rustfmt::skip]
const FLOWER_BLOCK_FACES: [[Vector3<f32>; 4]; 4] = [
    [
        Vector3 { x: 14./16., y: 0., z: 2./16.  },
        Vector3 { x: 2./16.,  y: 0., z: 14./16. },
        Vector3 { x: 14./16., y: 1., z: 2./16.  },
        Vector3 { x: 2./16.,  y: 1., z: 14./16. },
    ],
    [
        Vector3 { x: 2./16.,  y: 0., z: 2./16.  },
        Vector3 { x: 14./16., y: 0., z: 14./16. },
        Vector3 { x: 2./16.,  y: 1., z: 2./16.  },
        Vector3 { x: 14./16., y: 1., z: 14./16. },
    ],
    [
        Vector3 { x: 2./16.,  y: 0., z: 14./16. },
        Vector3 { x: 14./16., y: 0., z: 2./16.  },
        Vector3 { x: 2./16.,  y: 1., z: 14./16. },
        Vector3 { x: 14./16., y: 1., z: 2./16.  },
    ],
    [
        Vector3 { x: 14./16., y: 0., z: 14./16. },
        Vector3 { x: 2./16.,  y: 0., z: 2./16.  },
        Vector3 { x: 14./16., y: 1., z: 14./16. },
        Vector3 { x: 2./16.,  y: 1., z: 2./16.  },
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

const FACE_LIGHTING: [f32; 6] = [0.6, 0.6, 0.4, 1., 0.8, 0.8];

const TEX_COORDS: [Vector2<f32>; 4] = [
    Vector2 { x: 0., y: 1. },
    Vector2 { x: 1., y: 1. },
    Vector2 { x: 0., y: 0. },
    Vector2 { x: 1., y: 0. },
];

pub struct ChunkMeshes {
    pub solid_vertices: Vec<Vertex>,
    pub water_vertices: Vec<Vertex>,
    pub water_faces: Vec<Face>,
}

struct MeshGenerationContext<'a> {
    chunk: &'a Chunk,
    chunk_offset: Vector3<f32>,
    neighbor_chunks: [[Option<Ref<'a, Chunk>>; 3]; 3],
    current_block_coords: BlockCoords,
    meshes: ChunkMeshes,
}

struct FaceDesc<'a> {
    points: &'a [Vector3<f32>; 4],
    texture_id: u32,
    light: f32,
}

impl<'a> MeshGenerationContext<'a> {
    fn new(world: &'a World, chunk: &'a Chunk, chunk_coords: ChunkCoords) -> Self {
        // Cache neighboring chunks
        let mut neighbor_chunks: [[Option<Ref<Chunk>>; 3]; 3] =
            [[None, None, None], [None, None, None], [None, None, None]];
        for x in 0..3 {
            for y in 0..3 {
                if x != 1 || y != 1 {
                    let offset = ChunkCoords {
                        x: x as i32 - 1,
                        y: y as i32 - 1,
                    };
                    neighbor_chunks[x][y] = world.borrow_chunk(chunk_coords + offset);
                }
            }
        }

        let chunk_offset = Vector3 {
            x: (chunk_coords.x * Chunk::SIZE.x as i32) as f32,
            y: 0.,
            z: (chunk_coords.y * Chunk::SIZE.z as i32) as f32,
        };

        MeshGenerationContext {
            chunk,
            chunk_offset,
            neighbor_chunks,
            current_block_coords: BlockCoords::zero(),
            meshes: ChunkMeshes {
                solid_vertices: vec![],
                water_vertices: vec![],
                water_faces: vec![],
            },
        }
    }

    // Coords are relative to middle chunk in chunks array
    fn get_cell(&self, coords: Vector3<i32>) -> Option<Cell> {
        if coords.y >= Chunk::SIZE.y as i32 || coords.y < 0 {
            return None;
        }

        if coords.x >= 0
            && coords.x < Chunk::SIZE.x as i32
            && coords.z >= 0
            && coords.z < Chunk::SIZE.z as i32
        {
            return Some(self.chunk.data[coords.x as usize][coords.y as usize][coords.z as usize]);
        }

        // Coords relative to chunks array start
        let relative_x = (coords.x + Chunk::SIZE.x as i32) as usize;
        let relative_z = (coords.z + Chunk::SIZE.z as i32) as usize;

        let chunk_x = relative_x / Chunk::SIZE.x;
        let chunk_z = relative_z / Chunk::SIZE.z;

        if let Some(chunk) = &self.neighbor_chunks[chunk_x][chunk_z] {
            let block_x = relative_x % Chunk::SIZE.x;
            let block_z = relative_z % Chunk::SIZE.z;

            Some(chunk.data[block_x][coords.y as usize][block_z])
        } else {
            None
        }
    }

    fn is_transparent(&self, coords: Vector3<i32>) -> bool {
        if let Some(cell) = self.get_cell(coords) {
            cell.get_block().is_transparent()
        } else {
            true
        }
    }

    fn emit_face_vertices(
        vertex_array: &mut Vec<Vertex>,
        chunk_offset: Vector3<f32>,
        block_coords: BlockCoords,
        desc: FaceDesc,
    ) {
        let offset = chunk_offset + block_coords.map(|x| x as f32);
        let base_texture_coords = Vector2 {
            x: (desc.texture_id % 4) as f32,
            y: (desc.texture_id / 4) as f32,
        };

        desc.points
            .iter()
            .zip(TEX_COORDS)
            .map(|(&pos, tex)| Vertex {
                pos: pos + offset,
                tex: (base_texture_coords + tex) / 4.,
                light: desc.light,
            })
            .for_each(|x| vertex_array.push(x));
    }

    fn emit_solid_face(&mut self, desc: FaceDesc) {
        Self::emit_face_vertices(
            &mut self.meshes.solid_vertices,
            self.chunk_offset,
            self.current_block_coords,
            desc,
        );
    }

    fn emit_water_face(&mut self, desc: FaceDesc) {
        let offset = self.current_block_coords.map(|x| x as f32);
        self.meshes.water_faces.push(Face {
            base_index: self.meshes.water_vertices.len() as u32,
            center: offset + desc.points.iter().sum::<Vector3<f32>>() / 4.,
            distance: 0.,
        });

        Self::emit_face_vertices(
            &mut self.meshes.water_vertices,
            self.chunk_offset,
            self.current_block_coords,
            desc,
        );
    }

    fn emit_solid_block(&mut self, texture_ids: &[u32; 6]) {
        for (i, neighbor_offset) in NEIGHBOR_OFFSETS.iter().enumerate() {
            let neighbor_coords = self.current_block_coords + neighbor_offset;
            if self.is_transparent(neighbor_coords) {
                self.emit_solid_face(FaceDesc {
                    points: &SOLID_BLOCK_FACES[i],
                    texture_id: texture_ids[i],
                    light: FACE_LIGHTING[i],
                });
            }
        }
    }

    fn emit_water_block(&mut self, texture_id: u32) {
        const TOP_NEIGHBOR_OFFSET_INDEX: usize = 3;

        let top_neighbor =
            self.get_cell(self.current_block_coords + NEIGHBOR_OFFSETS[TOP_NEIGHBOR_OFFSET_INDEX]);
        let top_neighbor_block = top_neighbor.as_ref().map(Cell::get_block);
        let top_neighbor_is_fluid = matches!(top_neighbor_block, Some(Block::Fluid { .. }));
        let model = if top_neighbor_is_fluid {
            &SOLID_BLOCK_FACES
        } else {
            &FLUID_BLOCK_FACES
        };

        for (i, neighbor_offset) in NEIGHBOR_OFFSETS.iter().enumerate() {
            let neighbor_coords = self.current_block_coords + neighbor_offset;
            if let Some(neighbor_cell) = self.get_cell(neighbor_coords) {
                let neighbor_block = neighbor_cell.get_block();
                let checking_top_neighbor = i == TOP_NEIGHBOR_OFFSET_INDEX;
                let should_not_emit_face = if checking_top_neighbor {
                    top_neighbor_is_fluid
                } else {
                    matches!(neighbor_block, Block::Fluid { .. })
                        || !self.is_transparent(neighbor_coords)
                };

                if should_not_emit_face {
                    continue;
                }
            }

            self.emit_water_face(FaceDesc {
                points: &model[i],
                texture_id,
                light: FACE_LIGHTING[i],
            });
        }
    }

    fn emit_flower_block(&mut self, texture_id: u32) {
        for points in &FLOWER_BLOCK_FACES {
            self.emit_solid_face(FaceDesc {
                points,
                texture_id,
                light: 1.,
            });
        }
    }
}

impl ChunkMeshes {
    pub fn generate(world: &World, chunk: &Chunk, chunk_coords: ChunkCoords) -> Self {
        let mut generation_context = MeshGenerationContext::new(world, chunk, chunk_coords);

        for x in 0..Chunk::SIZE.x as i32 {
            for y in 0..Chunk::SIZE.y as i32 {
                for z in 0..Chunk::SIZE.z as i32 {
                    let current_cell = chunk.data[x as usize][y as usize][z as usize];
                    generation_context.current_block_coords = BlockCoords { x, y, z };

                    match current_cell.get_block() {
                        Block::Empty => {}
                        Block::Solid { texture_ids } => {
                            generation_context.emit_solid_block(texture_ids);
                        }
                        Block::Fluid { texture_id } => {
                            generation_context.emit_water_block(*texture_id);
                        }
                        Block::Flower { texture_id } => {
                            generation_context.emit_flower_block(*texture_id);
                        }
                    }
                }
            }
        }

        generation_context.meshes
    }
}
