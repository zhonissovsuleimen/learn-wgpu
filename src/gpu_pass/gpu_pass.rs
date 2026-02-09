use std::collections::HashMap;
use wgpu::{CommandEncoder, TextureView};

use crate::{
  app::{gpu_wrapper::GpuWrapper, window_wrapper::WindowWrapper},
  gpu_pass::buffer_wrapper::BufferWrapper,
};

pub trait GpuPass {
  fn new(gpu: &GpuWrapper, window: &WindowWrapper, buffers: &HashMap<&'static str, BufferWrapper>) -> Self
  where
    Self: Sized;

  fn run(&mut self, encoder: &mut CommandEncoder, view: &TextureView);
}
