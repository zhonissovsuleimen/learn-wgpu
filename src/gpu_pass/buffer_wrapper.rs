use wgpu::Buffer;

#[derive(Clone)]
pub struct BufferWrapper {
  pub buffer: Buffer,
  pub count: u32,
}
