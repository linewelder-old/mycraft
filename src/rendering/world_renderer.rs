use crate::{camera::Camera, context::Context, sky::Sky};

use super::{
    solid_block_pipeline::SolidBlockPipeline, texture::Texture, water_pipeline::WaterPipeline,
    ChunkGraphics, RenderTargetWithDepth,
};

pub struct WorldRenderer {
    solid_block_pipeline: SolidBlockPipeline,
    water_pipeline: WaterPipeline,
}

impl WorldRenderer {
    pub fn new(context: &Context) -> Self {
        let solid_block_pipeline = SolidBlockPipeline::new(context);
        let water_pipeline = WaterPipeline::new(context);
        WorldRenderer {
            solid_block_pipeline,
            water_pipeline,
        }
    }

    pub fn draw<'a>(
        &'a self,
        encoder: &'a mut wgpu::CommandEncoder,
        target: RenderTargetWithDepth<'a>,
        camera: &'a Camera,
        chunks: impl Iterator<Item = &'a ChunkGraphics> + Clone,
        sky: &'a Sky,
        texture: &'a Texture,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("World Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.color,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: target.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        self.solid_block_pipeline
            .draw(&mut render_pass, camera, chunks.clone(), sky, texture);
        self.water_pipeline
            .draw(&mut render_pass, camera, chunks, sky, texture);
    }
}
