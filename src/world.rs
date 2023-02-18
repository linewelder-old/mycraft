pub mod generation;
pub mod mesh;

use cgmath::Vector3;

pub struct Chunk {
    pub blocks: [[[bool; Self::SIZE.z]; Self::SIZE.y]; Self::SIZE.x],
}

impl Chunk {
    pub const SIZE: Vector3<usize> = Vector3 {
        x: 16,
        y: 256,
        z: 16,
    };

    pub fn new() -> Self {
        Chunk {
            blocks: [[[false; Self::SIZE.z]; Self::SIZE.y]; Self::SIZE.x]
        }
    }
}
