use std::time::Instant;

use super::state::State;
use tracing::{error, info};
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

#[derive(Default)]
pub struct App {
  state: Option<State>,
  last_print: Option<Instant>,
  frame_count: u64,
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

        self.frame_count += 1;

        match self.last_print {
          Some(last) => {
            let elapsed = (now - last).as_secs_f64();
            if elapsed >= 1.0 {
              let fps = self.frame_count as f64 / elapsed;
              info!("FPS: {:.1}", fps);
              self.frame_count = 0;
              self.last_print = Some(now);
            }
          }
          None => self.last_print = Some(now),
        }

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
