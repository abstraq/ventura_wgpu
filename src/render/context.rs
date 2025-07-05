use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroup, Device, Queue, Surface, SurfaceConfiguration, SurfaceError};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::render::camera::OrthographicCamera;
use crate::render::{SpritePipeline, Texture};

const CLEAR_COLOR: wgpu::Color = wgpu::Color { r: 0.01, g: 0.01, b: 0.01, a: 1.0 };

/// Provides a context for interacting with the WGPU API.
///
/// The RenderContext stores all state needed to maintain the WGPU surface.
pub struct RenderContext {
	device: Device,
	queue: Queue,
	surface: Surface<'static>,
	surface_configuration: SurfaceConfiguration,
	camera_bind_group: BindGroup,
	sprite_pipeline: SpritePipeline,
}

impl RenderContext {
	/// Creates the render context.
	pub fn new(window: Arc<Window>) -> Self {
		let window_size = window.inner_size();

		let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
			backends: wgpu::Backends::PRIMARY,
			backend_options: wgpu::BackendOptions::default(),
			flags: wgpu::InstanceFlags::empty(),
		});

		let surface = instance.create_surface(window.clone()).expect("Failed to create the WGPU surface.");

		let surface_configuration = SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: wgpu::TextureFormat::Bgra8UnormSrgb,
			width: window_size.width,
			height: window_size.height,
			present_mode: wgpu::PresentMode::AutoVsync,
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			desired_maximum_frame_latency: 2,
			view_formats: vec![],
		};

		let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::HighPerformance,
			compatible_surface: Some(&surface),
			force_fallback_adapter: false,
		}))
		.expect("No suitable adapter found for WGPU.");

		// Log adapter information during context initialization.
		let adapter_info = adapter.get_info();
		tracing::info!(
			"Using {} backend on {} with {}({}) driver.",
			&adapter_info.backend,
			&adapter_info.name,
			&adapter_info.driver,
			&adapter_info.driver_info,
		);

		let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
			required_features: wgpu::Features::empty(),
			required_limits: wgpu::Limits::default(),
			..Default::default()
		}))
		.expect("Failed to create logical device for the requested adapter.");

		surface.configure(&device, &surface_configuration);

		let camera = OrthographicCamera::new(window_size.width as f32, window_size.height as f32);

		let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Camera Uniform Buffer"),
			contents: bytemuck::bytes_of(&camera.view_projection()),
			usage: wgpu::BufferUsages::UNIFORM,
		});

		let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Camera Uniform Bind Group Layout"),
			entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
		});

		let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Camera Uniform Bind Group"),
			layout: &camera_bind_group_layout,
			entries: &[wgpu::BindGroupEntry { binding: 0, resource: camera_buffer.as_entire_binding() }],
		});

		let image_bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_sprite.png"));
		let texture = Texture::from_bytes(&device, &queue, image_bytes).unwrap();

		let sprite_pipeline = SpritePipeline::new(&device, &texture, &surface_configuration, &camera_bind_group_layout);

		Self { surface, surface_configuration, device, queue, camera_bind_group, sprite_pipeline }
	}

	/// Render the current frame to the surface.
	pub fn render(&self) -> Result<(), SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let texture_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &texture_view,
					resolve_target: None,
					ops: wgpu::Operations { load: wgpu::LoadOp::Clear(CLEAR_COLOR), store: wgpu::StoreOp::Store },
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});

			self.sprite_pipeline.draw(&mut render_pass, &self.camera_bind_group);
		}

		self.queue.submit([encoder.finish()]);
		output.present();
		Ok(())
	}

	/// Resize the WGPU surface.
	pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.surface_configuration.width = new_size.width;
			self.surface_configuration.height = new_size.height;
			self.surface.configure(&self.device, &self.surface_configuration);
		}
	}
}
