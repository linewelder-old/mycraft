pub mod frustrum;
pub mod line_renderer;
pub mod sky_renderer;
pub mod texture;
pub mod uniform;
pub mod world_renderer;

use crate::context::Context;

pub trait Bindable {
    fn create_bind_group_layout(context: &Context) -> wgpu::BindGroupLayout;
    fn get_bind_group(&self) -> &wgpu::BindGroup;
}

#[derive(Clone, Copy)]
pub struct RenderTargetWithDepth<'a> {
    pub color: &'a wgpu::TextureView,
    pub depth: &'a wgpu::TextureView,
}
