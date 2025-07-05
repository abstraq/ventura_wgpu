use ventura::runner::VenturaRunner;
use winit::error::EventLoopError;
use winit::event_loop::EventLoop;

fn main() -> Result<(), EventLoopError> {
	tracing_subscriber::fmt::init();
	let event_loop = EventLoop::new()?;
	let mut runner = VenturaRunner::default();

	event_loop.run_app(&mut runner)
}
