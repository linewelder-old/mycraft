use cgmath::Vector2;
use noise::{NoiseFn, Perlin};

use crate::world::{Chunk, ChunkCoords};

pub struct Generator {
    noise: Perlin,
}

impl Generator {
    const BASE_HEIGHT: f64 = 10.;
    const WATER_HEIGHT: usize = 27;

    pub fn new(seed: u32) -> Self {
        Generator {
            noise: Perlin::new(seed),
        }
    }

    #[inline]
    fn get_noise(&self, offset: Vector2<f64>, freq: f64, scale: f64) -> f64 {
        (self.noise.get((offset / freq).into()) / 2. + 0.5) * scale
    }

    fn get_height(&self, x: i32, z: i32) -> usize {
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
        height as usize
    }

    pub fn generate_chunk(&self, chunk: &mut Chunk, chunk_coords: ChunkCoords) {
        for x in 0..Chunk::SIZE.x {
            for z in 0..Chunk::SIZE.z {
                let height = self.get_height(
                    x as i32 + chunk_coords.x * Chunk::SIZE.x as i32,
                    z as i32 + chunk_coords.y * Chunk::SIZE.z as i32,
                );

                for y in 0..(height - 3) {
                    chunk.data[x][y][z].block_id = 1;
                }

                for y in (height - 3)..height {
                    chunk.data[x][y][z].block_id = 3;
                }

                let offset = Vector2 {
                    x: x as f64,
                    y: z as f64,
                };
                let sand_height = Self::WATER_HEIGHT + self.get_noise(offset, 30., 3.) as usize;
                chunk.data[x][height][z].block_id = if height <= sand_height { 7 } else { 2 };

                for y in (height + 1)..=Self::WATER_HEIGHT {
                    chunk.data[x][y][z].block_id = 6;
                }
            }
        }

        if chunk_coords == (ChunkCoords { x: 0, y: 0 }) {
            for i in 0..256 {
                chunk.data[0][i][0].block_id = 6;
            }
        }
    }
}
