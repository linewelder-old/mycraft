use cgmath::{Matrix4, Vector2, Vector3};

use crate::{
    camera::Camera,
    context::Context,
    rendering::{
        texture::{Texture, DEPTH_FORMAT},
        uniform::Uniform,
        vertex_array::VertexArray,
    },
};

#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub tex: Vector2<f32>,
    pub normal: Vector3<f32>,
}

impl Vertex {
    const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3],
    };
}

pub struct ChunkRenderer {
    render_pipeline: wgpu::RenderPipeline,
}

pub struct ChunkRendererTarget<'a> {
    pub output: &'a wgpu::TextureView,
    pub depth_buffer: &'a wgpu::TextureView,
}

pub struct ChunkGraphics {
    pub mesh: VertexArray<Vertex>,
    pub transform: Uniform<Matrix4<f32>>,
}

impl ChunkRenderer {
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
            .create_shader_module(wgpu::include_wgsl!("chunk_shader.wgsl"));

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

        ChunkRenderer { render_pipeline }
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
                    load: wgpu::LoadOp::Load,
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
            render_pass.set_vertex_buffer(0, chunk.mesh.vertices.slice(..));
            render_pass.draw(0..chunk.mesh.vertex_count, 0..1);
        }

        drop(render_pass);
        context.queue.submit(std::iter::once(encoder.finish()));
    }
}
