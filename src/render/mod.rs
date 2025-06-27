mod camera;
mod vertex;

use std::sync::Arc;

use glam::{Vec2, Vec3};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Instance, SurfaceConfiguration, SurfaceError};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::render::camera::OrthographicCamera;
use crate::render::vertex::Vertex;

pub struct RenderContext {
	surface: wgpu::Surface<'static>,
	device: wgpu::Device,
	queue: wgpu::Queue,
	surface_config: wgpu::SurfaceConfiguration,
	camera: OrthographicCamera,
	camera_bind_group: wgpu::BindGroup,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	render_pipeline: wgpu::RenderPipeline,
	pub window: Arc<Window>,
}

const VERTEX_ARRAY: &[Vertex] = &[
	Vertex { position: Vec2::new(0.0, 0.0), color: Vec3::new(1.0, 0.0, 0.0) },
	Vertex { position: Vec2::new(0.0, 48.0), color: Vec3::new(0.0, 1.0, 0.0) },
	Vertex { position: Vec2::new(48.0, 48.0), color: Vec3::new(0.0, 0.0, 1.0) },
	Vertex { position: Vec2::new(48.0, 0.0), color: Vec3::new(0.0, 0.0, 1.0) },
];

const INDEX_ARRAY: &[u16; 6] = &[0, 1, 2, 2, 3, 0];

impl RenderContext {
	pub fn init(window: Window) -> Self {
		let window = Arc::new(window);
		let window_size = window.inner_size();

		let instance = Instance::new(&wgpu::InstanceDescriptor {
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

		let camera = OrthographicCamera::new(&device, window_size.width, window_size.height);

		let camera_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: camera.buffer.as_entire_binding(),
			}],
		});

		let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Vertex Buffer"),
			contents: bytemuck::cast_slice(VERTEX_ARRAY),
			usage: wgpu::BufferUsages::VERTEX,
		});

		let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Index Buffer"),
			contents: bytemuck::cast_slice(INDEX_ARRAY),
			usage: wgpu::BufferUsages::INDEX,
		});

		let shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/shader.wgsl"));

		let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[&camera_bind_group_layout],
			push_constant_ranges: &[],
		});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				compilation_options: Default::default(),
				entry_point: None,
				buffers: &[wgpu::VertexBufferLayout {
					array_stride: size_of::<Vertex>() as u64,
					step_mode: wgpu::VertexStepMode::Vertex,
					attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3],
				}],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				compilation_options: Default::default(),
				entry_point: None,
				targets: &[Some(wgpu::ColorTargetState {
					format: surface_config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				})],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				front_face: wgpu::FrontFace::Ccw,
				polygon_mode: wgpu::PolygonMode::Fill,
				cull_mode: Some(wgpu::Face::Back),
				strip_index_format: None,
				unclipped_depth: false,
				conservative: false,
			},
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
			depth_stencil: None,
			cache: None,
		});

		surface.configure(&device, &surface_config);

		Self {
			surface,
			device,
			queue,
			surface_config,
			camera,
			camera_bind_group,
			vertex_buffer,
			index_buffer,
			render_pipeline,
			window,
		}
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
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &texture_view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.01, g: 0.01, b: 0.01, a: 1.0 }),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});

			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
			render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
			render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
			render_pass.draw_indexed(0..6, 0, 0..1);
		}

		self.queue.submit([encoder.finish()]);
		output.present();

		Ok(())
	}

	pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
		let width = new_size.width;
		let height = new_size.height;
		if width > 0 && height > 0 {
			self.surface_config.width = width;
			self.surface_config.height = height;
			self.surface.configure(&self.device, &self.surface_config);
			self.camera.resize(&self.queue, width, height);
		}
	}
}
