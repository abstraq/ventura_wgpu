use glam::{Mat4, Vec2, Vec3};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroup, Buffer, Device, RenderPipeline};

use crate::render::{Texture, VertexData};

const QUAD_VERTEX_ARRAY: &[VertexData] = &[
	VertexData { clip_position: Vec3::new(0.0, 0.0, 0.0), texture_coordinates: Vec2::new(0.0, 0.0) },
	VertexData { clip_position: Vec3::new(0.0, 1.0, 0.0), texture_coordinates: Vec2::new(0.0, 1.0) },
	VertexData { clip_position: Vec3::new(1.0, 1.0, 0.0), texture_coordinates: Vec2::new(1.0, 1.0) },
	VertexData { clip_position: Vec3::new(1.0, 0.0, 0.0), texture_coordinates: Vec2::new(1.0, 0.0) },
];

const QUAD_INDEX_ARRAY: &[u16] = &[0, 1, 2, 2, 3, 0];

/// This pipeline is used for rendering sprites in the scene.
pub struct SpritePipeline {
	pipeline: RenderPipeline,
	vertex_buffer: Buffer,
	index_buffer: Buffer,
	transform_bind_group: BindGroup,
	texture_bind_group: BindGroup,
}

impl SpritePipeline {
	pub fn new(
		device: &Device,
		texture: &Texture,
		surface_configuration: &wgpu::SurfaceConfiguration,
		camera_bind_group_layout: &wgpu::BindGroupLayout,
	) -> Self {
		let shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/shader.wgsl"));

		let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Sprite Texture Bind Group Layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2,
						multisampled: false,
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
			],
		});

		let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Sprite Texture Bind Group"),
			layout: &texture_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&texture.view) },
				wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&texture.sampler) },
			],
		});

		let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Sprite Vertex Buffer"),
			contents: bytemuck::cast_slice(QUAD_VERTEX_ARRAY),
			usage: wgpu::BufferUsages::VERTEX,
		});

		let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Sprite Index Buffer"),
			contents: bytemuck::cast_slice(QUAD_INDEX_ARRAY),
			usage: wgpu::BufferUsages::INDEX,
		});

		let transform_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Sprite Transform Uniform Buffer"),
			contents: bytemuck::bytes_of(&Mat4::IDENTITY),
			usage: wgpu::BufferUsages::UNIFORM,
		});

		let transform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Sprite Transform Uniform Bind Group Layout"),
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

		let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Sprite Transform Uniform Bind Group"),
			layout: &transform_bind_group_layout,
			entries: &[wgpu::BindGroupEntry { binding: 0, resource: transform_buffer.as_entire_binding() }],
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Sprite Pipeline Layout"),
			bind_group_layouts: &[camera_bind_group_layout, &transform_bind_group_layout, &texture_bind_group_layout],
			push_constant_ranges: &[],
		});

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Sprite Pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				compilation_options: Default::default(),
				entry_point: Some("vs_main"),
				buffers: &[wgpu::VertexBufferLayout {
					array_stride: size_of::<VertexData>() as u64,
					step_mode: wgpu::VertexStepMode::Vertex,
					attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2],
				}],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				compilation_options: Default::default(),
				entry_point: Some("fs_main"),
				targets: &[Some(wgpu::ColorTargetState {
					format: surface_configuration.format,
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
			multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
			multiview: None,
			depth_stencil: None,
			cache: None,
		});

		Self { pipeline, vertex_buffer, index_buffer, texture_bind_group, transform_bind_group }
	}

	pub fn draw(&self, render_pass: &mut wgpu::RenderPass, camera_bind_group: &wgpu::BindGroup) {
		render_pass.set_pipeline(&self.pipeline);
		render_pass.set_bind_group(0, camera_bind_group, &[]);
		render_pass.set_bind_group(1, &self.transform_bind_group, &[]);
		render_pass.set_bind_group(2, &self.texture_bind_group, &[]);
		render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
		render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
		render_pass.draw_indexed(0..6, 0, 0..1);
	}
}
