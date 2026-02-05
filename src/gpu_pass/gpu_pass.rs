use std::collections::HashMap;
use wgpu::{Buffer, CommandEncoder, TextureView};

use crate::app::{gpu_wrapper::GpuWrapper, window_wrapper::WindowWrapper};

pub trait GpuPass {
  fn new(gpu: &GpuWrapper, window: &WindowWrapper, buffers: &HashMap<&'static str, Buffer>) -> Self
  where
    Self: Sized;
  fn run(&mut self, encoder: &mut CommandEncoder, view: &TextureView);
}
