use std::rc::Rc;

use cgmath::Vector3;

use crate::{camera::Camera, context::Context, sky::Sky};

use super::{
    texture::{DepthBuffer, Texture},
    uniform::Uniform,
    Bindable, ChunkGraphics, RenderTargetWithDepth, Vertex,
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

struct SolidBlockPipeline {
    render_pipeline: wgpu::RenderPipeline,
}

impl SolidBlockPipeline {
    pub fn new(context: &Context) -> Self {
        let bind_group_layouts = &[
            &Camera::create_bind_group_layout(context),
            &Sky::create_bind_group_layout(context),
            &Texture::create_bind_group_layout(context),
            &Uniform::<Vector3<f32>>::create_bind_group_layout(context),
        ];

        let layout = context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Solid Block Render Pipeline Layout"),
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
                    label: Some("Solid Block Render Pipeline"),
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
                            format: context.surface_config.borrow().format,
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
                        format: DepthBuffer::FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        SolidBlockPipeline { render_pipeline }
    }

    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        camera: &'a Camera,
        chunks: impl Iterator<Item = &'a ChunkGraphics>,
        sky_uniform: &'a Sky,
        texture: &'a Texture,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera.get_bind_group(), &[]);
        render_pass.set_bind_group(1, sky_uniform.get_bind_group(), &[]);
        render_pass.set_bind_group(2, texture.get_bind_group(), &[]);

        for chunk in chunks {
            render_pass.set_bind_group(3, chunk.offset.get_bind_group(), &[]);

            render_pass.set_vertex_buffer(0, chunk.solid_mesh.vertices.slice(..));
            render_pass.set_index_buffer(
                chunk.solid_mesh.indices.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..chunk.solid_mesh.index_count, 0, 0..1);
        }
    }
}

struct WaterPipeline {
    render_pipeline: wgpu::RenderPipeline,
}

impl WaterPipeline {
    pub fn new(context: &Context) -> Self {
        let bind_group_layouts = &[
            &Camera::create_bind_group_layout(context),
            &Sky::create_bind_group_layout(context),
            &Texture::create_bind_group_layout(context),
            &Uniform::<Vector3<f32>>::create_bind_group_layout(context),
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
                            format: context.surface_config.borrow().format,
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
                        format: DepthBuffer::FORMAT,
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        WaterPipeline { render_pipeline }
    }

    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        camera: &'a Camera,
        chunks: impl Iterator<Item = &'a ChunkGraphics>,
        sky: &'a Sky,
        texture: &'a Texture,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera.get_bind_group(), &[]);
        render_pass.set_bind_group(1, sky.get_bind_group(), &[]);
        render_pass.set_bind_group(2, texture.get_bind_group(), &[]);

        for chunk in chunks {
            render_pass.set_bind_group(3, chunk.offset.get_bind_group(), &[]);

            render_pass.set_vertex_buffer(0, chunk.water_mesh.vertices.slice(..));
            render_pass.set_index_buffer(
                chunk.water_mesh.indices.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..chunk.water_mesh.index_count, 0, 0..1);
        }
    }
}
