use std::time::Instant;

use super::state::State;
use tracing::{error, info};
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

#[derive(Default)]
pub struct App {
  state: Option<State>,
  last_frame: Option<Instant>,
  last_print: Option<Instant>,
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
        let now = Instant::now();

        match self.last_frame {
          Some(last_frame) => {
            if self.last_print.is_none_or(|last_print| (now - last_print).as_micros() > 1000000) {
              let dif = now - last_frame;
              let fps = 1e6 / dif.as_micros() as f64;
              info!("FPS: {fps}");

              self.last_print = Some(now);
            }
          }
          None => (),
        }
        self.last_frame = Some(now);

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
