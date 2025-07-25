use glam::{Mat4, Vec2};

/// This component stores the entity's location and orientation in the world
/// space.
#[derive(Copy, Clone, Debug)]
pub struct Transform {
	pub position: Vec2,
	pub scale: Vec2,
	pub rotation: f32,
}

impl Transform {
	pub fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self {
		Self { position, rotation, scale }
	}

	pub fn from_translation(x: f32, y: f32) -> Self {
		Self { position: Vec2::new(x, y), ..Default::default() }
	}

	/// Convert the transform component into a 4x4 matrix.
	pub fn matrix(&self) -> Mat4 {
		let translation = Mat4::from_translation(self.position.extend(0.0));
		let rotation = Mat4::from_rotation_z(self.rotation);
		let scale = Mat4::from_scale(self.scale.extend(1.0));

		translation * rotation * scale
	}
}

impl Default for Transform {
	fn default() -> Self {
		Self { position: Vec2::ZERO, scale: Vec2::splat(32.0), rotation: 0.0 }
	}
}

impl From<Transform> for Mat4 {
	fn from(value: Transform) -> Self {
		value.matrix()
	}
}
