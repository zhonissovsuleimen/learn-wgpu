use super::window_wrapper::WindowWrapper;
use crate::{
  app::{gpu_wrapper::GpuWrapper, window_wrapper::WindowWrapperError},
  particle_sim::particle_sim::ParticleSim,
};
use std::collections::HashMap;
use tracing::{error, info};
use wgpu::{
  CommandEncoderDescriptor, DeviceDescriptor, Instance, InstanceDescriptor, RequestAdapterError, RequestAdapterOptions, RequestDeviceError,
  TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop, window::WindowId};

pub struct State {
  gpu: GpuWrapper,
  windows: HashMap<WindowId, WindowWrapper>,
  sim: Option<ParticleSim>,
}

impl State {
  pub async fn new(event_loop: &ActiveEventLoop) -> Result<State, StateError> {
    let instance_desc = InstanceDescriptor::default();
    let instance = Instance::new(&instance_desc);

    let adapter_opts = RequestAdapterOptions::default();
    let adapter = instance.request_adapter(&adapter_opts).await?;

    let device_desc = DeviceDescriptor::default();
    let (device, queue) = adapter.request_device(&device_desc).await?;

    let gpu = GpuWrapper {
      instance,
      adapter,
      device,
      queue,
    };

    let mut state = State {
      gpu,
      windows: HashMap::new(),
      sim: None,
    };

    let id = state.add_window(event_loop).await?;
    state.request_redraw(id);

    let window = state.windows.get(&id).unwrap();
    state.sim = Some(ParticleSim::init(&state.gpu, window));

    Ok(state)
  }

  pub async fn add_window(&mut self, event_loop: &ActiveEventLoop) -> Result<WindowId, StateError> {
    let window_wrapper = WindowWrapper::new(&self.gpu, event_loop).await?;
    let key = window_wrapper.window.id();
    info!("Window {key:?} created");

    self.windows.insert(key, window_wrapper);
    Ok(key)
  }

  pub fn has_windows(&self) -> bool {
    self.windows.len() > 0
  }

  pub fn request_redraw(&self, window_id: WindowId) {
    if let Some(wrapper) = &self.windows.get(&window_id) {
      wrapper.window.request_redraw();
    }
  }

  pub fn request_close(&mut self, window_id: WindowId) {
    self.windows.remove(&window_id);
  }

  pub fn render(&mut self, window_id: WindowId) {
    match self.windows.get(&window_id) {
      Some(window_wrapper) => {
        let Ok(texture) = window_wrapper.surface.get_current_texture() else {
          error!("Failed to aquire next swapchain texture");
          return;
        };

        let view = texture.texture.create_view(&TextureViewDescriptor {
          format: Some(window_wrapper.surface_config.format),
          ..Default::default()
        });

        let mut command_encoder = self.gpu.device.create_command_encoder(&CommandEncoderDescriptor::default());
        if let Some(sim) = self.sim.as_mut() {
          sim.render(&mut command_encoder, &self.gpu, window_wrapper, &view);
        }

        self.gpu.queue.submit(Some(command_encoder.finish()));
        texture.present();
      }
      None => {
        let msg = format!("Failed find window with id {:?}", window_id);
        error!(msg);
      }
    }
  }

  pub fn compute(&mut self) {
    let mut command_encoder = self.gpu.device.create_command_encoder(&CommandEncoderDescriptor::default());
    if let Some(sim) = self.sim.as_mut() {
      sim.compute(&mut command_encoder, &self.gpu, self.windows.values().collect());
    }

    self.gpu.queue.submit(Some(command_encoder.finish()));
  }

  pub fn resize(&mut self, window_id: WindowId, new_size: PhysicalSize<u32>) {
    match self.windows.get_mut(&window_id) {
      Some(window_wrapper) => window_wrapper.resize(new_size, &self.gpu.device),
      None => {
        let msg = format!("Failed find window with id {:?}", window_id);
        error!(msg);
      }
    }
  }
}

#[derive(thiserror::Error, Debug)]
pub enum StateError {
  #[error("Request adapter error: {0}")]
  RequestAdapterError(#[from] RequestAdapterError),

  #[error("Request device error: {0}")]
  RequestDeviceError(#[from] RequestDeviceError),

  #[error("App window error: {0}")]
  AppWindowError(#[from] WindowWrapperError),
}
