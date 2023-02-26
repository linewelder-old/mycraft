use cgmath::Matrix4;

use crate::{
    camera::Camera,
    context::Context,
    rendering::{
        texture::{Texture, DEPTH_FORMAT},
        uniform::Uniform,
        ChunkGraphics, ChunkRendererTarget, Vertex,
    },
};

pub struct WaterRenderer {
    render_pipeline: wgpu::RenderPipeline,
}

impl WaterRenderer {
    pub fn new(context: &Context) -> Self {
        let bind_group_layouts = &[
            &Uniform::<Matrix4<f32>>::create_bind_group_layout(context),
            &Uniform::<Matrix4<f32>>::create_bind_group_layout(context),
            &Texture::create_bind_group_layout(context),
        ];

        let layout = context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Water Render Pipeline Layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        let shader = context
            .device
            .create_shader_module(wgpu::include_wgsl!("water_shader.wgsl"));

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Water Render Pipeline"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Vertex::BUFFER_LAYOUT],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: context.surface_config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: DEPTH_FORMAT,
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        WaterRenderer { render_pipeline }
    }

    pub fn draw<'a>(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: ChunkRendererTarget,
        camera: &Camera,
        chunks: impl Iterator<Item = &'a ChunkGraphics>,
        texture: &Texture,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Water Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.output,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: target.depth_buffer,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: false,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera.get_bind_group(), &[]);
        render_pass.set_bind_group(2, texture.get_bind_group(), &[]);

        for chunk in chunks {
            render_pass.set_bind_group(1, chunk.transform.get_bind_group(), &[]);
            render_pass.set_vertex_buffer(0, chunk.water_mesh.vertices.slice(..));
            render_pass.set_index_buffer(
                chunk.water_mesh.indices.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..chunk.water_mesh.index_count, 0, 0..1);
        }
    }
}
