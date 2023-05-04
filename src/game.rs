use std::{collections::HashMap, rc::Rc};

use anyhow::Result;
use cgmath::{Vector2, Vector3};
use winit::{
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
    },
    window::CursorGrabMode,
};

use crate::{
    camera::Camera,
    consts::*,
    context::Context,
    egui::EguiContext,
    rendering::{
        sky_renderer::SkyRenderer,
        texture::{DepthBuffer, Texture},
        world_renderer::{WorldRenderer, WorldRendererTarget},
    },
    resources::Resources,
    sky::Sky,
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
    sky_renderer: SkyRenderer,

    sky: Sky,

    camera: Camera,
    looking_at: Option<raycasting::Hit>,
    movement_input: Input3d,
    in_menu: bool,
    egui: EguiContext,

    current_block: BlockId,
    hotbar: HashMap<VirtualKeyCode, BlockId>,

    world: World,
    blocks_texture: Texture,
    sky_texture: Texture,
}

impl Mycraft {
    pub fn try_new(context: Rc<Context>) -> Result<Self> {
        let mut world = World::new(context.clone());
        for x in -RENDER_DISTANCE..RENDER_DISTANCE {
            for y in -RENDER_DISTANCE..RENDER_DISTANCE {
                if x * x + y * y < RENDER_DISTANCE * RENDER_DISTANCE {
                    world.load_chunk(ChunkCoords { x, y });
                }
            }
        }

        let resources = Resources::try_load("./res")?;
        let blocks_texture = Texture::new(&context, "Blocks Texture", resources.blocks_image);
        let sky_texture = Texture::new(&context, "Sky Texture", resources.sky_image);

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
        let sky_renderer = SkyRenderer::new(&context);

        let sky = Sky::new(context.clone());

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

        let hotbar: HashMap<VirtualKeyCode, BlockId> = HashMap::from([
            (Key1, BlockId::Stone),
            (Key2, BlockId::Grass),
            (Key3, BlockId::Dirt),
            (Key4, BlockId::Trunk),
            (Key5, BlockId::Leaves),
            (Key6, BlockId::Water),
            (Key7, BlockId::Sand),
            (Key8, BlockId::Planks),
            (Key9, BlockId::RedFlower),
            (Key0, BlockId::YellowFlower),
            (Minus, BlockId::Torch),
        ]);

        let egui = EguiContext::new(context.clone());

        Ok(Mycraft {
            context,

            depth_buffer,
            world_renderer,
            sky_renderer,

            sky,

            camera,
            looking_at: None,
            movement_input,
            in_menu: false,
            egui,

            current_block: BlockId::Dirt,
            hotbar,

            world,
            blocks_texture,
            sky_texture,
        })
    }

    fn grab_cursor(&mut self) {
        let _ = self
            .context
            .window
            .set_cursor_grab(CursorGrabMode::Confined);
        self.context.window.set_cursor_visible(false);
    }

    fn ungrab_cursor(&mut self) {
        let _ = self.context.window.set_cursor_grab(CursorGrabMode::None);
        self.context.window.set_cursor_visible(true);
    }

    pub fn event(&mut self, event: &Event<()>) {
        if self.in_menu {
            self.egui.handle_event(event);
        }

        match event {
            Event::WindowEvent { window_id, event } if *window_id == self.context.window.id() => {
                match event {
                    WindowEvent::CursorEntered { .. } => {
                        if !self.in_menu {
                            self.grab_cursor();
                        }
                    }

                    WindowEvent::CursorLeft { .. } | WindowEvent::Focused(false) => {
                        self.ungrab_cursor();
                    }

                    WindowEvent::KeyboardInput { input, .. } => {
                        self.movement_input.update(input);

                        if let KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(code),
                            ..
                        } = input
                        {
                            if let Some(block_id) = self.hotbar.get(code) {
                                self.current_block = *block_id;
                            }

                            if *code == VirtualKeyCode::Escape {
                                if self.in_menu {
                                    self.grab_cursor();
                                } else {
                                    self.ungrab_cursor();
                                }
                                self.in_menu = !self.in_menu;
                            }
                        }
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
                                        self.current_block,
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
                if !self.in_menu {
                    self.camera.rotate(
                        Vector2 {
                            x: delta.0 as f32,
                            y: -delta.1 as f32,
                        } * MOUSE_SENSITIVITY,
                    );
                }
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

        self.sky.update(delta);

        let movement = self.movement_input.get_value() * CAMERA_MOVEMENT_SPEED * delta_secs;
        self.camera.move_relative_to_view(movement);
        self.camera.update_matrix();

        self.looking_at = raycasting::cast_ray(
            &self.world,
            self.camera.position,
            self.camera.get_direction(),
            MAX_RAYCASTING_DISTANCE,
        );

        self.world.update(&self.camera);
    }

    pub fn render(&mut self, target: &wgpu::TextureView) {
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        self.sky_renderer.draw(
            &mut encoder,
            target,
            &self.camera,
            self.sky.get_uniform(),
            &self.sky_texture,
        );
        self.world_renderer.draw(
            &mut encoder,
            WorldRendererTarget {
                output: target,
                depth_buffer: &self.depth_buffer,
            },
            &self.camera,
            self.world.render_queue_iter(),
            self.sky.get_uniform(),
            &self.blocks_texture,
        );

        self.egui.draw_frame(&mut encoder, target, |ctx| {
            if !self.in_menu {
                ctx.set_cursor_icon(egui::CursorIcon::None);
            }
        });

        self.context.queue.submit(std::iter::once(encoder.finish()));
    }
}
