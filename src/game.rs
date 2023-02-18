use cgmath::{Matrix4, Vector2, Vector3, SquareMatrix};
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    window::CursorGrabMode,
};

use crate::{
    camera::Camera,
    consts::*,
    context::Context,
    input1d::Input1d,
    rendering::{
        block_renderer::{BlockRenderer, BlockRendererTarget, Object, Vertex},
        texture::{create_depth_buffer, Texture},
        uniform::Uniform,
        vertex_array::VertexArray,
    },
    world::{Chunk, generation::generate_chunk, mesh::generate_chunk_mesh},
};

pub struct Mycraft {
    depth_buffer: wgpu::TextureView,
    block_renderer: BlockRenderer,
    camera: Camera,

    movement_x_input: Input1d,
    movement_y_input: Input1d,
    movement_z_input: Input1d,

    world_mesh: VertexArray<Vertex>,
    world_transform: Uniform<Matrix4<f32>>,
    test_texture: Texture,
}

impl Mycraft {
    pub fn new(context: &mut Context) -> Self {
        let world_transform = Uniform::new(context, "World Transform", Matrix4::identity());

        let mut chunk = Chunk::new();
        generate_chunk(&mut chunk);
        let world_mesh = generate_chunk_mesh(context, &chunk);

        let image = image::load_from_memory(include_bytes!("test.png")).unwrap();
        let test_texture = Texture::new(context, "Cube Texture", image);

        let depth_buffer = create_depth_buffer(
            context,
            "Block Depth Buffer",
            context.surface_config.width,
            context.surface_config.height,
        );
        let block_renderer = BlockRenderer::new(context, "Block Renderer");
        let camera = Camera::new(context, "Camera");

        use winit::event::VirtualKeyCode::*;
        let movement_x_input = Input1d::new(D, A);
        let movement_y_input = Input1d::new(E, Q);
        let movement_z_input = Input1d::new(W, S);

        Mycraft {
            depth_buffer,
            block_renderer,
            camera,

            movement_x_input,
            movement_y_input,
            movement_z_input,

            world_mesh,
            world_transform,
            test_texture,
        }
    }

    pub fn event(&mut self, context: &mut Context, event: &Event<()>) {
        match event {
            Event::WindowEvent { window_id, event } if *window_id == context.window.id() => {
                match event {
                    WindowEvent::CursorEntered { .. } => {
                        let _ = context.window.set_cursor_grab(CursorGrabMode::Confined);
                        context.window.set_cursor_visible(false);
                    }

                    WindowEvent::CursorLeft { .. } => {
                        let _ = context.window.set_cursor_grab(CursorGrabMode::None);
                        context.window.set_cursor_visible(true);
                    }

                    WindowEvent::KeyboardInput { input, .. } => {
                        self.movement_x_input.update(input);
                        self.movement_y_input.update(input);
                        self.movement_z_input.update(input);
                    }

                    _ => {}
                }
            }

            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                self.camera.rotate(
                    Vector2 {
                        x: delta.0 as f32,
                        y: -delta.1 as f32,
                    } * MOUSE_SENSITIVITY,
                );
            }

            _ => {}
        }
    }

    pub fn resize(&mut self, context: &mut Context, size: Vector2<u32>) {
        self.camera.resize_projection(size.x as f32 / size.y as f32);
        self.depth_buffer = create_depth_buffer(context, "Block Depth Buffer", size.x, size.y);
    }

    pub fn update(&mut self, context: &mut Context, delta: std::time::Duration) {
        let delta_secs = delta.as_secs_f32();

        let movement = Vector3 {
            x: self.movement_x_input.get_value(),
            y: self.movement_y_input.get_value(),
            z: -self.movement_z_input.get_value(),
        } * CAMERA_MOVEMENT_SPEED
            * delta_secs;

        self.camera.move_relative_to_view(movement);
        self.camera.update_matrix(context);
    }

    pub fn render(&mut self, context: &Context, target: &wgpu::TextureView) {
        self.block_renderer.draw(
            context,
            BlockRendererTarget {
                output: target,
                depth_buffer: &self.depth_buffer,
            },
            &self.camera,
            &[Object {
                shape: &self.world_mesh,
                transform: &self.world_transform,
                texture: &self.test_texture,
            }],
        );
    }
}
