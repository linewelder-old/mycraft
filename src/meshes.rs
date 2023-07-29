use cgmath::Vector3;

use crate::world::Chunk;

const fn vec3(x: f32, y: f32, z: f32) -> Vector3<f32> {
    Vector3 { x, y, z }
}

#[rustfmt::skip]
const fn create_cube_line_mesh(start: Vector3<f32>, end: Vector3<f32>) -> [Vector3<f32>; 12 * 2] {
    [
        vec3(start.x, start.y, start.z), vec3(end.x, start.y, start.z),
        vec3(start.x, start.y, start.z), vec3(start.x, end.y, start.z),
        vec3(end.x, start.y, start.z), vec3(end.x, end.y, start.z),
        vec3(start.x, end.y, start.z), vec3(end.x, end.y, start.z),
        vec3(start.x, start.y, start.z), vec3(start.x, start.y, end.z),
        vec3(end.x, start.y, start.z), vec3(end.x, start.y, end.z),
        vec3(start.x, end.y, start.z), vec3(start.x, end.y, end.z),
        vec3(end.x, end.y, start.z), vec3(end.x, end.y, end.z),
        vec3(start.x, start.y, end.z), vec3(end.x, start.y, end.z),
        vec3(start.x, start.y, end.z), vec3(start.x, end.y, end.z),
        vec3(end.x, start.y, end.z), vec3(end.x, end.y, end.z),
        vec3(start.x, end.y, end.z), vec3(end.x, end.y, end.z),
    ]
}

#[rustfmt::skip]
pub const BLOCK_SELECTION_COLOR: Vector3<f32> = Vector3 { x: 0., y: 0., z: 0. };

const BLOCK_SELECTION_PADDING: f32 = 0.01;
pub const BLOCK_SELECTION_VERTICES: &[Vector3<f32>] = &create_cube_line_mesh(
    Vector3 {
        x: -BLOCK_SELECTION_PADDING,
        y: -BLOCK_SELECTION_PADDING,
        z: -BLOCK_SELECTION_PADDING,
    },
    Vector3 {
        x: 1. + BLOCK_SELECTION_PADDING,
        y: 1. + BLOCK_SELECTION_PADDING,
        z: 1. + BLOCK_SELECTION_PADDING,
    },
);

const CHUNK_BORDERS_VERTEX_COUNT: usize = (4 * 6 * (Chunk::SIZE + 1)) as usize;
const fn create_chunk_borders() -> [Vector3<f32>; CHUNK_BORDERS_VERTEX_COUNT] {
    let mut result = [vec3(0., 0., 0.); CHUNK_BORDERS_VERTEX_COUNT];
    let mut count = 0;

    /// Create 3 lines in the 3 unit directions, since the mesh is symmetrical in that aspect.
    macro_rules! lines {
        ($sx:expr, $sy:expr, $sz:expr => $ex:expr, $ey:expr, $ez:expr) => {
            result[count + 0] = vec3($sx, $sy, $sz);
            result[count + 1] = vec3($ex, $ey, $ez);

            result[count + 2] = vec3($sy, $sx, $sz);
            result[count + 3] = vec3($ey, $ex, $ez);

            result[count + 4] = vec3($sy, $sz, $sx);
            result[count + 5] = vec3($ey, $ez, $ex);

            count += 6;
        };
    }

    const CHUNK_SIZE: f32 = Chunk::SIZE as f32;
    let mut i = 0;
    while i <= Chunk::SIZE {
        let offset = i as f32;

        lines!(offset, 0., 0. => offset, CHUNK_SIZE, 0.);
        lines!(offset, 0., 0. => offset, 0., CHUNK_SIZE);
        lines!(offset, CHUNK_SIZE, 0. => offset, CHUNK_SIZE, CHUNK_SIZE);
        lines!(offset, 0., CHUNK_SIZE => offset, CHUNK_SIZE, CHUNK_SIZE);

        i += 1;
    }

    result
}

#[rustfmt::skip]
pub const CHUNK_BORDERS_COLOR: Vector3<f32> = Vector3 { x: 1., y: 0., z: 0. };
pub const CHUNK_BORDERS_VERTICES: &[Vector3<f32>] = &create_chunk_borders();
