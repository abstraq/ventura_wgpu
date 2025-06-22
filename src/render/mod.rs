use std::sync::Arc;

use wgpu::{Device, Queue, Surface, SurfaceConfiguration, SurfaceError};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct RenderContext {
	surface: Surface<'static>,
	device: Device,
	queue: Queue,
	surface_config: SurfaceConfiguration,
	pub window: Arc<Window>,
}

impl RenderContext {
	pub fn new(window: Window) -> Self {
		let window = Arc::new(window);
		let window_size = window.inner_size();

		let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
			backends: wgpu::Backends::PRIMARY,
			flags: wgpu::InstanceFlags::empty(),
			..Default::default()
		});

		let surface = instance
			.create_surface(window.clone())
			.expect("Failed to create the WGPU surface.");

		let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::HighPerformance,
			compatible_surface: Some(&surface),
			force_fallback_adapter: false,
		}))
		.expect("No suitable adapter found for WGPU.");

		let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
			label: None,
			required_features: wgpu::Features::empty(),
			required_limits: wgpu::Limits::default(),
			..Default::default()
		}))
		.expect("Failed to create logical device for the requested adapter.");

		let surface_capabilities = surface.get_capabilities(&adapter);
		let surface_format = surface_capabilities
			.formats
			.iter()
			.find(|format| format.is_srgb())
			.copied()
			.unwrap_or(surface_capabilities.formats[0]);

		let surface_config = SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: window_size.width,
			height: window_size.height,
			present_mode: surface_capabilities.present_modes[0],
			alpha_mode: surface_capabilities.alpha_modes[0],
			desired_maximum_frame_latency: 2,
			view_formats: vec![],
		};

		surface.configure(&device, &surface_config);

		Self { surface, device, queue, surface_config, window }
	}

	pub fn draw(&mut self) -> Result<(), SurfaceError> {
		self.window.request_redraw();
		let output = self.surface.get_current_texture()?;
		let texture_view = output
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

		{
			let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &texture_view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});
		}

		self.queue.submit([encoder.finish()]);
		output.present();

		Ok(())
	}

	pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.surface_config.width = new_size.width;
			self.surface_config.height = new_size.height;
			self.surface.configure(&self.device, &self.surface_config);
		}
	}
}
