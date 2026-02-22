use super::state::State;
use crate::app::module::Module;
use tracing::error;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

#[derive(Default)]
pub struct App {
  state: Option<State>,
  modules: Vec<Box<dyn Module>>,
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let state_future = State::new(event_loop);

    // cannot await & return result because of function signature
    match futures::executor::block_on(state_future) {
      Ok(state) => self.state = Some(state),
      Err(e) => panic!("Failed to create app state: {e}"),
    }
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
    let Some(state) = &mut self.state else {
      error!("App state does not exit");
      return;
    };

    match event {
      WindowEvent::CloseRequested => {
        state.request_close(window_id);

        if !state.has_windows() {
          event_loop.exit();
        }
      }
      WindowEvent::RedrawRequested => {
        self.modules.iter_mut().for_each(|module| module.on_redraw());
        state.render(window_id);
        state.request_redraw(window_id);
      }
      WindowEvent::Resized(new_size) => {
        state.resize(window_id, new_size);
      }
      _ => (),
    }
  }
}

impl App {
  pub fn add_module(&mut self, module: Box<dyn Module>) {
    self.modules.push(module);
  }
}
