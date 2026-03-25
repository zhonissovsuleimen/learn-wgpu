use wgpu::{CommandEncoder, TextureView};

use crate::app::{gpu_wrapper::GpuWrapper, window_wrapper::WindowWrapper};

pub trait Pass {
  fn run(&mut self, encoder: &mut CommandEncoder, window: &WindowWrapper, gpu: &GpuWrapper, view: &TextureView);
}
