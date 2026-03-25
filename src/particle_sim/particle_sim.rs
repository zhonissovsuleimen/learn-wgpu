use crate::{
  app::{gpu_wrapper::GpuWrapper, pass::Pass, window_wrapper::WindowWrapper},
  particle_sim::{compute_pass::ComputePass, render_pass::RenderPass},
};
use wgpu::{
  Buffer, BufferUsages, CommandEncoder, Device, Queue, TextureView,
  util::{BufferInitDescriptor, DeviceExt},
};

pub struct ParticleSim {
  window_buffer: Buffer,
  compute: ComputePass,
  render: RenderPass,
}

impl ParticleSim {
  pub fn init(gpu: &GpuWrapper, window: &WindowWrapper) -> ParticleSim {
    let window_buffer = ParticleSim::init_window_buffer(&gpu.device);
    let compute = ComputePass::init(gpu, &window_buffer);

    let particle_buffer = compute.get_particle_buffer().clone();
    let render = RenderPass::init(gpu, window, &window_buffer, particle_buffer);

    ParticleSim {
      window_buffer,
      compute,
      render,
    }
  }

  fn init_window_buffer(device: &Device) -> Buffer {
    let data = [[0.0, 0.0], [0.0, 0.0]];

    device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Window Buffer"),
      contents: bytemuck::bytes_of(&data),
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
  }

  fn update_window_buffer(&mut self, queue: &Queue, window: &WindowWrapper) {
    let Ok(top_left) = window.window.inner_position() else {
      return;
    };
    let size = window.window.inner_size().cast::<f32>();
    let data: [[f32; 2]; 2] = [
      [top_left.x as f32, top_left.y as f32],
      [top_left.x as f32 + size.width, top_left.y as f32 + size.height],
    ];
    queue.write_buffer(&self.window_buffer, 0, bytemuck::bytes_of(&data));
  }
}

impl Pass for ParticleSim {
  fn run(&mut self, encoder: &mut CommandEncoder, window: &WindowWrapper, gpu: &GpuWrapper, view: &TextureView) {
    self.update_window_buffer(&gpu.queue, window);
    self.compute.run(encoder, gpu);
    self.render.run(encoder, view);
  }
}
