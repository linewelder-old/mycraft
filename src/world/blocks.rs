pub struct Block {
    pub transparent: bool,
    pub texture_ids: Option<[u32; 6]>,
}

#[rustfmt::skip]
pub const BLOCKS: &[Block] = &[
    Block {  // Air
        transparent: true,
        texture_ids: None,
    },
    Block {  // Stone
        transparent: false,
        texture_ids: Some([0; 6]),
    },
    Block {  // Grass
        transparent: false,
        texture_ids: Some([2, 2, 3, 1, 2, 2]),
    },
    Block {  // Dirt
        transparent: false,
        texture_ids: Some([3; 6]),
    },
    Block {  // Trunk
        transparent: false,
        texture_ids: Some([5, 5, 4, 4, 5, 5]),
    },
    Block {  // Leaves
        transparent: false,
        texture_ids: Some([6; 6]),
    },
    Block {  // Water
        transparent: false,
        texture_ids: Some([7; 6]),
    },
    Block {  // Sand
        transparent: false,
        texture_ids: Some([8; 6]),
    },
    Block {  // Planks
        transparent: false,
        texture_ids: Some([9; 6]),
    },
];
