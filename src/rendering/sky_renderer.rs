use cgmath::{Matrix4, Vector2};
use wgpu::util::DeviceExt;

use super::{texture::Texture, uniform::Uniform};
use crate::{camera::Camera, context::Context, utils::as_bytes_slice};

pub struct SkyRenderer {
    render_pipeline: wgpu::RenderPipeline,
    screen_quad: wgpu::Buffer,
}

impl SkyRenderer {
    const SCREEN_QUAD_VERTICES: [Vector2<f32>; 6] = [
        Vector2 { x: -1., y: 1. },
        Vector2 { x: -1., y: -1. },
        Vector2 { x: 1., y: -1. },
        Vector2 { x: -1., y: 1. },
        Vector2 { x: 1., y: -1. },
        Vector2 { x: 1., y: 1. },
    ];

    const VERTEX_BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vector2<f32>>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
    };

    pub fn new(context: &Context) -> Self {
        let bind_group_layouts = &[
            &Uniform::<Matrix4<f32>>::create_bind_group_layout(context),
            &Texture::create_bind_group_layout(context),
        ];

        let layout = context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Sky Render Pipeline Layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        let shader = context
            .device
            .create_shader_module(wgpu::include_wgsl!("sky_shader.wgsl"));

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Sky Render Pipeline"),
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
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        let screen_quad = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Screen Quad Vertices"),
                contents: as_bytes_slice(&Self::SCREEN_QUAD_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        SkyRenderer {
            render_pipeline,
            screen_quad,
        }
    }

    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        camera: &Camera,
        texture: &Texture,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Sky Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera.get_bind_group(), &[]);
        render_pass.set_bind_group(1, texture.get_bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.screen_quad.slice(..));
        render_pass.draw(0..(Self::SCREEN_QUAD_VERTICES.len() as u32), 0..1);
    }
}
