use crate::{
    camera::Camera,
    context::Context,
    rendering::{
        solid_block_pipeline::SolidBlockPipeline,
        texture::{DepthBuffer, Texture},
        water_pipeline::WaterPipeline,
        ChunkGraphics,
    },
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
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: WorldRendererTarget,
        camera: &Camera,
        chunks: impl Iterator<Item = &'a ChunkGraphics> + Clone,
        texture: &Texture,
    ) {
        self.solid_block_pipeline
            .draw(encoder, target, camera, chunks.clone(), texture);
        self.water_pipeline
            .draw(encoder, target, camera, chunks, texture);
    }
}
