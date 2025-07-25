use bytemuck::{Pod, Zeroable};
use glam::Mat4;

/// Marker component representing the primary camera used for the game world.
#[derive(Copy, Clone, Debug)]
pub struct PrimaryCamera;

/// Marker component tagging an entity that the camera focus on.
pub struct CameraTarget;

/// This component describes the projection matrix to use on entities with the
/// [`PrimaryCamera`] component.
#[derive(Copy, Clone, Debug)]
pub struct OrthographicProjection;

impl OrthographicProjection {
	pub fn matrix(&self, window_width: f32, window_height: f32) -> Mat4 {
		let half_width = window_width / 2.0;
		let half_height = window_height / 2.0;
		Mat4::orthographic_rh(-half_width, half_width, -half_height, half_height, -1.0, 1.0)
	}
}

/// Contains the view-projection matrix for the [`PrimaryCamera`] that is used
/// to convert the world-space coordinates into clip-space coordinates for
/// rendering.
///
/// This is uploaded to the GPU each frame
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub(super) struct CameraUniform {
	pub view_projection: Mat4,
}
