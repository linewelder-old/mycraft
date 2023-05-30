use crate::{camera::Camera, context::Context, sky::SkyUniform};

use super::{
    solid_block_pipeline::SolidBlockPipeline,
    texture::{DepthBuffer, Texture},
    uniform::Uniform,
    water_pipeline::WaterPipeline,
    ChunkGraphics,
};

#[derive(Clone, Copy)]
pub struct WorldRendererTarget<'a> {
    pub output: &'a wgpu::TextureView,
    pub depth_buffer: &'a DepthBuffer,
}

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
        target: WorldRendererTarget<'a>,
        camera: &'a Camera,
        chunks: impl Iterator<Item = &'a ChunkGraphics> + Clone,
        sky_uniform: &'a Uniform<SkyUniform>,
        texture: &'a Texture,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("World Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.output,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: target.depth_buffer.get_texture_view(),
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
            sky_uniform,
            texture,
        );
        self.water_pipeline
            .draw(&mut render_pass, camera, chunks, sky_uniform, texture);
    }
}
