use std::rc::Rc;

use cgmath::{Vector2, Vector3};
use winit::{
    event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent},
    window::CursorGrabMode,
};

use crate::{
    camera::Camera,
    consts::*,
    context::Context,
    rendering::{
        texture::{DepthBuffer, Texture},
        world_renderer::{WorldRenderer, WorldRendererTarget},
    },
    utils::{
        input::{Input3d, Input3dDesc},
        raycasting,
    },
    world::{blocks::BlockId, ChunkCoords, World},
};

pub struct Mycraft {
    context: Rc<Context>,

    depth_buffer: DepthBuffer,
    world_renderer: WorldRenderer,

    camera: Camera,
    looking_at: Option<raycasting::Hit>,
    movement_input: Input3d,

    world: World,
    test_texture: Texture,
}

impl Mycraft {
    pub fn new(context: Rc<Context>) -> Self {
        let mut world = World::new(context.clone());
        for x in -5..5 {
            for y in -5..5 {
                world.load_chunk(ChunkCoords { x, y });
            }
        }

        let image = image::load_from_memory(include_bytes!("blocks.png")).unwrap();
        let test_texture = Texture::new(&context, "Cube Texture", image);

        let depth_buffer = {
            let surface_config = context.surface_config.borrow();
            DepthBuffer::new(
                context.clone(),
                "Block Depth Buffer",
                Vector2 {
                    x: surface_config.width,
                    y: surface_config.height,
                },
            )
        };
        let world_renderer = WorldRenderer::new(&context);

        let mut camera = Camera::new(context.clone(), "Camera");
        camera.position = Vector3::new(0., 40., 0.);

        use winit::event::VirtualKeyCode::*;
        let movement_input = Input3d::new(Input3dDesc {
            pos_x: D,
            neg_x: A,
            pos_y: E,
            neg_y: Q,
            pos_z: S,
            neg_z: W,
        });

        Mycraft {
            context,

            depth_buffer,
            world_renderer,

            camera,
            looking_at: None,
            movement_input,

            world,
            test_texture,
        }
    }

    pub fn event(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { window_id, event } if *window_id == self.context.window.id() => {
                match event {
                    WindowEvent::CursorEntered { .. } => {
                        let _ = self
                            .context
                            .window
                            .set_cursor_grab(CursorGrabMode::Confined);
                        self.context.window.set_cursor_visible(false);
                    }

                    WindowEvent::CursorLeft { .. } => {
                        let _ = self.context.window.set_cursor_grab(CursorGrabMode::None);
                        self.context.window.set_cursor_visible(true);
                    }

                    WindowEvent::KeyboardInput { input, .. } => {
                        self.movement_input.update(input);
                    }

                    WindowEvent::MouseInput {
                        button,
                        state: ElementState::Pressed,
                        ..
                    } => {
                        if let Some(hit) = &self.looking_at {
                            match button {
                                MouseButton::Left => {
                                    self.world.set_block(hit.coords, BlockId::Air);
                                }

                                MouseButton::Right => {
                                    self.world.set_block(
                                        hit.coords + hit.side.to_direction(),
                                        BlockId::Dirt,
                                    );
                                }

                                _ => {}
                            }
                        }
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

    pub fn resize(&mut self, size: Vector2<u32>) {
        self.camera.resize_projection(size.x as f32 / size.y as f32);
        self.depth_buffer.resize(size);
    }

    pub fn update(&mut self, delta: std::time::Duration) {
        let delta_secs = delta.as_secs_f32();

        let movement = self.movement_input.get_value() * CAMERA_MOVEMENT_SPEED * delta_secs;
        self.camera.move_relative_to_view(movement);
        self.camera.update_matrix();

        self.looking_at = raycasting::cast_ray(
            &self.world,
            self.camera.position,
            self.camera.get_direction(),
            6.,
        );

        self.world.update_chunk_graphics();
        self.world
            .ensure_water_geometry_is_sorted(self.camera.position);
    }

    pub fn render(&mut self, target: &wgpu::TextureView) {
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        self.world_renderer.draw(
            &mut encoder,
            WorldRendererTarget {
                output: target,
                depth_buffer: &self.depth_buffer,
            },
            &self.camera,
            self.world.render_queue_iter(),
            &self.test_texture,
        );

        self.context.queue.submit(std::iter::once(encoder.finish()));
    }
}
