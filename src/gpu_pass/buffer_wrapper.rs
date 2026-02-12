use std::marker::PhantomData;

use wgpu::Buffer;

#[derive(Clone)]
pub struct BufferWrapper<T> {
  pub buffer: Buffer,
  pub count: u32,

  _type_marker: PhantomData<T>,
}

impl<T> BufferWrapper<T> {
  pub fn new(buffer: Buffer, count: u32) -> Self {
    BufferWrapper {
      buffer,
      count,
      _type_marker: PhantomData,
    }
  }
}
