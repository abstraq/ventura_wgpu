use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};

/// Data that is passed to the vertex shader.
///
/// `clip_position` represents the position of the vertex in model space.
/// `texture_coordinates` are passed to the sampler.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct VertexData {
	pub clip_position: Vec3,
	pub texture_coordinates: Vec2,
}
