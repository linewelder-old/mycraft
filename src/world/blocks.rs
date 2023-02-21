pub enum Block {
    Empty,
    Solid { texture_ids: [u32; 6] },
}

impl Block {
    pub fn is_transparent(&self) -> bool {
        match self {
            Block::Empty => true,
            Block::Solid { .. } => false,
        }
    }
}

#[rustfmt::skip]
pub const BLOCKS: &[Block] = &[
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
    Block::Solid {  // Water
        texture_ids: [7; 6],
    },
    Block::Solid {  // Sand
        texture_ids: [8; 6],
    },
    Block::Solid {  // Planks
        texture_ids: [9; 6],
    },
];
