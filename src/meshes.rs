use cgmath::Vector3;

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
