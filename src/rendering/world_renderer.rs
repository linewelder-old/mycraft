use std::rc::Rc;

use cgmath::Vector3;

use crate::{camera::Camera, context::Context, sky::Sky};

use super::{
    texture::{DepthBuffer, Texture},
    uniform::Uniform,
    Bindable, ChunkGraphics, RenderTargetWithDepth, Vertex,
};

pub struct WorldRenderer {
    solid_block_pipeline: wgpu::RenderPipeline,
    water_pipeline: wgpu::RenderPipeline,
    blocks_texture: Rc<Texture>,
}

impl WorldRenderer {
    pub fn new(context: &Context, blocks_texture: Rc<Texture>) -> Self {
        let solid_block_pipeline = create_world_pipeline(
            context,
            WorldPipelineDesc {
                label: "Solid Block Render Pipeline",
                blend: wgpu::BlendState::REPLACE,
                shader: wgpu::include_wgsl!("solid_block_shader.wgsl"),
                cull_mode: Some(wgpu::Face::Back),
                depth_write_enabled: true,
            },
        );

        let water_pipeline = create_world_pipeline(
            context,
            WorldPipelineDesc {
                label: "Water Block Render Pipeline",
                blend: wgpu::BlendState::ALPHA_BLENDING,
                shader: wgpu::include_wgsl!("water_shader.wgsl"),
                cull_mode: None,
                depth_write_enabled: false,
            },
        );

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

        render_pass.set_pipeline(&self.solid_block_pipeline);
        render_pass.set_bind_group(0, camera.get_bind_group(), &[]);
        render_pass.set_bind_group(1, sky.get_bind_group(), &[]);
        render_pass.set_bind_group(2, self.blocks_texture.get_bind_group(), &[]);

        for chunk in chunks.clone() {
            render_pass.set_bind_group(3, chunk.offset.get_bind_group(), &[]);

            render_pass.set_vertex_buffer(0, chunk.solid_mesh.vertices.slice(..));
            render_pass.set_index_buffer(
                chunk.solid_mesh.indices.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..chunk.solid_mesh.index_count, 0, 0..1);
        }

        render_pass.set_pipeline(&self.water_pipeline);

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

struct WorldPipelineDesc<'a> {
    label: &'static str,
    blend: wgpu::BlendState,
    cull_mode: Option<wgpu::Face>,
    depth_write_enabled: bool,
    shader: wgpu::ShaderModuleDescriptor<'a>,
}

fn create_world_pipeline(context: &Context, desc: WorldPipelineDesc) -> wgpu::RenderPipeline {
    let bind_group_layouts = &[
        &Camera::create_bind_group_layout(context),
        &Sky::create_bind_group_layout(context),
        &Texture::create_bind_group_layout(context),
        &Uniform::<Vector3<f32>>::create_bind_group_layout(context),
    ];

    let layout = context
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} Layout", desc.label)),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

    let shader = context.device.create_shader_module(desc.shader);

    context
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(desc.label),
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
                    blend: Some(desc.blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: desc.cull_mode,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DepthBuffer::FORMAT,
                depth_write_enabled: desc.depth_write_enabled,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
}
