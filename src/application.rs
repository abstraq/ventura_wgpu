use std::sync::Arc;
use std::time::{Duration, Instant};

use glam::Vec2;
use hecs::World;
use wgpu::SurfaceError;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::render::camera::{OrthographicProjection, PrimaryCamera};
use crate::render::{RenderContext, Sprite};
use crate::transform::Transform;

pub struct VenturaApp {
	world: World,
	render_context: Option<RenderContext>,
	primary_window: Option<Arc<Window>>,
	last_frame: Instant,
}

impl VenturaApp {
	pub fn new() -> Self {
		Self { world: World::new(), render_context: None, primary_window: None, last_frame: Instant::now() }
	}

	fn run_startup_systems(&mut self) {
		self.world.spawn((PrimaryCamera, OrthographicProjection, Transform::new(Vec2::ZERO, 0.0, Vec2::ONE)));
		self.world.spawn((Sprite::from_image("sprite_1"), Transform::from_translation(20.0, 20.0)));
		self.world.spawn((Sprite::from_image("sprite_2"), Transform::from_translation(100.0, 20.0)));
		self.world.spawn((Sprite::from_image("sprite_3"), Transform::from_translation(300.0, 20.0)));
	}

	fn run_update_systems(&mut self, frame_delta: Duration) {}
}

impl Default for VenturaApp {
	fn default() -> Self {
		Self::new()
	}
}

impl ApplicationHandler for VenturaApp {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window_attributes = Window::default_attributes();
		let window_builder = event_loop.create_window(window_attributes).expect("Failed to create window.");

		let window = Arc::new(window_builder);
		let render_context = RenderContext::new(window.clone());

		self.primary_window = Some(window);
		self.render_context = Some(render_context);
		self.last_frame = Instant::now();
		self.run_startup_systems();
	}

	fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
		match event {
			WindowEvent::CloseRequested => event_loop.exit(),
			WindowEvent::RedrawRequested => {
				let current_time = Instant::now();
				let frame_delta = current_time - self.last_frame;
				self.last_frame = current_time;

				self.run_update_systems(frame_delta);

				let window = self.primary_window.as_ref().unwrap();
				window.request_redraw();

				if let Some(render_context) = &mut self.render_context {
					let window_size = window.inner_size();
					match render_context.render(&mut self.world) {
						Err(SurfaceError::Lost | SurfaceError::Outdated) => {
							tracing::warn!("The surface was lost or outdated, reconfiguring.");
							render_context.resize(window_size.width, window_size.height);
						}
						Err(e) => tracing::warn!("Encountered an error while drawing frame {}", e),
						_ => {}
					}
				}
			}
			WindowEvent::Resized(new_size) => {
				if let Some(render_context) = &mut self.render_context {
					render_context.resize(new_size.width, new_size.height);
				}
			}
			_ => {}
		}
	}
}
