use std::marker::PhantomData;

use wgpu::{BindGroup, BindGroupLayout, Buffer, Device};

use crate::render::texture::Texture;

/// Wrapper around a WGPU storage buffer.
///
/// This wrapper is responsible for storing the [`wgpu::Buffer`] that contains
/// data of type `T`, and the respective [`wgpu::BindGroup`] and
/// [`wgpu::BindGroupLayout`] for interacting with the buffer.
pub struct StorageBinding<T: Sized> {
	buffer: Buffer,
	bind_group_layout: BindGroupLayout,
	bind_group: BindGroup,
	_marker: PhantomData<T>,
}

impl<T> StorageBinding<T> {
	pub fn create(device: &Device, buffer_label: Option<String>, max_elements: u64) -> Self {
		let buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: buffer_label.as_deref(),
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
			size: max_elements * size_of::<T>() as u64,
			mapped_at_creation: false,
		});

		let bind_group_layout_label = buffer_label.as_ref().map(|label| format!("{label} Bind Group Layout"));
		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: bind_group_layout_label.as_deref(),
			entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Storage { read_only: true },
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
		});

		let bind_group_label = buffer_label.as_ref().map(|label| format!("{label} Bind Group"));
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: bind_group_label.as_deref(),
			entries: &[wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }],
			layout: &bind_group_layout,
		});

		Self { buffer, bind_group_layout, bind_group, _marker: PhantomData }
	}

	pub fn buffer(&self) -> &Buffer {
		&self.buffer
	}

	pub fn bind_group_layout(&self) -> &BindGroupLayout {
		&self.bind_group_layout
	}

	pub fn bind_group(&self) -> &BindGroup {
		&self.bind_group
	}
}

/// Wrapper around a WGPU uniform buffer.
///
/// This wrapper is responsible for storing the [`wgpu::Buffer`] that contains
/// data of type `T`, and the respective [`wgpu::BindGroup`] and
/// [`wgpu::BindGroupLayout`] for interacting with the buffer.
pub struct UniformBinding<T: Sized> {
	buffer: Buffer,
	bind_group_layout: BindGroupLayout,
	bind_group: BindGroup,
	_marker: PhantomData<T>,
}

impl<T> UniformBinding<T> {
	pub fn create(device: &Device, buffer_label: Option<String>) -> Self {
		let buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: buffer_label.as_deref(),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: size_of::<T>() as u64,
			mapped_at_creation: false,
		});

		let bind_group_layout_label = buffer_label.as_ref().map(|label| format!("{label} Bind Group Layout"));
		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: bind_group_layout_label.as_deref(),
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

		let bind_group_label = buffer_label.as_ref().map(|label| format!("{label} Bind Group"));
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: bind_group_label.as_deref(),
			entries: &[wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }],
			layout: &bind_group_layout,
		});

		Self { buffer, bind_group_layout, bind_group, _marker: PhantomData }
	}

	pub fn buffer(&self) -> &Buffer {
		&self.buffer
	}

	pub fn bind_group_layout(&self) -> &BindGroupLayout {
		&self.bind_group_layout
	}

	pub fn bind_group(&self) -> &BindGroup {
		&self.bind_group
	}
}

pub struct TextureBinding {
	texture: Texture,
	bind_group_layout: BindGroupLayout,
	bind_group: BindGroup,
}

impl TextureBinding {
	pub fn create(device: &Device, texture_label: Option<String>, texture: Texture) -> Self {
		let bind_group_layout_label = texture_label.as_ref().map(|label| format!("{label} Bind Group Layout"));
		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
			label: bind_group_layout_label.as_deref(),
		});

		let bind_group_label = texture_label.as_ref().map(|label| format!("{label} Bind Group"));
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&texture.view) },
				wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&texture.sampler) },
			],
			label: bind_group_label.as_deref(),
		});

		Self { texture, bind_group_layout, bind_group }
	}

	pub fn texture(&self) -> &Texture {
		&self.texture
	}

	pub fn bind_group_layout(&self) -> &BindGroupLayout {
		&self.bind_group_layout
	}

	pub fn bind_group(&self) -> &BindGroup {
		&self.bind_group
	}
}
