use cgmath::Vector3;

pub struct Aabb {
    pub start: Vector3<f32>,
    pub size: Vector3<f32>,
}

impl Aabb {
    pub fn farthest_point_in_direction(&self, direction: Vector3<f32>) -> Vector3<f32> {
        self.start
            + Vector3 {
                x: if direction.x > 0. { self.size.x } else { 0. },
                y: if direction.y > 0. { self.size.y } else { 0. },
                z: if direction.z > 0. { self.size.z } else { 0. },
            }
    }
}
