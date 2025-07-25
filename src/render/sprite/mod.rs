mod instance;
mod pipeline;

use self::instance::SpriteInstanceData;
pub(super) use self::pipeline::SpritePipeline;

/// Component describing a sprite rendered by the sprite pipeline.
pub struct Sprite {}

impl Sprite {
	// TODO: The path will automatically reference an asset from the asset loader.
	pub fn from_image(path: &str) -> Self {
		tracing::info!("Created sprite {path}");
		Self {}
	}

	pub fn from_color(color: wgpu::Color) -> Self {
		Self {}
	}
}
