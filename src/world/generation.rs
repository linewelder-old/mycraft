use cgmath::{InnerSpace, Vector2};
use noise::{NoiseFn, Perlin};

use super::{blocks::BlockId, BlockCoords, Chunk, ChunkCoords};

pub struct Generator {
    noise: Perlin,
}

fn hash(seed: Vector2<f64>) -> f64 {
    let m1 = Vector2::new(3.1251, 17.8737);
    let m2 = 43758.545312;
    (seed.dot(m1).sin() * m2).fract()
}

fn set_block(chunk: &mut Chunk, coords: BlockCoords, id: BlockId) {
    if coords.x >= 0
        && coords.x < Chunk::SIZE.x
        && coords.y >= 0
        && coords.y < Chunk::SIZE.y
        && coords.z >= 0
        && coords.z < Chunk::SIZE.z
    {
        chunk[coords].block_id = id;
    }
}

fn fill(chunk: &mut Chunk, start: BlockCoords, end: BlockCoords, id: BlockId) {
    for x in start.x..=end.x {
        for y in start.y..=end.y {
            for z in start.z..=end.z {
                set_block(chunk, BlockCoords { x, y, z }, id);
            }
        }
    }
}

fn plant_tree(chunk: &mut Chunk, ground: BlockCoords) {
    set_block(chunk, ground, BlockId::Dirt);
    fill(
        chunk,
        ground + BlockCoords::new(-2, 3, -2),
        ground + BlockCoords::new(2, 4, 2),
        BlockId::Leaves,
    );
    fill(
        chunk,
        ground + BlockCoords::new(-1, 5, -1),
        ground + BlockCoords::new(1, 6, 1),
        BlockId::Leaves,
    );
    fill(
        chunk,
        ground + BlockCoords::new(0, 1, 0),
        ground + BlockCoords::new(0, 3, 0),
        BlockId::Trunk,
    );
}

impl Generator {
    const BASE_HEIGHT: f64 = 10.;
    const WATER_HEIGHT: i32 = 27;

    pub fn new(seed: u32) -> Self {
        Generator {
            noise: Perlin::new(seed),
        }
    }

    #[inline]
    fn get_noise(&self, offset: Vector2<f64>, freq: f64, scale: f64) -> f64 {
        (self.noise.get((offset / freq).into()) / 2. + 0.5) * scale
    }

    fn get_height(&self, x: i32, z: i32) -> i32 {
        let offset = Vector2 {
            x: x as f64,
            y: z as f64,
        };

        let octaves = [
            self.get_noise(offset, 80., 24.),
            self.get_noise(offset, 30., 12.),
            self.get_noise(offset, 15., 4.),
            self.get_noise(offset, 10., 3.),
        ];

        let height = Self::BASE_HEIGHT + octaves.iter().sum::<f64>();
        height as i32
    }

    pub fn generate_chunk(&self, chunk: &mut Chunk, chunk_coords: ChunkCoords) {
        for x in -2..(Chunk::SIZE.x + 2) {
            for z in -2..(Chunk::SIZE.z + 2) {
                let world_coords = Vector2 {
                    x: x + chunk_coords.x * Chunk::SIZE.x,
                    y: z + chunk_coords.y * Chunk::SIZE.z,
                };
                let offset = world_coords.map(|x| x as f64);

                let height = self.get_height(world_coords.x, world_coords.y);
                let sand_height = Self::WATER_HEIGHT + self.get_noise(offset, 30., 3.) as i32;
                let plant_random = hash(offset);

                let is_grass = height > sand_height;

                if x >= 0 && x < Chunk::SIZE.x && z >= 0 && z < Chunk::SIZE.z {
                    for y in 0..=Chunk::SIZE.y {
                        let coords = BlockCoords { x, y, z };
                        chunk[coords].block_id = if y < height - 3 {
                            BlockId::Stone
                        } else if y < height {
                            BlockId::Dirt
                        } else if y == height {
                            if !is_grass {
                                BlockId::Sand
                            } else {
                                BlockId::Grass
                            }
                        } else if y <= Self::WATER_HEIGHT {
                            BlockId::Water
                        } else {
                            break;
                        };
                    }

                    if is_grass {
                        let coords = BlockCoords::new(x, height + 1, z);
                        if plant_random > 0.95 {
                            chunk[coords].block_id = BlockId::RedFlower;
                        } else if plant_random > 0.9 {
                            chunk[coords].block_id = BlockId::YellowFlower;
                        }
                    }
                }

                if is_grass && plant_random > 0.99 {
                    plant_tree(chunk, BlockCoords { x, y: height, z });
                }
            }
        }
    }
}
