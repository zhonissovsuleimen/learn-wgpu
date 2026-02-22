use tracing::Level;
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod gpu_pass;
mod modules;
mod particle_sim;

use app::app::App;

use crate::modules::fps::FpsModule;

#[tokio::main]
async fn main() {
  LogTracer::init().expect("failed to set logger");
  let subscriber = FmtSubscriber::builder().with_max_level(Level::INFO).finish();

  tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");

  let event_loop = EventLoop::new().unwrap();
  event_loop.set_control_flow(ControlFlow::Poll);

  let mut app = App::default();
  app.add_module(Box::new(FpsModule::default()));

  event_loop.run_app(&mut app).unwrap();
}
