use wgpu::SurfaceError;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::render::RenderContext;

#[derive(Default)]
pub struct VenturaRunner {
	render_context: Option<RenderContext>,
}

impl VenturaRunner {
	pub fn new() -> Self {
		Self { render_context: None }
	}
}
impl ApplicationHandler for VenturaRunner {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window_attributes = Window::default_attributes();
		let window = event_loop
			.create_window(window_attributes)
			.expect("Failed to create window.");

		self.render_context = Some(RenderContext::new(window));
	}

	fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
		match event {
			WindowEvent::CloseRequested => event_loop.exit(),
			WindowEvent::RedrawRequested => {
				if let Some(render_context) = &mut self.render_context {
					match render_context.draw() {
						Err(SurfaceError::Lost | SurfaceError::Outdated) => {
							tracing::warn!("The surface was lost or outdated, reconfiguring.");
							let size = render_context.window.inner_size();
							render_context.resize(size);
						}
						Err(e) => tracing::warn!("Encountered an error while drawing frame {}", e),
						_ => {}
					}
				}
			}
			WindowEvent::Resized(new_size) => {
				if let Some(render_context) = &mut self.render_context {
					render_context.resize(new_size);
				}
			}
			_ => {}
		}
	}
}
