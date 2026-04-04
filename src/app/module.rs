use winit::window::WindowId;

pub trait Module {
  fn on_render(&mut self, window_id: WindowId) {}
}
