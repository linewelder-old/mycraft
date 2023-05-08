use cgmath::{Vector2, Vector3, Zero};

use super::{
    blocks::Block, utils::ChunkNeighborhood, BlockCoords, Cell, Chunk, ChunkCoords, LightLevel,
    World,
};
use crate::rendering::{Face, Vertex, VertexDesc};

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
const TORCH_BLOCK_FACES: [[Vector3<f32>; 4]; 5] = [
    // Pos Y
    [
        Vector3 { x: 9./16., y: 9./16., z: 7./16. },
        Vector3 { x: 7./16., y: 9./16., z: 7./16. },
        Vector3 { x: 9./16., y: 9./16., z: 9./16. },
        Vector3 { x: 7./16., y: 9./16., z: 9./16. },
    ],
    // Neg Z
    [
        Vector3 { x: 1., y: 0., z: 7./16. },
        Vector3 { x: 0., y: 0., z: 7./16. },
        Vector3 { x: 1., y: 1., z: 7./16. },
        Vector3 { x: 0., y: 1., z: 7./16. },
    ],
    // Pos Z
    [
        Vector3 { x: 0., y: 0., z: 9./16. },
        Vector3 { x: 1., y: 0., z: 9./16. },
        Vector3 { x: 0., y: 1., z: 9./16. },
        Vector3 { x: 1., y: 1., z: 9./16. },
    ],
    // Neg X
    [
        Vector3 { x: 7./16., y: 0., z: 0. },
        Vector3 { x: 7./16., y: 0., z: 1. },
        Vector3 { x: 7./16., y: 1., z: 0. },
        Vector3 { x: 7./16., y: 1., z: 1. },
    ],
    // Pos X
    [
        Vector3 { x: 9./16., y: 0., z: 1. },
        Vector3 { x: 9./16., y: 0., z: 0. },
        Vector3 { x: 9./16., y: 1., z: 1. },
        Vector3 { x: 9./16., y: 1., z: 0. },
    ],
];

#[rustfmt::skip]
const TORCH_TOP_TEX_COORDS: [Vector2<f32>; 4] = [
    Vector2 { x: 7./16., y: 9./16. },
    Vector2 { x: 9./16., y: 9./16. },
    Vector2 { x: 7./16., y: 7./16. },
    Vector2 { x: 9./16., y: 7./16. },
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

const FACE_LIGHTING: [u8; 6] = [9, 9, 6, 15, 12, 12];

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
    chunks: ChunkNeighborhood<'a>,
    current_block_coords: BlockCoords,
    meshes: ChunkMeshes,
}

struct FaceDesc<'a> {
    points: &'a [Vector3<f32>; 4],
    texture_id: u32,
    texture_coords: &'a [Vector2<f32>; 4],
    diffused_light: u8,
    sun_light: LightLevel,
    block_light: LightLevel,
}

impl<'a> MeshGenerationContext<'a> {
    fn new(world: &'a World, chunk: &'a Chunk, chunk_coords: ChunkCoords) -> Self {
        let chunks = ChunkNeighborhood::new(world, chunk, chunk_coords);

        MeshGenerationContext {
            chunks,
            current_block_coords: BlockCoords::zero(),
            meshes: ChunkMeshes {
                solid_vertices: vec![],
                water_vertices: vec![],
                water_faces: vec![],
            },
        }
    }

    fn is_transparent(cell: Option<Cell>) -> bool {
        cell.map_or(false, |cell| cell.get_block().is_transparent())
    }

    fn get_light_levels(cell: Option<Cell>) -> (u8, u8) {
        cell.map_or((15, 0), |cell| (cell.sun_light, cell.block_light))
    }

    fn emit_face_vertices(
        vertex_array: &mut Vec<Vertex>,
        block_coords: BlockCoords,
        desc: FaceDesc,
    ) {
        let offset = block_coords.map(|x| x as f32);
        let base_texture_coords = Vector2 {
            x: (desc.texture_id % 4) as f32,
            y: (desc.texture_id / 4) as f32,
        };

        desc.points
            .iter()
            .zip(desc.texture_coords)
            .map(|(&pos, tex)| {
                Vertex::new(VertexDesc {
                    pos: pos + offset,
                    tex: (base_texture_coords + tex) / 4.,
                    diffused_light: desc.diffused_light,
                    sun_light: desc.sun_light,
                    block_light: desc.block_light,
                })
            })
            .for_each(|x| vertex_array.push(x));
    }

    fn emit_solid_face(&mut self, desc: FaceDesc) {
        Self::emit_face_vertices(
            &mut self.meshes.solid_vertices,
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
            self.current_block_coords,
            desc,
        );
    }

    fn emit_solid_block(&mut self, texture_ids: &[u32; 6]) {
        for (i, neighbor_offset) in NEIGHBOR_OFFSETS.iter().enumerate() {
            let neighbor_coords = self.current_block_coords + neighbor_offset;
            let neighbor_cell = self.chunks.get_cell(neighbor_coords);
            let draw_face = Self::is_transparent(neighbor_cell);
            let (sun_light, block_light) = Self::get_light_levels(neighbor_cell);

            if draw_face {
                self.emit_solid_face(FaceDesc {
                    points: &SOLID_BLOCK_FACES[i],
                    texture_id: texture_ids[i],
                    texture_coords: &TEX_COORDS,
                    diffused_light: FACE_LIGHTING[i],
                    sun_light,
                    block_light,
                });
            }
        }
    }

    fn emit_water_block(&mut self, texture_id: u32) {
        const TOP_NEIGHBOR_OFFSET_INDEX: usize = 3;

        let top_neighbor = self
            .chunks
            .get_cell(self.current_block_coords + NEIGHBOR_OFFSETS[TOP_NEIGHBOR_OFFSET_INDEX]);
        let top_neighbor_block = top_neighbor.as_ref().map(Cell::get_block);
        let top_neighbor_is_fluid = matches!(top_neighbor_block, Some(Block::Fluid { .. }));
        let model = if top_neighbor_is_fluid {
            &SOLID_BLOCK_FACES
        } else {
            &FLUID_BLOCK_FACES
        };

        for (i, neighbor_offset) in NEIGHBOR_OFFSETS.iter().enumerate() {
            let is_top_face = i == TOP_NEIGHBOR_OFFSET_INDEX;

            let neighbor_coords = self.current_block_coords + neighbor_offset;
            let neighbor_cell = self.chunks.get_cell(neighbor_coords);
            let (sun_light, block_light) = if is_top_face {
                Self::get_light_levels(self.chunks.get_cell(self.current_block_coords))
            } else {
                Self::get_light_levels(neighbor_cell)
            };

            if let Some(neighbor_cell) = neighbor_cell {
                let neighbor_block = neighbor_cell.get_block();

                let should_not_emit_face = if is_top_face {
                    top_neighbor_is_fluid
                } else {
                    matches!(neighbor_block, Block::Fluid { .. })
                        || !neighbor_block.is_transparent()
                };

                if should_not_emit_face {
                    continue;
                }

                self.emit_water_face(FaceDesc {
                    points: &model[i],
                    texture_id,
                    texture_coords: &TEX_COORDS,
                    diffused_light: FACE_LIGHTING[i],
                    sun_light,
                    block_light,
                });
            }
        }
    }

    fn emit_flower_block(&mut self, texture_id: u32) {
        let cell = self.chunks.get_cell(self.current_block_coords).unwrap();
        let sun_light = cell.sun_light;
        let block_light = cell.block_light;

        for points in &FLOWER_BLOCK_FACES {
            self.emit_solid_face(FaceDesc {
                points,
                texture_id,
                texture_coords: &TEX_COORDS,
                diffused_light: 15,
                sun_light,
                block_light,
            });
        }
    }

    fn emit_torch_block(&mut self, texture_id: u32) {
        let cell = self.chunks.get_cell(self.current_block_coords).unwrap();
        let sun_light = cell.sun_light;
        let block_light = cell.block_light;

        for (i, points) in TORCH_BLOCK_FACES.iter().enumerate() {
            let texture_coords = if i == 0 {
                &TORCH_TOP_TEX_COORDS
            } else {
                &TEX_COORDS
            };
            self.emit_solid_face(FaceDesc {
                points,
                texture_id,
                texture_coords,
                diffused_light: 15,
                sun_light,
                block_light,
            });
        }
    }
}

impl ChunkMeshes {
    pub fn generate(world: &World, chunk: &Chunk, chunk_coords: ChunkCoords) -> Self {
        let mut generation_context = MeshGenerationContext::new(world, chunk, chunk_coords);

        for x in 0..Chunk::SIZE.x {
            for y in 0..Chunk::SIZE.y {
                for z in 0..Chunk::SIZE.z {
                    generation_context.current_block_coords = BlockCoords { x, y, z };
                    let current_cell = chunk[generation_context.current_block_coords];

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
                        Block::Torch { texture_id } => {
                            generation_context.emit_torch_block(*texture_id);
                        }
                    }
                }
            }
        }

        generation_context.meshes
    }
}
