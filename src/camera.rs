use std::rc::Rc;

use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3, Vector4, Zero};

use crate::{
    context::Context,
    rendering::{frustrum::Frustrum, uniform::Uniform},
};

#[repr(C)]
struct CameraUniform {
    matrix: Matrix4<f32>,
    inverse_matrix: Matrix4<f32>,
    position: Vector3<f32>,
    _padding: f32,
}

pub struct Camera {
    projection: Matrix4<f32>,
    matrix: Matrix4<f32>,
    matrix_uniform: Uniform<CameraUniform>,

    fov: f32,
    near: f32,
    far: f32,

    pub position: Vector3<f32>,
    rotation: Vector2<f32>,
}

impl Camera {
    const MAX_Y_ROTATION: f32 = 90.;
    const MIN_Y_ROTATION: f32 = -90.;

    pub fn new(context: Rc<Context>, label: &str) -> Self {
        Camera {
            projection: Matrix4::identity(),
            matrix: Matrix4::identity(),
            matrix_uniform: Uniform::new(
                context,
                &format!("{} Matrix", label),
                CameraUniform {
                    matrix: Matrix4::identity(),
                    inverse_matrix: Matrix4::identity(),
                    position: Vector3::zero(),
                    _padding: 0.,
                },
            ),

            fov: 60.,
            near: 0.01,
            far: 500.,

            position: Vector3::zero(),
            rotation: Vector2::zero(),
        }
    }

    pub fn update_matrix(&mut self) {
        let matrix_without_translation = self.projection
            * Matrix4::from_angle_x(cgmath::Deg(-self.rotation.y))
            * Matrix4::from_angle_y(cgmath::Deg(self.rotation.x));
        self.matrix = matrix_without_translation * Matrix4::from_translation(-self.position);

        let inverse_matrix = matrix_without_translation
            .invert()
            .unwrap_or(Matrix4::identity());

        self.matrix_uniform.write(CameraUniform {
            matrix: self.matrix,
            inverse_matrix,
            position: self.position,
            _padding: 0.,
        });
    }

    pub fn resize_projection(&mut self, aspect_ratio: f32) {
        self.projection =
            cgmath::perspective(cgmath::Deg(self.fov), aspect_ratio, self.near, self.far);
    }

    pub fn rotate(&mut self, amount: Vector2<f32>) {
        self.rotation += amount;

        if self.rotation.y > Self::MAX_Y_ROTATION {
            self.rotation.y = Self::MAX_Y_ROTATION;
        }
        if self.rotation.y < Self::MIN_Y_ROTATION {
            self.rotation.y = Self::MIN_Y_ROTATION;
        }
    }

    pub fn move_relative_to_view(&mut self, amount: Vector3<f32>) {
        let sin = self.rotation.x.to_radians().sin();
        let cos = self.rotation.x.to_radians().cos();

        self.position += Vector3 {
            x: amount.x * cos - amount.z * sin,
            y: amount.y,
            z: amount.z * cos + amount.x * sin,
        };
    }

    #[inline]
    pub fn get_frustrum(&self) -> Frustrum {
        Frustrum::new(self.matrix)
    }

    pub fn get_direction(&self) -> Vector3<f32> {
        let vec4 = Matrix4::from_angle_y(cgmath::Deg(-self.rotation.x))
            * Matrix4::from_angle_x(cgmath::Deg(self.rotation.y))
            * Vector4::new(0., 0., -1., 0.);

        Vector3::new(vec4.x, vec4.y, vec4.z)
    }

    #[inline]
    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        self.matrix_uniform.get_bind_group()
    }
}
