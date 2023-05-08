use std::time::Duration;

pub const WIN_TITLE: &str = "Mycraft";
pub const WIN_SIZE: (u32, u32) = (1080, 720);
pub const FPS: u32 = 60;

pub const MAX_UPDATE_TIME: Duration = Duration::from_millis(15);

pub const CAMERA_MOVEMENT_SPEED: f32 = 8.;
pub const MOUSE_SENSITIVITY: f32 = 0.2;

pub const RENDER_DISTANCE: i32 = 32;
pub const WORLD_HEIGHT: i32 = 16;
pub const MAX_RAYCASTING_DISTANCE: f32 = 6.;

pub const MIDNIGHT_SUNLIGHT: f32 = 0.2;
pub const DAY_LENGTH_SECS: f32 = 10. * 60.;
