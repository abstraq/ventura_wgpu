use glam::Mat4;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, Device, Queue};

pub struct OrthographicCamera {
	pub buffer: Buffer,
	projection: Mat4,
}

impl OrthographicCamera {
	pub fn new(device: &Device, width: u32, height: u32) -> Self {
		let projection = Mat4::orthographic_rh(0.0, width as f32, height as f32, 0.0, 1.0, -1.0);

		let buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Camera Buffer"),
			contents: bytemuck::bytes_of(&projection),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		Self { buffer, projection }
	}

	pub fn resize(&mut self, queue: &Queue, width: u32, height: u32) {
		self.projection = Mat4::orthographic_rh(0.0, width as f32, height as f32, 0.0, 1.0, -1.0);
		queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.projection));
	}
}
