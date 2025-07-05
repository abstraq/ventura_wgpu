use image::{GenericImageView, ImageError};
use wgpu::{Device, Queue, Sampler, Texture as WGPUTexture, TextureView};

pub struct Texture {
	raw_texture: WGPUTexture,
	pub view: TextureView,
	pub sampler: Sampler,
}

impl Texture {
	pub fn from_bytes(device: &Device, queue: &Queue, bytes: &[u8]) -> Result<Self, ImageError> {
		let texture_image = image::load_from_memory(bytes)?;
		let texture_image_rgba = texture_image.to_rgba8();
		let texture_image_dimensions = texture_image.dimensions();

		let texture_size = wgpu::Extent3d {
			width: texture_image_dimensions.0,
			height: texture_image_dimensions.1,
			depth_or_array_layers: 1,
		};

		let raw_texture = device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Diffuse Texture"),
			size: texture_size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			view_formats: &[],
		});

		queue.write_texture(
			wgpu::TexelCopyTextureInfo {
				texture: &raw_texture,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
				mip_level: 0,
			},
			&texture_image_rgba,
			wgpu::TexelCopyBufferLayout {
				offset: 0,
				bytes_per_row: Some(4 * texture_image_dimensions.0),
				rows_per_image: Some(texture_image_dimensions.1),
			},
			texture_size,
		);

		let view = raw_texture.create_view(&wgpu::TextureViewDescriptor::default());
		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		Ok(Self { raw_texture, view, sampler })
	}
}
