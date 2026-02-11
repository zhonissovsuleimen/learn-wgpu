use std::collections::HashMap;
use wgpu::{CommandEncoder, TextureView};

use crate::{
  app::{gpu_wrapper::GpuWrapper, window_wrapper::WindowWrapper},
  gpu_pass::buffer_wrapper::BufferWrapper,
};

pub trait GpuPass {
  fn run(
    &mut self,
    encoder: &mut CommandEncoder,
    window: &WindowWrapper,
    gpu: &GpuWrapper,
    view: &TextureView,
    buffers: &mut HashMap<&'static str, BufferWrapper>,
  );
}
