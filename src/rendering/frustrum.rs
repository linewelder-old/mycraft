use cgmath::{InnerSpace, Matrix4, Vector3};

use crate::utils::aabb::Aabb;

struct Plane {
    distance_from_origin: f32,
    normal: Vector3<f32>,
}

impl Plane {
    fn distance_to_point(&self, point: Vector3<f32>) -> f32 {
        point.dot(self.normal) + self.distance_from_origin
    }
}

pub struct Frustrum {
    planes: [Plane; 6],
}

impl Frustrum {
    pub fn new(projection: Matrix4<f32>) -> Self {
        let planes = [
            // Left
            Plane {
                normal: Vector3 {
                    x: projection.x.w + projection.x.x,
                    y: projection.y.w + projection.y.x,
                    z: projection.z.w + projection.z.x,
                },
                distance_from_origin: projection.w.w + projection.w.x,
            },
            // Right
            Plane {
                normal: Vector3 {
                    x: projection.x.w - projection.x.x,
                    y: projection.y.w - projection.y.x,
                    z: projection.z.w - projection.z.x,
                },
                distance_from_origin: projection.w.w - projection.w.x,
            },
            // Bottom
            Plane {
                normal: Vector3 {
                    x: projection.x.w + projection.x.y,
                    y: projection.y.w + projection.y.y,
                    z: projection.z.w + projection.z.y,
                },

                distance_from_origin: projection.w.w + projection.w.y,
            },
            // Top
            Plane {
                normal: Vector3 {
                    x: projection.x.w - projection.x.y,
                    y: projection.y.w - projection.y.y,
                    z: projection.z.w - projection.z.y,
                },
                distance_from_origin: projection.w.w - projection.w.y,
            },
            // Near
            Plane {
                normal: Vector3 {
                    x: projection.x.w + projection.x.z,
                    y: projection.y.w + projection.y.z,
                    z: projection.z.w + projection.z.z,
                },
                distance_from_origin: projection.w.w + projection.w.z,
            },
            // Far
            Plane {
                normal: Vector3 {
                    x: projection.x.w - projection.x.z,
                    y: projection.y.w - projection.y.z,
                    z: projection.z.w - projection.z.z,
                },
                distance_from_origin: projection.w.w - projection.w.z,
            },
        ];

        Frustrum { planes }
    }

    pub fn intersects_with_aabb(&self, aabb: &Aabb) -> bool {
        self.planes.iter().all(|plane| {
            plane.distance_to_point(aabb.farthest_point_in_direction(plane.normal)) > 0.
        })
    }
}
