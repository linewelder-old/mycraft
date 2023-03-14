use std::cmp::Ordering;

use cgmath::Vector3;

use super::{
    blocks::Block,
    utils::{ChunkNeighborhood, ChunkNeighborhoodMut},
    BlockCoords, Cell, Chunk, ChunkCoords, LightLevel, World,
};

#[rustfmt::skip]
const DIRECTIONS: [Vector3<i32>; 6] = [
    Vector3 { x:  0, y:  0, z: -1 },
    Vector3 { x:  0, y:  0, z:  1 },
    Vector3 { x:  0, y: -1, z:  0 },
    Vector3 { x:  0, y:  1, z:  0 },
    Vector3 { x: -1, y:  0, z:  0 },
    Vector3 { x:  1, y:  0, z:  0 },
];

pub fn recalculate_light(world: &World, chunk: &mut Chunk, coords: ChunkCoords) {
    for x in 0..Chunk::SIZE.x {
        for z in 0..Chunk::SIZE.z {
            let mut sun_light = 15;
            for y in (0..Chunk::SIZE.y).rev() {
                let coords = BlockCoords { x, y, z };
                let cell = &mut chunk[coords];
                let block = cell.get_block();

                if !block.is_transparent() {
                    sun_light = 0;
                }

                cell.sun_light = sun_light;
                cell.block_light = block.light_level();
            }
        }
    }

    let neighbors = ChunkNeighborhood::new(world, chunk, coords);
    for _ in 0..16 {
        propagate_light(chunk, &neighbors);
    }
}

fn propagated(light: LightLevel) -> LightLevel {
    if light > 0 {
        light - 1
    } else {
        0
    }
}

fn propagate_light(chunk: &Chunk, neighbors: &ChunkNeighborhood) {
    for x in 0..Chunk::SIZE.x {
        for y in 0..Chunk::SIZE.y {
            for z in 0..Chunk::SIZE.z {
                let coords = BlockCoords { x, y, z };
                let cell = &chunk[coords];

                if !cell.get_block().is_transparent() || cell.sun_light == 15 {
                    continue;
                }

                let (neighbor_sun_light, neighbor_block_light) = DIRECTIONS
                    .iter()
                    .filter_map(|direction| {
                        let neighbor_cell = neighbors.get_cell(coords + direction)?;
                        if neighbor_cell.get_block().is_transparent() {
                            Some((neighbor_cell.sun_light, neighbor_cell.block_light))
                        } else {
                            None
                        }
                    })
                    .max()
                    .unwrap_or((0, 0));
                let received_sun_light = propagated(neighbor_sun_light);
                let received_block_light = propagated(neighbor_block_light);
                let new_sun_light = cell.sun_light.max(received_sun_light);
                let new_block_light = cell.block_light.max(received_block_light);

                // The chunk is borrowed exclusively by us, no other thread accesses it
                let cell_ptr = cell as *const Cell as *mut Cell;
                unsafe {
                    (*cell_ptr).sun_light = new_sun_light;
                    (*cell_ptr).block_light = new_block_light;
                }
            }
        }
    }
}

pub struct LightUpdater<'a> {
    pub chunks: ChunkNeighborhoodMut<'a>,
}

trait LightFuncs {
    fn get_light(cell: Cell) -> LightLevel;
    fn set_light(cell: &mut Cell, new_light: LightLevel);
    fn get_min_light(cell: Cell) -> LightLevel;
}

struct BlockLightDesc;
impl LightFuncs for BlockLightDesc {
    #[inline]
    fn get_light(cell: Cell) -> LightLevel {
        cell.block_light
    }

    #[inline]
    fn set_light(cell: &mut Cell, new_light: LightLevel) {
        cell.block_light = new_light;
    }

    #[inline]
    fn get_min_light(cell: Cell) -> LightLevel {
        cell.get_block().light_level()
    }
}

struct SunLightDesc;
impl LightFuncs for SunLightDesc {
    #[inline]
    fn get_light(cell: Cell) -> LightLevel {
        cell.sun_light
    }

    #[inline]
    fn set_light(cell: &mut Cell, new_light: LightLevel) {
        cell.sun_light = new_light;
    }

    #[inline]
    fn get_min_light(_cell: Cell) -> LightLevel {
        0
    }
}

impl<'a> LightUpdater<'a> {
    #[inline]
    pub fn new(world: &'a World, chunk: &'a mut Chunk, chunk_coords: ChunkCoords) -> Self {
        LightUpdater {
            chunks: ChunkNeighborhoodMut::new(world, chunk, chunk_coords),
        }
    }

    pub fn on_block_placed(&mut self, coords: BlockCoords, block: &Block) {
        if block.is_transparent() {
            self.borrow_light_from_neighbors::<BlockLightDesc>(coords);

            let above_sun_light = self
                .chunks
                .get_cell(coords + BlockCoords::new(0, 1, 0))
                .map(|cell| cell.sun_light)
                .unwrap_or(15);

            for y in (0..=coords.y).rev() {
                let current_coords = BlockCoords {
                    x: coords.x,
                    y,
                    z: coords.z,
                };
                let current_block = self.chunks.get_cell(current_coords).unwrap().get_block();
                if !current_block.is_transparent() {
                    break;
                }

                if above_sun_light == 15 {
                    self.update_light::<SunLightDesc>(current_coords, 15);
                } else {
                    self.borrow_light_from_neighbors::<SunLightDesc>(current_coords);
                }
            }
        } else {
            self.update_light::<BlockLightDesc>(coords, block.light_level());

            self.dec_light::<SunLightDesc>(coords, 0);
            for y in (0..coords.y).rev() {
                let current_coords = BlockCoords {
                    x: coords.x,
                    y,
                    z: coords.z,
                };
                let current_sun_light = self.chunks.get_cell(current_coords).unwrap().sun_light;
                if current_sun_light < 15 {
                    break;
                }

                self.borrow_light_from_neighbors::<SunLightDesc>(current_coords);
            }
        }
    }

    fn update_light<Funcs: LightFuncs>(&mut self, coords: BlockCoords, new_light: LightLevel) {
        let light = Funcs::get_light(self.chunks.get_cell(coords).unwrap());
        match new_light.cmp(&light) {
            Ordering::Greater => self.inc_light::<Funcs>(coords, new_light),
            Ordering::Less => self.dec_light::<Funcs>(coords, new_light),
            _ => {}
        }
    }

    fn inc_light<Funcs: LightFuncs>(&mut self, coords: BlockCoords, new_light: LightLevel) {
        Funcs::set_light(self.chunks.get_cell_mut(coords).unwrap(), new_light);
        let propagated = propagated(new_light);

        for direction in &DIRECTIONS {
            let neighbor_coords = coords + direction;
            if let Some(neighbor) = self.chunks.get_cell(neighbor_coords) {
                if neighbor.get_block().is_transparent() && Funcs::get_light(neighbor) < propagated
                {
                    self.inc_light::<Funcs>(neighbor_coords, propagated);
                }
            }
        }
    }

    fn dec_light<Funcs: LightFuncs>(&mut self, coords: BlockCoords, new_light: LightLevel) {
        let cell = self.chunks.get_cell_mut(coords).unwrap();
        let old_light = Funcs::get_light(*cell);
        Funcs::set_light(cell, new_light);

        for direction in &DIRECTIONS {
            let neighbor_coords = coords + direction;
            if let Some(neighbor) = self.chunks.get_cell(neighbor_coords) {
                if neighbor.get_block().is_transparent() && Funcs::get_light(neighbor) < old_light {
                    self.borrow_light_from_neighbors::<Funcs>(neighbor_coords);
                }
            }
        }
    }

    fn light_from_neighbors<Funcs: LightFuncs>(&self, coords: BlockCoords) -> LightLevel {
        let neighbor_light = DIRECTIONS
            .iter()
            .filter_map(|direction| {
                let neighbor_coords = coords + direction;
                Some(Funcs::get_light(self.chunks.get_cell(neighbor_coords)?))
            })
            .max()
            .unwrap_or(0);

        propagated(neighbor_light)
    }

    fn borrow_light_from_neighbors<Funcs: LightFuncs>(&mut self, coords: BlockCoords) {
        let min_light = Funcs::get_min_light(self.chunks.get_cell(coords).unwrap());
        let propagated = self.light_from_neighbors::<Funcs>(coords);
        self.update_light::<Funcs>(coords, propagated.max(min_light));
    }
}
