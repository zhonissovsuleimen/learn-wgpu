use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Params {
  pub dt: f32,
  pub window_count: u32,
}

impl Params {
  pub fn new() -> Params {
    Params { dt: 0.0, window_count: 0 }
  }
}
