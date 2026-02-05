use wgpu::{Adapter, Device, Instance, Queue};

pub struct GpuWrapper {
  pub instance: Instance,
  pub adapter: Adapter,
  pub device: Device,
  pub queue: Queue,
}

impl<'a> From<&'a GpuWrapper> for (&'a Instance, &'a Adapter, &'a Device, &'a Queue) {
  fn from(gpu: &'a GpuWrapper) -> Self {
    (&gpu.instance, &gpu.adapter, &gpu.device, &gpu.queue)
  }
}
