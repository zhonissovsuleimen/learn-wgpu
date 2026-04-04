use std::time::Instant;

use tracing::info;
use winit::window::WindowId;

use crate::app::module::Module;

pub struct FpsModule {
  window_id: Option<WindowId>,
  last_print: Option<Instant>,
  frame_count: u64,

  cooldown_sec: f64,
}

impl Module for FpsModule {
  fn on_render(&mut self, window_id: WindowId) {
    match self.window_id {
      Some(my_id) if my_id != window_id => return,
      None => self.window_id = Some(window_id),
      _ => {}
    }

    let now = Instant::now();
    self.frame_count += 1;

    match self.last_print {
      Some(last) => {
        let elapsed = (now - last).as_secs_f64();
        if elapsed >= self.cooldown_sec {
          let fps = self.frame_count as f64 / elapsed;
          info!("{window_id:?} FPS: {:.1}", fps);
          self.frame_count = 0;
          self.last_print = Some(now);
        }
      }
      None => self.last_print = Some(now),
    }
  }
}

impl Default for FpsModule {
  fn default() -> FpsModule {
    FpsModule {
      window_id: None,
      last_print: None,
      frame_count: 0,
      cooldown_sec: 2.0,
    }
  }
}
