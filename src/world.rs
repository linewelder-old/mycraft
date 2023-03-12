pub mod blocks;
pub mod generation;
mod light;
pub mod mesh;
mod utils;

use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    ops::{Index, IndexMut},
    rc::Rc,
};

use cgmath::{Vector2, Vector3, Zero};

use crate::{
    context::Context,
    rendering::{chunk_mesh::ChunkMesh, ChunkGraphics, ChunkGraphicsData, Face, RenderQueue},
    world::{
        blocks::{Block, BlockId},
        generation::Generator,
        light::{recalculate_light, LightUpdater},
        mesh::ChunkMeshes,
    },
};

pub type LightLevel = u8;

#[derive(Clone, Copy)]
pub struct Cell {
    pub block_id: BlockId,
    pub sun_light: LightLevel,
    pub block_light: LightLevel,
}

impl Cell {
    #[inline]
    pub fn get_block(&self) -> &'static Block {
        Block::by_id(self.block_id)
    }
}

pub struct Chunk {
    data: [[[Cell; Self::SIZE.z as usize]; Self::SIZE.y as usize]; Self::SIZE.x as usize],
    graphics: Option<Rc<ChunkGraphics>>,
}

impl Chunk {
    pub const SIZE: Vector3<i32> = Vector3 {
        x: 16,
        y: 256,
        z: 16,
    };

    fn new() -> Self {
        Chunk {
            data: [[[Cell {
                block_id: BlockId::Air,
                sun_light: 15,
                block_light: 0,
            }; Self::SIZE.z as usize]; Self::SIZE.y as usize];
                Self::SIZE.x as usize],
            graphics: None,
        }
    }

    fn needs_graphics_update(&self) -> bool {
        if let Some(graphics) = &self.graphics {
            graphics.graphics_data.borrow().needs_update
        } else {
            true
        }
    }

    fn invalidate_graphics(&self) {
        if let Some(graphics) = &self.graphics {
            graphics.graphics_data.borrow_mut().needs_update = true;
        }
    }
}

impl Index<BlockCoords> for Chunk {
    type Output = Cell;

    #[inline]
    fn index(&self, coords: BlockCoords) -> &Self::Output {
        &self.data[coords.x as usize][coords.y as usize][coords.z as usize]
    }
}

impl IndexMut<BlockCoords> for Chunk {
    #[inline]
    fn index_mut(&mut self, coords: BlockCoords) -> &mut Self::Output {
        &mut self.data[coords.x as usize][coords.y as usize][coords.z as usize]
    }
}

pub type ChunkCoords = Vector2<i32>;
pub type BlockCoords = Vector3<i32>;

pub struct World {
    context: Rc<Context>,

    chunks: HashMap<ChunkCoords, RefCell<Chunk>>,
    generator: Generator,

    render_queue: RenderQueue,
    prev_cam_chunk_coords: ChunkCoords,
    prev_cam_block_coords: BlockCoords,
}

fn get_chunk_block_coords(position: Vector3<f32>) -> (ChunkCoords, BlockCoords) {
    let block_coords = position.map(|x| x.floor() as i32);
    let chunk_coords = World::get_chunk_coords(block_coords);

    (chunk_coords, block_coords)
}

impl World {
    pub fn new(context: Rc<Context>) -> Self {
        World {
            context,

            chunks: HashMap::new(),
            generator: Generator::new(0),

            render_queue: RenderQueue::new(),
            prev_cam_block_coords: Vector3::zero(),
            prev_cam_chunk_coords: Vector2::zero(),
        }
    }

    pub fn load_chunk(&mut self, coords: ChunkCoords) {
        if self.chunks.contains_key(&coords) {
            return;
        }

        let mut chunk = Chunk::new();
        self.generator.generate_chunk(&mut chunk, coords);
        crate::timeit!("Light Generation" => recalculate_light(self, &mut chunk, coords));
        self.chunks.insert(coords, RefCell::new(chunk));
    }

    fn check_what_is_to_sort(&mut self, camera_position: Vector3<f32>) {
        let (cam_chunk_coords, cam_block_coords) = get_chunk_block_coords(camera_position);
        if cam_chunk_coords != self.prev_cam_chunk_coords {
            self.render_queue.mark_unsorted();
            self.prev_cam_chunk_coords = cam_chunk_coords;
        }

        if cam_block_coords != self.prev_cam_block_coords {
            for (_, graphics) in self.render_queue.iter_for_update() {
                graphics.graphics_data.borrow_mut().water_faces_unsorted = true;
            }
            self.prev_cam_block_coords = cam_block_coords;
        }
    }

    pub fn ensure_water_geometry_is_sorted(&mut self, camera_position: Vector3<f32>) {
        self.check_what_is_to_sort(camera_position);

        self.render_queue.sort_if_needed(self.prev_cam_chunk_coords);
        for (coords, graphics) in self.render_queue.iter_for_update() {
            let chunk_offset = Vector3 {
                x: (coords.x * Chunk::SIZE.x) as f32,
                y: 0.,
                z: (coords.y * Chunk::SIZE.z) as f32,
            };
            let relative_cam_pos = camera_position - chunk_offset;

            if graphics.sort_water_faces_if_needed(relative_cam_pos) {
                break;
            }
        }
    }

    pub fn update_chunk_graphics(&mut self) {
        for (coords, chunk) in &self.chunks {
            let mut chunk = chunk.borrow_mut();

            if chunk.needs_graphics_update() {
                let meshes = ChunkMeshes::generate(self, &chunk, *coords);
                let solid_mesh = ChunkMesh::new(
                    self.context.clone(),
                    "Solid Chunk Mesh",
                    &meshes.solid_vertices,
                    &Face::generate_default_indices(meshes.solid_vertices.len() * 4),
                );
                let water_mesh = ChunkMesh::new(
                    self.context.clone(),
                    "Water Chunk Mesh",
                    &meshes.water_vertices,
                    &Face::generate_indices(&meshes.water_faces),
                );

                let graphics = Rc::new(ChunkGraphics {
                    solid_mesh,
                    water_mesh,

                    graphics_data: RefCell::new(ChunkGraphicsData {
                        water_faces: meshes.water_faces,
                        needs_update: false,
                        water_faces_unsorted: true,
                    }),
                });
                chunk.graphics = Some(graphics.clone());
                self.render_queue.insert(*coords, graphics);
            }
        }
    }

    pub fn get_chunk_coords(block_coords: BlockCoords) -> ChunkCoords {
        ChunkCoords {
            x: block_coords.x.div_euclid(Chunk::SIZE.x),
            y: block_coords.z.div_euclid(Chunk::SIZE.z),
        }
    }

    pub fn to_chunk_block_coords(block_coords: BlockCoords) -> (ChunkCoords, BlockCoords) {
        let chunk_coords = Self::get_chunk_coords(block_coords);
        let block_coords = BlockCoords {
            x: block_coords.x.rem_euclid(Chunk::SIZE.x),
            y: block_coords.y,
            z: block_coords.z.rem_euclid(Chunk::SIZE.z),
        };

        (chunk_coords, block_coords)
    }

    #[inline]
    pub fn borrow_chunk(&self, coords: ChunkCoords) -> Option<Ref<Chunk>> {
        self.chunks.get(&coords).map(RefCell::borrow)
    }

    #[inline]
    pub fn borrow_mut_chunk(&self, coords: ChunkCoords) -> Option<RefMut<Chunk>> {
        self.chunks.get(&coords).map(RefCell::borrow_mut)
    }

    pub fn get_block(&self, coords: BlockCoords) -> Option<&'static Block> {
        if coords.y < 0 || coords.y >= Chunk::SIZE.y {
            return None;
        }

        let (chunk_coords, block_coords) = Self::to_chunk_block_coords(coords);
        self.borrow_chunk(chunk_coords)
            .map(|chunk| chunk[block_coords].get_block())
    }

    fn invalidate_chunk_graphics(&self, chunk_coords: ChunkCoords) {
        self.borrow_chunk(chunk_coords)
            .map(|chunk| chunk.invalidate_graphics());
    }

    pub fn set_block(&mut self, coords: BlockCoords, block_id: BlockId) {
        if coords.y < 0 || coords.y >= Chunk::SIZE.y {
            return;
        }

        let (chunk_coords, block_coords) = Self::to_chunk_block_coords(coords);
        self.borrow_mut_chunk(chunk_coords).map(|mut chunk| {
            chunk[block_coords].block_id = block_id;
            crate::timeit!("Light Update" => {
                let mut updater = LightUpdater::new(self, &mut chunk, chunk_coords);
                updater.on_block_placed(block_coords, Block::by_id(block_id));
            });
            chunk.invalidate_graphics();

            for x in -1..=1 {
                for y in -1..=1 {
                    if x != 0 || y != 0 {
                        self.invalidate_chunk_graphics(chunk_coords + ChunkCoords { x, y });
                    }
                }
            }
        });
    }

    pub fn render_queue_iter(&self) -> impl Iterator<Item = &ChunkGraphics> {
        self.render_queue.iter_for_render()
    }
}
