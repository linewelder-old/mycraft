use std::cell::RefCell;

use winit::{dpi::PhysicalSize, window::Window};

pub struct Context {
    pub window: Window,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub surface_config: RefCell<wgpu::SurfaceConfiguration>,
}

impl Context {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(Default::default());
        let surface = unsafe {
            instance
                .create_surface(&window)
                .expect("Failed to create the window surface.")
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to get the device adapter.");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: Some("Device"),
                },
                None,
            )
            .await
            .expect("Failed to get the graphics device and the command queue.");

        let surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .expect("Failed to get the default surface config.");
        surface.configure(&device, &surface_config);

        Context {
            window,
            device,
            queue,
            surface,
            surface_config: RefCell::new(surface_config),
        }
    }

    pub fn resize(&self, new_size: PhysicalSize<u32>) {
        let mut surface_config = self.surface_config.borrow_mut();
        surface_config.width = new_size.width;
        surface_config.height = new_size.height;
        self.surface.configure(&self.device, &surface_config);
    }

    pub fn recofigure_curface(&self) {
        let surface_config = self.surface_config.borrow();
        self.surface.configure(&self.device, &surface_config);
    }
}
