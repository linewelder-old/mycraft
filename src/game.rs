use cgmath::{Vector2, Vector3};
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
        solid_block_renderer::SolidBlockRenderer,
        texture::{create_depth_buffer, Texture},
        water_renderer::WaterRenderer,
        ChunkRendererTarget, RenderQueue,
    },
    world::{BlockCoords, Chunk, ChunkCoords, World},
};

pub struct Mycraft {
    depth_buffer: wgpu::TextureView,
    solid_block_renderer: SolidBlockRenderer,
    water_renderer: WaterRenderer,
    camera: Camera,

    movement_x_input: Input1d,
    movement_y_input: Input1d,
    movement_z_input: Input1d,

    world: World,
    render_queue: RenderQueue,
    prev_cam_chunk_coords: ChunkCoords,
    prev_cam_block_coords: BlockCoords,
    test_texture: Texture,
}

fn get_chunk_block_coords(position: Vector3<f32>) -> (ChunkCoords, BlockCoords) {
    let block_coords = position.map(|x| x.floor() as i32);
    let chunk_coords = ChunkCoords {
        x: block_coords.x.div_euclid(Chunk::SIZE.x as i32),
        y: block_coords.z.div_euclid(Chunk::SIZE.z as i32),
    };

    (chunk_coords, block_coords)
}

impl Mycraft {
    pub fn new(context: &mut Context) -> Self {
        let mut world = World::new();
        for x in -5..5 {
            for y in -5..5 {
                world.load_chunk(ChunkCoords { x, y });
            }
        }
        world.update_chunk_graphics(context);

        let image = image::load_from_memory(include_bytes!("blocks.png")).unwrap();
        let test_texture = Texture::new(context, "Cube Texture", image);

        let depth_buffer = create_depth_buffer(
            context,
            "Block Depth Buffer",
            context.surface_config.width,
            context.surface_config.height,
        );
        let solid_block_renderer = SolidBlockRenderer::new(context);
        let water_renderer = WaterRenderer::new(context);

        let mut camera = Camera::new(context, "Camera");
        camera.position = Vector3::new(0., 40., 0.);

        let (cam_chunk_coords, cam_block_coords) = get_chunk_block_coords(camera.position);
        let render_queue = RenderQueue::new(cam_chunk_coords, &world);

        use winit::event::VirtualKeyCode::*;
        let movement_x_input = Input1d::new(D, A);
        let movement_y_input = Input1d::new(E, Q);
        let movement_z_input = Input1d::new(W, S);

        Mycraft {
            depth_buffer,
            solid_block_renderer,
            water_renderer,
            camera,

            movement_x_input,
            movement_y_input,
            movement_z_input,

            world,
            render_queue,
            prev_cam_chunk_coords: cam_chunk_coords,
            prev_cam_block_coords: cam_block_coords,
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

        let (cam_chunk_coords, cam_block_coords) = get_chunk_block_coords(self.camera.position);
        if cam_chunk_coords != self.prev_cam_chunk_coords {
            self.render_queue.sort(cam_chunk_coords);
            self.prev_cam_chunk_coords = cam_chunk_coords;
        }

        if cam_block_coords != self.prev_cam_block_coords {
            for (coords, graphics) in self.render_queue.iter_with_coords() {
                let chunk_offset = Vector3 {
                    x: (coords.x * Chunk::SIZE.x as i32) as f32,
                    y: 0.,
                    z: (coords.y * Chunk::SIZE.z as i32) as f32,
                };
                let relative_cam_pos = self.camera.position - chunk_offset;

                graphics.sort_water_geometry(context, relative_cam_pos);
            }
            self.prev_cam_block_coords = cam_block_coords;
        }
    }

    pub fn render(&mut self, context: &Context, target: &wgpu::TextureView) {
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.solid_block_renderer.draw(
            &mut encoder,
            ChunkRendererTarget {
                output: target,
                depth_buffer: &self.depth_buffer,
            },
            &self.camera,
            self.render_queue.iter(),
            &self.test_texture,
        );
        self.water_renderer.draw(
            &mut encoder,
            ChunkRendererTarget {
                output: target,
                depth_buffer: &self.depth_buffer,
            },
            &self.camera,
            self.render_queue.iter(),
            &self.test_texture,
        );

        context.queue.submit(std::iter::once(encoder.finish()));
    }
}
