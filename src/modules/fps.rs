use std::time::Instant;

use tracing::info;

use crate::app::module::Module;

pub struct FpsModule {
  last_print: Option<Instant>,
  frame_count: u64,

  cooldown_sec: f64,
}

impl Module for FpsModule {
  fn on_redraw(&mut self) {
    let now = Instant::now();
    self.frame_count += 1;

    match self.last_print {
      Some(last) => {
        let elapsed = (now - last).as_secs_f64();
        if elapsed >= self.cooldown_sec {
          let fps = self.frame_count as f64 / elapsed;
          info!("FPS: {:.1}", fps);
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
      last_print: None,
      frame_count: 0,
      cooldown_sec: 2.0,
    }
  }
}
