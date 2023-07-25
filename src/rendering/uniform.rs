use std::{marker::PhantomData, rc::Rc};

use wgpu::util::DeviceExt;

use crate::{context::Context, utils::as_bytes};

use super::Bindable;

pub struct Uniform<T> {
    context: Rc<Context>,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    phantom: PhantomData<T>,
}

impl<T> Uniform<T> {
    #[inline]
    pub fn new(context: Rc<Context>, label: &str, value: T) -> Self {
        let (buffer, bind_group) = create_buffer_and_bind_group(&context, label, as_bytes(&value));

        Uniform {
            context,
            buffer,
            bind_group,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn write(&self, value: T) {
        self.context
            .queue
            .write_buffer(&self.buffer, 0 as wgpu::BufferAddress, as_bytes(&value));
    }
}

impl<T> Bindable for Uniform<T> {
    fn create_bind_group_layout(context: &Context) -> wgpu::BindGroupLayout {
        context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                }],
            })
    }

    #[inline]
    fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

fn create_buffer_and_bind_group(
    context: &Context,
    label: &str,
    contents: &[u8],
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Buffer", label)),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents,
        });

    let bind_group = context
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} Bind Group", label)),
            layout: &Uniform::<()>::create_bind_group_layout(context),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

    (buffer, bind_group)
}
