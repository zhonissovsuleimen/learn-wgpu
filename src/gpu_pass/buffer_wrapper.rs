use wgpu::{Buffer, VertexBufferLayout};

#[derive(Clone)]
pub struct BufferWrapper {
  pub buffer: Buffer,
  pub layout: VertexBufferLayout<'static>,
  pub count: u32,
}
