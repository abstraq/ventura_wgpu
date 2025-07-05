use glam::Mat4;

/// A camera with an orthographic projection matrix for rendering game objects.
///
/// This is the primary camera used for the game.
#[derive(Copy, Clone, Debug)]
pub struct OrthographicCamera {
	pub view: Mat4,
	pub projection: Mat4,
}

impl OrthographicCamera {
	pub fn new(width: f32, height: f32) -> Self {
		let projection = Mat4::orthographic_rh(0.0, width, height, 0.0, 0.0, 1.0);
		let view = Mat4::IDENTITY;

		Self { view, projection }
	}

	pub fn view_projection(&self) -> Mat4 {
		self.projection * self.view
	}
}
