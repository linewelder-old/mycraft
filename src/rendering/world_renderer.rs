use std::{cell::RefCell, rc::Rc};

use cgmath::{MetricSpace, Vector2, Vector3};
use wgpu::util::DeviceExt;

use crate::{camera::Camera, context::Context, sky::Sky, utils::as_bytes_slice};

use super::{
    texture::{DepthBuffer, Texture},
    uniform::Uniform,
    Bindable, RenderTargetWithDepth,
};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex(u32, u32);

pub struct VertexDesc {
    pub pos: Vector3<u16>,
    pub texture_id: u16,
    pub texture_coords: Vector2<u8>,
    pub diffused_light: u8,
    pub sun_light: u8,
    pub block_light: u8,
}

impl Vertex {
    const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Uint32x2],
    };

    #[inline]
    pub fn new(desc: VertexDesc) -> Self {
        Vertex(
            ((desc.pos.x as u32) & 0x1FF)
                | (((desc.pos.y as u32) & 0x1FF) << 9)
                | (((desc.pos.z as u32) & 0x1FF) << 18)
                | (((desc.texture_coords.x as u32) & 0x1F) << 27),
            ((desc.texture_coords.y as u32) & 0x1F)
                | ((desc.texture_id as u32) << 5)
                | (((desc.sun_light as u32) & 0xF) << 21)
                | (((desc.block_light as u32) & 0xF) << 25)
                | (((desc.diffused_light as u32) & 0x3) << 29),
        )
    }
}

pub struct ChunkMesh {
    context: Rc<Context>,
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub index_count: u32,
}

impl ChunkMesh {
    pub fn new(context: Rc<Context>, label: &str, vertices: &[Vertex], indices: &[u32]) -> Self {
        let index_count = indices.len() as u32;
        let vertices = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Vertices", label)),
                usage: wgpu::BufferUsages::VERTEX,
                contents: as_bytes_slice(vertices),
            });
        let indices = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Indices", label)),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                contents: as_bytes_slice(indices),
            });

        ChunkMesh {
            context,
            vertices,
            indices,
            index_count,
        }
    }

    pub fn write_indices(&self, indices: &[u32]) {
        assert_eq!(indices.len() as u32, self.index_count);
        self.context
            .queue
            .write_buffer(&self.indices, 0, as_bytes_slice(indices));
    }
}

pub struct Face {
    pub base_index: u32,
    pub center: Vector3<f32>,
    pub distance: f32,
}

impl Face {
    const VERTEX_INDICES: [u32; 6] = [0, 1, 2, 2, 1, 3];

    pub fn generate_default_indices(face_count: usize) -> Vec<u32> {
        Self::VERTEX_INDICES
            .iter()
            .cycle()
            .enumerate()
            .map(|(i, x)| x + (i as u32 / 6) * 4)
            .take(face_count * 6)
            .collect()
    }

    pub fn generate_indices(faces: &[Face]) -> Vec<u32> {
        faces
            .iter()
            .flat_map(|face| Self::VERTEX_INDICES.iter().map(|x| x + face.base_index))
            .collect()
    }
}

pub struct ChunkGraphicsData {
    pub water_faces: Vec<Face>,
    pub water_faces_unsorted: bool,
}

pub struct ChunkGraphics {
    pub solid_mesh: ChunkMesh,
    pub water_mesh: ChunkMesh,
    pub offset: Uniform<Vector3<f32>>,

    pub graphics_data: RefCell<ChunkGraphicsData>,
}

impl ChunkGraphics {
    pub fn needs_water_faces_sorting(&self) -> bool {
        let data = self.graphics_data.borrow();
        data.water_faces_unsorted
    }

    pub fn sort_water_faces(&self, relative_cam_pos: Vector3<f32>) {
        puffin::profile_function!();

        let mut data = self.graphics_data.borrow_mut();
        data.water_faces_unsorted = false;

        for face in data.water_faces.iter_mut() {
            face.distance = relative_cam_pos.distance2(face.center);
        }

        data.water_faces
            .sort_by(|x, y| y.distance.total_cmp(&x.distance));
        self.water_mesh
            .write_indices(&Face::generate_indices(&data.water_faces));
    }
}

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
