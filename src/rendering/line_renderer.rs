use cgmath::Vector3;

use super::{
    meshes::LineMesh, texture::DepthBuffer, uniform::Uniform, Bindable, RenderTargetWithDepth,
};
use crate::{camera::Camera, context::Context};

pub struct LineRenderer {
    render_pipeline: wgpu::RenderPipeline,
}

impl LineRenderer {
    const VERTEX_BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vector3<f32>>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3],
    };

    pub fn new(context: &Context) -> Self {
        let bind_group_layouts = &[
            &Camera::create_bind_group_layout(context),
            &Uniform::<Vector3<f32>>::create_bind_group_layout(context),
        ];

        let layout = context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Line Render Pipeline Layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        let shader = context
            .device
            .create_shader_module(wgpu::include_wgsl!("line_shader.wgsl"));

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Line Render Pipeline"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Self::VERTEX_BUFFER_LAYOUT],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: context.surface_config.borrow().format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::LineList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: DepthBuffer::FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        LineRenderer { render_pipeline }
    }

    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: RenderTargetWithDepth,
        camera: &Camera,
        mesh: &LineMesh,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Line Render Pass"),
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
                    load: wgpu::LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera.get_bind_group(), &[]);
        render_pass.set_bind_group(1, mesh.offset.get_bind_group(), &[]);
        render_pass.set_vertex_buffer(0, mesh.vertices.slice(..));
        render_pass.draw(0..mesh.vertex_count, 0..1);
    }
}
