use wgpu::{Buffer, VertexBufferLayout};

pub struct BufferWrapper {
  pub buffer: Buffer,
  pub layout: VertexBufferLayout<'static>,
  pub count: u32,
}
