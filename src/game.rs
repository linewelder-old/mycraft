use cgmath::{Matrix4, Vector2, Vector3};
use winit::event::WindowEvent;

use crate::{
    camera::Camera,
    consts::*,
    context::Context,
    cube::create_cube_vertices,
    input1d::Input1d,
    rendering::{
        block_renderer::{BlockRenderer, Object, Vertex},
        texture::Texture,
        uniform::Uniform,
        vertex_array::VertexArray,
    },
};

pub struct Mycraft {
    block_renderer: BlockRenderer,
    camera: Camera,

    movement_x_input: Input1d,
    movement_y_input: Input1d,
    movement_z_input: Input1d,
    rotation_x_input: Input1d,
    rotation_y_input: Input1d,

    cube_shape: VertexArray<Vertex>,
    cube_transform: Uniform<Matrix4<f32>>,
    cube_texture: Texture,
}

impl Mycraft {
    pub fn new(context: &mut Context) -> Self {
        let cube_transform = Uniform::new(
            context,
            "Cube Transform",
            Matrix4::from_translation(Vector3 {
                x: -0.5,
                y: 0.,
                z: -0.5,
            }),
        );
        let cube_shape = VertexArray::new(context, "Cube Shape", &create_cube_vertices());

        let image = image::load_from_memory(include_bytes!("test.png")).unwrap();
        let cube_texture = Texture::new(context, "Cube Texture", image);

        let block_renderer = BlockRenderer::new(context, "Block Renderer");
        let camera = Camera::new(context, "Camera");

        use winit::event::VirtualKeyCode::*;
        let movement_x_input = Input1d::new(D, A);
        let movement_y_input = Input1d::new(E, Q);
        let movement_z_input = Input1d::new(W, S);
        let rotation_x_input = Input1d::new(Right, Left);
        let rotation_y_input = Input1d::new(Up, Down);

        Mycraft {
            block_renderer,
            camera,

            movement_x_input,
            movement_y_input,
            movement_z_input,
            rotation_x_input,
            rotation_y_input,

            cube_shape,
            cube_transform,
            cube_texture,
        }
    }

    pub fn input(&mut self, _context: &mut Context, event: &WindowEvent) -> bool {
        self.movement_x_input.update(event)
            || self.movement_y_input.update(event)
            || self.movement_z_input.update(event)
            || self.rotation_x_input.update(event)
            || self.rotation_y_input.update(event)
    }

    pub fn resize(&mut self, _context: &mut Context, size: Vector2<u32>) {
        self.camera.resize_projection(size.x as f32 / size.y as f32);
    }

    pub fn update(&mut self, context: &mut Context, delta: std::time::Duration) {
        let delta_secs = delta.as_secs_f32();

        let rotation = Vector2 {
            x: self.rotation_x_input.get_value(),
            y: self.rotation_y_input.get_value(),
        } * CAMERA_ROTATION_SPEED
            * delta_secs;

        let movement = Vector3 {
            x: self.movement_x_input.get_value(),
            y: self.movement_y_input.get_value(),
            z: -self.movement_z_input.get_value(),
        } * CAMERA_MOVEMENT_SPEED
            * delta_secs;

        self.camera.rotate(rotation);
        self.camera.move_relative_to_view(movement);
        self.camera.update_matrix(context);
    }

    pub fn render(&mut self, context: &Context, target: &wgpu::TextureView) {
        self.block_renderer.draw(
            context,
            &target,
            &self.camera,
            &[Object {
                shape: &self.cube_shape,
                transform: &self.cube_transform,
                texture: &self.cube_texture,
            }],
        );
    }
}
