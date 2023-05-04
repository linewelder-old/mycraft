use std::rc::Rc;

use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::event::Event;

use crate::context::Context;

pub struct EguiContext {
    context: Rc<Context>,
    platform: Platform,
    render_pass: RenderPass,
}

impl EguiContext {
    pub fn new(context: Rc<Context>) -> Self {
        let surface_config = context.surface_config.borrow();

        let platform = Platform::new(PlatformDescriptor {
            physical_width: surface_config.width,
            physical_height: surface_config.height,
            scale_factor: context.window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });
        let render_pass = RenderPass::new(&context.device, surface_config.format, 1);

        drop(surface_config);
        EguiContext {
            context,
            platform,
            render_pass,
        }
    }

    pub fn handle_event(&mut self, event: &Event<()>) {
        self.platform.handle_event(event);
    }

    pub fn draw_frame(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        do_ui: impl Fn(&egui::Context),
    ) {
        puffin::profile_function!();

        self.platform.begin_frame();
        let context = self.platform.context();

        do_ui(&context);

        let full_output = self.platform.end_frame(Some(&self.context.window));
        let paint_jobs = context.tessellate(full_output.shapes);

        let screen_descriptor = {
            let surface_config = self.context.surface_config.borrow();
            ScreenDescriptor {
                physical_width: surface_config.width,
                physical_height: surface_config.height,
                scale_factor: self.context.window.scale_factor() as f32,
            }
        };
        let textures_delta = full_output.textures_delta;
        self.render_pass
            .add_textures(&self.context.device, &self.context.queue, &textures_delta)
            .expect("Failed to add egui textures");
        self.render_pass.update_buffers(
            &self.context.device,
            &self.context.queue,
            &paint_jobs,
            &screen_descriptor,
        );

        self.render_pass
            .execute(encoder, target, &paint_jobs, &screen_descriptor, None)
            .unwrap();

        self.render_pass
            .remove_textures(textures_delta)
            .expect("Failed to remove egui textures");
    }
}
