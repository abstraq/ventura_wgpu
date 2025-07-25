use glam::{Mat4, Vec2};
use hecs::World;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue, RenderPipeline};

use crate::render::sprite::{Sprite, SpriteInstanceData};
use crate::render::texture::Texture;
use crate::render::util::{StorageBinding, TextureBinding, UniformBinding};
use crate::render::vertex::Vertex;
use crate::transform::Transform;

/// The WGSL shader used for rendering sprites.
const SPRITE_SHADER: wgpu::ShaderModuleDescriptor = wgpu::include_wgsl!("./shaders/shader.wgsl");

/// Vertex array containing the unique vertices to create a sprite quad.
const SPRITE_VERTICES: &[Vertex] = &[
	Vertex { position: Vec2::new(-0.5, 0.5), texture_coordinates: Vec2::new(0.0, 0.0) }, // 1: Top Left
	Vertex { position: Vec2::new(-0.5, -0.5), texture_coordinates: Vec2::new(0.0, 1.0) }, // 0: Bottom Left
	Vertex { position: Vec2::new(0.5, -0.5), texture_coordinates: Vec2::new(1.0, 1.0) }, // 3: Bottom Right
	Vertex { position: Vec2::new(0.5, 0.5), texture_coordinates: Vec2::new(1.0, 0.0) },  // 2: Top Right
];

/// Index array to define a quad from the [`SPRITE_VERTICES`] vertex array.
const SPRITE_INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

/// The maximum number of instances that will be rendered per frame.
const MAX_INSTANCES: u64 = 10_000;

/// This pipeline is used for rendering sprites in the scene.
pub struct SpritePipeline {
	pipeline: RenderPipeline,
	vertex_buffer: Buffer,
	index_buffer: Buffer,
	instance_binding: StorageBinding<SpriteInstanceData>,
	instance_count: u32,
	texture_binding: TextureBinding,
}

impl SpritePipeline {
	pub fn new(device: &Device, queue: &Queue, camera_bind_layout: &BindGroupLayout) -> Self {
		let shader = device.create_shader_module(SPRITE_SHADER);

		let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Sprite Vertex Buffer"),
			usage: wgpu::BufferUsages::VERTEX,
			contents: bytemuck::cast_slice(SPRITE_VERTICES),
		});

		let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Sprite Index Buffer"),
			usage: wgpu::BufferUsages::INDEX,
			contents: bytemuck::cast_slice(SPRITE_INDICES),
		});

		let instance_binding = StorageBinding::create(device, Some("Sprite Instance Buffer".into()), MAX_INSTANCES);

		let texture = Texture::from_bytes(device, queue, include_bytes!("../../../assets/test_sprite.png")).unwrap();
		let texture_binding = TextureBinding::create(device, Some("Test Texture".into()), texture);

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Sprite Pipeline Layout"),
			bind_group_layouts: &[
				camera_bind_layout,
				instance_binding.bind_group_layout(),
				texture_binding.bind_group_layout(),
			],
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
					array_stride: size_of::<Vertex>() as u64,
					step_mode: wgpu::VertexStepMode::Vertex,
					attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
				}],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				compilation_options: Default::default(),
				entry_point: Some("fs_main"),
				targets: &[Some(wgpu::ColorTargetState {
					format: wgpu::TextureFormat::Bgra8UnormSrgb,
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

		Self { pipeline, vertex_buffer, index_buffer, instance_binding, instance_count: 0, texture_binding }
	}

	pub fn prepare(&mut self, queue: &Queue, world: &mut World) {
		let mut instances = Vec::new();
		let mut query = world.query::<(&Transform, &Sprite)>();
		for (_, (transform, sprite)) in query.iter() {
			let instance = SpriteInstanceData::new(transform.matrix());
			instances.push(instance);
		}

		queue.write_buffer(self.instance_binding.buffer(), 0, bytemuck::cast_slice(&instances[..]));
		self.instance_count = instances.len() as u32;
	}

	pub fn draw(&self, render_pass: &mut wgpu::RenderPass, camera_bind_group: &BindGroup) {
		render_pass.set_pipeline(&self.pipeline);
		render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
		render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
		render_pass.set_bind_group(0, camera_bind_group, &[]);
		render_pass.set_bind_group(1, self.instance_binding.bind_group(), &[]);
		render_pass.set_bind_group(2, self.texture_binding.bind_group(), &[]);
		render_pass.draw_indexed(0..6, 0, 0..self.instance_count);
	}
}
