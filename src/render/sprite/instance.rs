use bytemuck::{Pod, Zeroable};
use glam::Mat4;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug)]
pub struct SpriteInstanceData {
	transform: Mat4,
}

impl SpriteInstanceData {
	pub fn new(transform: Mat4) -> Self {
		Self { transform }
	}
}
