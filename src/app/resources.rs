use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::gpu_pass::buffer_wrapper::BufferWrapper;

pub struct Resources {
  buffers: HashMap<(TypeId, &'static str), Box<dyn Any>>,
}

impl Resources {
  pub fn new() -> Resources {
    Resources { buffers: HashMap::new() }
  }

  pub fn insert<T: 'static>(&mut self, name: &'static str, buffer: BufferWrapper<T>) {
    self.buffers.insert((TypeId::of::<T>(), name), Box::new(buffer));
  }

  pub fn get<T: 'static>(&self, name: &'static str) -> Option<&BufferWrapper<T>> {
    self
      .buffers
      .get(&(TypeId::of::<T>(), name))
      .and_then(|b| b.downcast_ref::<BufferWrapper<T>>())
  }
}
