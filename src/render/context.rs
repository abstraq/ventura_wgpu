use std::sync::Arc;

use glam::{Mat4, Vec3};
use hecs::World;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, SurfaceError};
use winit::window::Window;

use crate::render::camera::{CameraUniform, OrthographicProjection, PrimaryCamera};
use crate::render::sprite::SpritePipeline;
use crate::transform::Transform;

/// The color used to clear the surface each frame.
const CLEAR_COLOR: wgpu::Color = wgpu::Color { r: 0.01, g: 0.01, b: 0.01, a: 1.0 };

/// Provides a context for interacting with the WGPU API.
///
/// The RenderContext stores all state needed to maintain the WGPU surface,
/// including the logical [`wgpu::Device`], the command [`wgpu::Queue`], the
/// presentation [`wgpu::Surface`], and the various pipelines, bind groups, and
/// any other GPU resources.
pub struct RenderContext {
	device: Device,
	queue: Queue,
	surface: Surface<'static>,
	surface_configuration: SurfaceConfiguration,
	camera_buffer: wgpu::Buffer,
	camera_bind_group: wgpu::BindGroup,
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

		// Create the uniform buffer and bind group for the camera.
		let (camera_buffer, camera_bind_group_layout, camera_bind_group) = Self::create_camera_buffer(&device);

		let sprite_pipeline = SpritePipeline::new(&device, &queue, &camera_bind_group_layout);

		Self { surface, surface_configuration, device, queue, camera_buffer, camera_bind_group, sprite_pipeline }
	}

	/// Render the current frame to the surface.
	pub fn render(&mut self, world: &mut World) -> Result<(), SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let texture_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

		// Update the camera uniform buffer.
		self.update_camera_buffer(world);

		// Upload sprite information to the sprite pipeline.
		self.sprite_pipeline.prepare(&self.queue, world);

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
	pub fn resize(&mut self, new_width: u32, new_height: u32) {
		if new_width > 0 && new_height > 0 {
			self.surface_configuration.width = new_width;
			self.surface_configuration.height = new_height;
			self.surface.configure(&self.device, &self.surface_configuration);
		} else {
			tracing::warn!("Attempted to resize WGPU surface width or height smaller than zero.");
		}
	}

	fn create_camera_buffer(device: &Device) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
		let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Camera Uniform Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			contents: bytemuck::bytes_of(&CameraUniform { view_projection: Mat4::IDENTITY }),
		});

		let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Camera Bind Group Layout"),
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
			label: Some("Camera Bind Group"),
			layout: &camera_bind_group_layout,
			entries: &[wgpu::BindGroupEntry { binding: 0, resource: camera_buffer.as_entire_binding() }],
		});

		(camera_buffer, camera_bind_group_layout, camera_bind_group)
	}

	fn update_camera_buffer(&self, world: &World) {
		let mut query = world.query::<(&Transform, &OrthographicProjection)>().with::<&PrimaryCamera>();
		for (_, (transform, projection)) in query.iter() {
			let window_width = self.surface_configuration.width as f32;
			let window_height = self.surface_configuration.height as f32;

			let position = transform.position.extend(0.0);
			let view = Mat4::look_at_rh(position, position + Vec3::Z, Vec3::Y);

			let projection = projection.matrix(window_width, window_height);
			let data = CameraUniform { view_projection: projection * view };

			self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&data));
		}
	}
}
