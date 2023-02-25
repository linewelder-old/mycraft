use cgmath::Matrix4;

use crate::{
    camera::Camera,
    consts::SKY_COLOR,
    context::Context,
    rendering::{
        texture::{Texture, DEPTH_FORMAT},
        uniform::Uniform,
        ChunkGraphics, ChunkRendererTarget, Vertex,
    },
};

pub struct SolidBlockRenderer {
    render_pipeline: wgpu::RenderPipeline,
}

impl SolidBlockRenderer {
    pub fn new(context: &Context, label: &str) -> Self {
        let bind_group_layouts = &[
            &Uniform::<Matrix4<f32>>::create_bind_group_layout(context),
            &Uniform::<Matrix4<f32>>::create_bind_group_layout(context),
            &Texture::create_bind_group_layout(context),
        ];

        let layout = context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Render Pipeline Layout", label)),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        let shader = context
            .device
            .create_shader_module(wgpu::include_wgsl!("solid_block_shader.wgsl"));

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(&format!("{} Render Pipeline", label)),
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
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        SolidBlockRenderer { render_pipeline }
    }

    pub fn draw<'a>(
        &self,
        context: &Context,
        target: ChunkRendererTarget,
        camera: &Camera,
        chunks: impl Iterator<Item = &'a ChunkGraphics>,
        texture: &Texture,
    ) {
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.output,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(SKY_COLOR),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: target.depth_buffer,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera.get_bind_group(), &[]);
        render_pass.set_bind_group(2, texture.get_bind_group(), &[]);

        for chunk in chunks {
            render_pass.set_bind_group(1, chunk.transform.get_bind_group(), &[]);
            render_pass.set_vertex_buffer(0, chunk.solid_mesh.vertices.slice(..));
            render_pass.draw(0..chunk.solid_mesh.vertex_count, 0..1);
        }

        drop(render_pass);
        context.queue.submit(std::iter::once(encoder.finish()));
    }
}