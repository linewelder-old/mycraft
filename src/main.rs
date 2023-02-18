mod camera;
mod consts;
mod context;
mod game;
mod input1d;
mod rendering;
mod utils;
mod world;

use std::time::{Duration, Instant};

use cgmath::Vector2;
use winit::{
    dpi::PhysicalSize,
    event::{Event, StartCause, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::{consts::*, context::Context, game::Mycraft};

fn main() {
    fn resize(context: &mut Context, game: &mut Mycraft, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        context.resize(size);
        game.resize(context, Vector2::new(size.width, size.height));
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(WIN_TITLE)
        .with_inner_size(PhysicalSize {
            width: WIN_SIZE.0,
            height: WIN_SIZE.1,
        })
        .build(&event_loop)
        .expect("Failed to create the window");
    env_logger::init();

    let mut context = pollster::block_on(Context::new(window));
    let mut game = Mycraft::new(&mut context);

    let frame_duration = Duration::new(1, 0) / FPS;
    let mut last_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(StartCause::Init)
        | Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
            let current_frame_time = Instant::now();
            let delta = current_frame_time - last_frame_time;
            last_frame_time = current_frame_time;

            game.update(&mut context, delta);
            context.window.request_redraw();
            control_flow.set_wait_until(current_frame_time + frame_duration);
        }

        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if window_id == context.window.id() => {
            control_flow.set_exit();
        }

        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            window_id,
        } if window_id == context.window.id() => {
            resize(&mut context, &mut game, size);
        }

        Event::WindowEvent {
            event: WindowEvent::ScaleFactorChanged { new_inner_size, .. },
            window_id,
        } if window_id == context.window.id() => {
            resize(&mut context, &mut game, *new_inner_size);
        }

        Event::RedrawRequested(window_id) if window_id == context.window.id() => {
            match context.surface.get_current_texture() {
                Ok(output) => {
                    let target = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    game.render(&context, &target);
                    output.present();
                }
                err @ Err(wgpu::SurfaceError::Lost) => {
                    drop(err);
                    context.resize(PhysicalSize {
                        width: context.surface_config.width,
                        height: context.surface_config.height,
                    });
                }
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    log::error!("Error on frame render: Out of memory");
                    control_flow.set_exit_with_code(1);
                }
                Err(err) => log::error!("Error on frame render: {:?}", err),
            }
        }

        event => {
            game.event(&mut context, &event);
        }
    });
}
