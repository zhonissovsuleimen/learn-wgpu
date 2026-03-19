//repr C needed cause wgsl expects c-like padding (i think), and this is a compiler hint for that
#[repr(C)]
#[derive(Clone)]
pub struct Window {
  pub top_left: [f32; 2],
  pub bottom_right: [f32; 2],
}
