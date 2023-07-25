use std::rc::Rc;

use crate::{camera::Camera, context::Context, sky::Sky};

use super::{
    solid_block_pipeline::SolidBlockPipeline, texture::Texture, water_pipeline::WaterPipeline,
    ChunkGraphics, RenderTargetWithDepth,
};

pub struct WorldRenderer {
    solid_block_pipeline: SolidBlockPipeline,
    water_pipeline: WaterPipeline,
    blocks_texture: Rc<Texture>,
}

impl WorldRenderer {
    pub fn new(context: &Context, blocks_texture: Rc<Texture>) -> Self {
        let solid_block_pipeline = SolidBlockPipeline::new(context);
        let water_pipeline = WaterPipeline::new(context);
        WorldRenderer {
            solid_block_pipeline,
            water_pipeline,
            blocks_texture,
        }
    }

    pub fn draw<'a>(
        &'a self,
        encoder: &'a mut wgpu::CommandEncoder,
        target: RenderTargetWithDepth<'a>,
        camera: &'a Camera,
        chunks: impl Iterator<Item = &'a ChunkGraphics> + Clone,
        sky: &'a Sky,
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

        self.solid_block_pipeline.draw(
            &mut render_pass,
            camera,
            chunks.clone(),
            sky,
            &self.blocks_texture,
        );
        self.water_pipeline
            .draw(&mut render_pass, camera, chunks, sky, &self.blocks_texture);
    }
}
