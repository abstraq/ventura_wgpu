use ventura::application::VenturaApp;
use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};

/// Entry point for the application.
fn main() -> Result<(), EventLoopError> {
	tracing_subscriber::fmt::init();
	let event_loop = EventLoop::new()?;
	event_loop.set_control_flow(ControlFlow::Poll);

	// TODO: Load configuration file and overrides from command line arguments.

	// Start the winit application handler for Ventura.
	let mut app = VenturaApp::new();
	event_loop.run_app(&mut app)
}
