use tracing::Level;
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod gpu_pass;
use app::app::App;

#[tokio::main]
async fn main() {
  LogTracer::init().expect("failed to set logger");
  let subscriber = FmtSubscriber::builder().with_max_level(Level::INFO).finish();

  tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");

  let event_loop = EventLoop::new().unwrap();
  event_loop.set_control_flow(ControlFlow::Poll);

  let mut app = App::default();
  event_loop.run_app(&mut app).unwrap();
}
