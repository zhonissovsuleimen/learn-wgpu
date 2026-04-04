use crate::{
  app::{gpu_wrapper::GpuWrapper, window_wrapper::WindowWrapper},
  particle_sim::{compute_pass::ComputePass, render_pass::RenderPass, window::Window},
};
use wgpu::{
  Buffer, BufferUsages, CommandEncoder, Device, Queue, TextureView,
  util::{BufferInitDescriptor, DeviceExt},
};

pub struct ParticleSim {
  render_window_buffer: Buffer,
  compute_windows_buffer: Buffer,
  compute: ComputePass,
  render: RenderPass,
}

impl ParticleSim {
  pub fn init(gpu: &GpuWrapper, window: &WindowWrapper) -> ParticleSim {
    let compute_windows_buffer = ParticleSim::init_compute_windows_buffer(&gpu.device);
    let compute = ComputePass::init(gpu, &compute_windows_buffer);

    let render_window_buffer = ParticleSim::init_window_buffer(&gpu.device);
    let (particle_buffer, particle_count) = compute.get_particle_buffer();
    let render = RenderPass::init(gpu, window, &render_window_buffer, particle_buffer.clone(), particle_count);

    ParticleSim {
      render_window_buffer,
      compute_windows_buffer,
      compute,
      render,
    }
  }

  pub fn render(&mut self, encoder: &mut CommandEncoder, gpu: &GpuWrapper, window: &WindowWrapper, view: &TextureView) {
    self.update_render_window_buffer(&gpu.queue, window);
    self.render.run(encoder, view);
  }

  pub fn compute(&mut self, encoder: &mut CommandEncoder, gpu: &GpuWrapper, windows: Vec<&WindowWrapper>) {
    let count = windows.len() as u32;
    self.update_compute_window_buffer(&gpu.queue, windows);
    self.compute.run(encoder, gpu, count);
  }

  fn init_window_buffer(device: &Device) -> Buffer {
    device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Window Buffer"),
      contents: bytemuck::bytes_of(&Window::empty()),
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
  }

  fn init_compute_windows_buffer(device: &Device) -> Buffer {
    const MAX: usize = 32;
    let data = vec![Window::empty(); MAX];

    device.create_buffer_init(&BufferInitDescriptor {
      label: Some("All Windows Buffer"),
      contents: bytemuck::cast_slice(&data),
      usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    })
  }

  fn update_render_window_buffer(&mut self, queue: &Queue, window: &WindowWrapper) {
    let Ok(top_left) = window.window.inner_position() else {
      return;
    };
    let size = window.window.inner_size().cast::<f32>();
    let data: [[f32; 2]; 2] = [
      [top_left.x as f32, top_left.y as f32],
      [top_left.x as f32 + size.width, top_left.y as f32 + size.height],
    ];
    queue.write_buffer(&self.render_window_buffer, 0, bytemuck::bytes_of(&data));
  }

  fn update_compute_window_buffer(&mut self, queue: &Queue, windows: Vec<&WindowWrapper>) {
    const MAX: usize = 32;
    if windows.len() > MAX {
      return;
    };

    let windows_data: Vec<Window> = windows.iter().map(|&wrapper| Window::from(wrapper)).collect();
    queue.write_buffer(&self.compute_windows_buffer, 0, bytemuck::cast_slice(&windows_data));
  }
}
