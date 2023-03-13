use std::time::Duration;

pub const WIN_TITLE: &str = "Mycraft";
pub const WIN_SIZE: (u32, u32) = (1080, 720);
pub const FPS: u32 = 60;

pub const MAX_UPDATE_TIME: Duration = Duration::from_millis(15);

pub const CAMERA_MOVEMENT_SPEED: f32 = 8.;
pub const MOUSE_SENSITIVITY: f32 = 0.2;

#[rustfmt::skip]
pub const SKY_COLOR: wgpu::Color = wgpu::Color { r: 0.43, g: 0.77, b: 0.98, a: 1. };

pub const MAX_RAYCASTING_DISTANCE: f32 = 6.;
