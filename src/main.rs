mod camera;
mod consts;
mod context;
mod game;
mod rendering;
mod resources;
mod sky;
mod utils;
mod world;

use std::{
    rc::Rc,
    time::{Duration, Instant},
};

use anyhow::Result;
use cgmath::Vector2;
use winit::{
    dpi::PhysicalSize,
    event::{Event, StartCause, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::{consts::*, context::Context, game::Mycraft};

fn main() -> Result<()> {
    fn resize(context: &Context, game: &mut Mycraft, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        context.resize(size);
        game.resize(Vector2::new(size.width, size.height));
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

    let context = Rc::new(pollster::block_on(Context::new(window)));
    let mut game = Mycraft::try_new(context.clone())?;

    let frame_duration = Duration::new(1, 0) / FPS;
    let mut last_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(StartCause::Init)
        | Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
            let current_frame_time = Instant::now();
            let delta = current_frame_time - last_frame_time;
            last_frame_time = current_frame_time;

            game.update(delta);
            context.window.request_redraw();
            control_flow.set_wait_until(current_frame_time + frame_duration);

            context.window.set_title(&format!("Mycraft | {:.1} FPS", 1. / delta.as_secs_f64()));
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
            resize(&context, &mut game, size);
        }

        Event::WindowEvent {
            event: WindowEvent::ScaleFactorChanged { new_inner_size, .. },
            window_id,
        } if window_id == context.window.id() => {
            resize(&context, &mut game, *new_inner_size);
        }

        Event::RedrawRequested(window_id) if window_id == context.window.id() => {
            match context.surface.get_current_texture() {
                Ok(output) => {
                    let target = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    game.render(&target);
                    output.present();
                }
                err @ Err(wgpu::SurfaceError::Lost) => {
                    drop(err);
                    context.recofigure_curface();
                }
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    log::error!("Error on frame render: Out of memory");
                    control_flow.set_exit_with_code(1);
                }
                Err(err) => log::error!("Error on frame render: {:?}", err),
            }
        }

        event => {
            game.event(&event);
        }
    });
}
