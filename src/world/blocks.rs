use num_enum::{IntoPrimitive, TryFromPrimitive};

pub enum Block {
    Empty,
    Solid { texture_ids: [u32; 6] },
    Fluid { texture_id: u32 },
    Flower { texture_id: u32 },
}

impl Block {
    pub fn is_transparent(&self) -> bool {
        match self {
            Block::Empty | Block::Fluid { .. } | Block::Flower { .. } => true,
            Block::Solid { .. } => false,
        }
    }

    #[inline]
    pub fn by_id(id: BlockId) -> &'static Self {
        &BLOCKS[Into::<u16>::into(id) as usize]
    }
}

#[derive(Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum BlockId {
    Air,
    Stone,
    Grass,
    Dirt,
    Trunk,
    Leaves,
    Water,
    Sand,
    Planks,
    RedFlower,
    YellowFlower,
}

#[rustfmt::skip]
const BLOCKS: &[Block] = &[
    Block::Empty, // Air
    Block::Solid {  // Stone
        texture_ids: [0; 6],
    },
    Block::Solid {  // Grass
        texture_ids: [2, 2, 3, 1, 2, 2],
    },
    Block::Solid {  // Dirt
        texture_ids: [3; 6],
    },
    Block::Solid {  // Trunk
        texture_ids: [5, 5, 4, 4, 5, 5],
    },
    Block::Solid {  // Leaves
        texture_ids: [6; 6],
    },
    Block::Fluid {  // Water
        texture_id: 7,
    },
    Block::Solid {  // Sand
        texture_ids: [8; 6],
    },
    Block::Solid {  // Planks
        texture_ids: [9; 6],
    },
    Block::Flower { // Red Flower
        texture_id: 10,
    },
    Block::Flower { // Yellow Flower
        texture_id: 11,
    }
];
