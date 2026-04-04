use bytemuck::{Pod, Zeroable};

use crate::app::window_wrapper::WindowWrapper;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Window {
  pub top_left: [f32; 2],
  pub bottom_right: [f32; 2],
}

impl Window {
  pub fn empty() -> Window {
    Window {
      top_left: [0.0, 0.0],
      bottom_right: [0.0, 0.0],
    }
  }
}

impl From<&WindowWrapper> for Window {
  fn from(wrapper: &WindowWrapper) -> Self {
    let window = &wrapper.window;
    let pos = window.inner_position().expect("Coundn't get inner position");
    let size = window.inner_size();

    let top_left = [pos.x as f32, pos.y as f32];
    let bottom_right = [top_left[0] + size.width as f32, top_left[1] + size.height as f32];
    Window { top_left, bottom_right }
  }
}
