use wgpu::{CommandEncoder, TextureView};

use crate::app::{gpu_wrapper::GpuWrapper, resources::Resources, window_wrapper::WindowWrapper};

pub trait GpuPass {
  fn run(&mut self, encoder: &mut CommandEncoder, window: &WindowWrapper, gpu: &GpuWrapper, view: &TextureView, resources: &mut Resources);
}
