use cgmath::{MetricSpace, Vector3, Zero};

use crate::world::{blocks::Block, BlockCoords, World};

#[derive(Clone, Copy)]
pub enum BlockSide {
    NegZ,
    PosZ,
    NegY,
    PosY,
    NegX,
    PosX,
}

impl BlockSide {
    #[rustfmt::skip]
    pub fn to_direction(self) -> Vector3<i32> {
        match self {
            BlockSide::NegZ => Vector3 { x:  0, y:  0, z: -1 },
            BlockSide::PosZ => Vector3 { x:  0, y:  0, z:  1 },
            BlockSide::NegY => Vector3 { x:  0, y: -1, z:  0 },
            BlockSide::PosY => Vector3 { x:  0, y:  1, z:  0 },
            BlockSide::NegX => Vector3 { x: -1, y:  0, z:  0 },
            BlockSide::PosX => Vector3 { x:  1, y:  0, z:  0 },
        }
    }
}

pub struct Hit {
    pub point: Vector3<f32>,
    pub side: BlockSide,
    pub coords: BlockCoords,
    pub block: &'static Block,
}

pub fn cast_ray(
    world: &World,
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    max_distance: f32,
) -> Option<Hit> {
    // If the ray is parallel to a plane, it will never go through the faces parallel to it
    let check_xy = direction.z != 0.;
    let check_xz = direction.y != 0.;
    let check_yz = direction.x != 0.;

    if !(check_xy || check_xz || check_yz) {
        return None;
    }

    // Move the intersection checkers to the first face on their way
    let xy_step;
    let xz_step;
    let yz_step;
    let mut xy_current_point;
    let mut xz_current_point;
    let mut yz_current_point;

    if check_xy {
        let z = if direction.z > 0. {
            origin.z.ceil()
        } else {
            origin.z.floor()
        };
        let dz = (z - origin.z).abs();

        xy_step = Vector3 {
            x: direction.x / direction.z,
            y: direction.y / direction.z,
            z: 1.,
        } * direction.z.signum();
        xy_current_point = Vector3 {
            x: origin.x + dz * xy_step.x,
            y: origin.y + dz * xy_step.y,
            z,
        };
    } else {
        xy_step = Vector3::zero();
        xy_current_point = origin;
    }

    if check_xz {
        let y = if direction.y > 0. {
            origin.y.ceil()
        } else {
            origin.y.floor()
        };
        let dy = (y - origin.y).abs();

        xz_step = Vector3 {
            x: direction.x / direction.y,
            y: 1.,
            z: direction.z / direction.y,
        } * direction.y.signum();
        xz_current_point = Vector3 {
            x: origin.x + dy * xz_step.x,
            y,
            z: origin.z + dy * xz_step.z,
        };
    } else {
        xz_step = Vector3::zero();
        xz_current_point = origin;
    }

    if check_yz {
        let x = if direction.x > 0. {
            origin.x.ceil()
        } else {
            origin.x.floor()
        };
        let dx = (x - origin.x).abs();

        yz_step = Vector3 {
            x: 1.,
            y: direction.y / direction.x,
            z: direction.z / direction.x,
        } * direction.x.signum();
        yz_current_point = Vector3 {
            x,
            y: origin.y + dx * yz_step.y,
            z: origin.z + dx * yz_step.z,
        };
    } else {
        yz_step = Vector3::zero();
        yz_current_point = origin;
    }

    let max_distance_squared = max_distance * max_distance;
    loop {
        let distance_to_xy = origin.distance2(xy_current_point);
        let distance_to_xz = origin.distance2(xz_current_point);
        let distance_to_yz = origin.distance2(yz_current_point);

        let current_point: Vector3<f32>;
        let hit_side: BlockSide;
        let hit_coords: BlockCoords;

        let next_face_is_xy = check_xy
            && (!check_xz || distance_to_xy < distance_to_xz)
            && (!check_yz || distance_to_xy < distance_to_yz);
        let next_face_is_xz = check_xz
            && (!check_xy || distance_to_xz < distance_to_xy)
            && (!check_yz || distance_to_xz < distance_to_yz);

        if next_face_is_xy {
            if distance_to_xy > max_distance_squared {
                break None;
            }

            current_point = xy_current_point;
            hit_coords = BlockCoords {
                x: xy_current_point.x.floor() as i32,
                y: xy_current_point.y.floor() as i32,
                z: xy_current_point.z as i32 - if xy_step.z < 0. { 1 } else { 0 },
            };
            hit_side = if xy_step.z < 0. {
                BlockSide::PosZ
            } else {
                BlockSide::NegZ
            };

            xy_current_point += xy_step;
        } else if next_face_is_xz {
            if distance_to_xz > max_distance_squared {
                break None;
            }

            current_point = xz_current_point;
            hit_coords = BlockCoords {
                x: xz_current_point.x.floor() as i32,
                y: xz_current_point.y as i32 - if xz_step.y < 0. { 1 } else { 0 },
                z: xz_current_point.z.floor() as i32,
            };
            hit_side = if xz_step.y < 0. {
                BlockSide::PosY
            } else {
                BlockSide::NegY
            };

            xz_current_point += xz_step;
        } else {
            if distance_to_yz > max_distance_squared {
                break None;
            }

            current_point = yz_current_point;
            hit_coords = BlockCoords {
                x: yz_current_point.x as i32 - if yz_step.x < 0. { 1 } else { 0 },
                y: yz_current_point.y.floor() as i32,
                z: yz_current_point.z.floor() as i32,
            };
            hit_side = if yz_step.x < 0. {
                BlockSide::PosX
            } else {
                BlockSide::NegX
            };

            yz_current_point += yz_step;
        }

        if let Some(hit_block) = world.get_block(hit_coords) {
            if !hit_block.is_transparent() {
                break Some(Hit {
                    point: current_point,
                    side: hit_side,
                    coords: hit_coords,
                    block: hit_block,
                });
            }
        }
    }
}
