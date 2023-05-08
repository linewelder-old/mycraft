use num_enum::{IntoPrimitive, TryFromPrimitive};

pub enum Block {
    Empty,
    Solid { texture_ids: [u16; 6] },
    Fluid { texture_id: u16 },
    Flower { texture_id: u16 },
    Torch { texture_id: u16 },
}

impl Block {
    pub fn is_transparent(&self) -> bool {
        !matches!(self, Block::Solid { .. })
    }

    pub fn light_level(&self) -> u8 {
        if let Block::Torch { .. } = self {
            10
        } else {
            0
        }
    }

    #[inline]
    pub fn by_id(id: BlockId) -> &'static Self {
        &BLOCKS[Into::<u16>::into(id) as usize]
    }
}

macro_rules! define_blocks {
    ($($name:ident => $def:expr),+ $(,)?) => {
        #[derive(Clone, Copy, IntoPrimitive, TryFromPrimitive)]
        #[repr(u16)]
        pub enum BlockId {
            $($name),+
        }

        const BLOCKS: &[Block] = &[
            $($def),+
        ];
    };
}

define_blocks! {
    Air => Block::Empty,
    Stone => Block::Solid {
        texture_ids: [0; 6],
    },
    Grass => Block::Solid {
        texture_ids: [2, 2, 3, 1, 2, 2],
    },
    Dirt => Block::Solid {
        texture_ids: [3; 6],
    },
    Trunk => Block::Solid {
        texture_ids: [5, 5, 4, 4, 5, 5],
    },
    Leaves => Block::Solid {
        texture_ids: [6; 6],
    },
    Water => Block::Fluid {
        texture_id: 7,
    },
    Sand => Block::Solid {
        texture_ids: [8; 6],
    },
    Planks => Block::Solid {
        texture_ids: [9; 6],
    },
    RedFlower => Block::Flower {
        texture_id: 10,
    },
    YellowFlower => Block::Flower {
        texture_id: 11,
    },
    Torch => Block::Torch {
        texture_id: 12,
    }
}
