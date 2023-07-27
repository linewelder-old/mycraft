use cgmath::Vector3;

use super::{utils::ChunkNeighborhood, BlockCoords, Cell, Chunk, ChunkCoords, LightLevel, World};

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
    puffin::profile_function!("Light recalculation");

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
