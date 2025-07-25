use bytemuck::{Pod, Zeroable};
use glam::Vec2;

/// Represents vertex information that will be uploaded to the GPU.
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct Vertex {
	pub position: Vec2,
	pub texture_coordinates: Vec2,
}
