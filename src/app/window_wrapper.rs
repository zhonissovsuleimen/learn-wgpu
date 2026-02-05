use std::sync::Arc;

use wgpu::{CompositeAlphaMode, CreateSurfaceError, Device, PresentMode, Surface, SurfaceConfiguration, TextureFormat, TextureUsages};
use winit::{
  dpi::PhysicalSize,
  error::OsError,
  event_loop::ActiveEventLoop,
  window::{Window, WindowAttributes},
};

use crate::app::gpu_wrapper::GpuWrapper;

pub struct WindowWrapper {
  pub window: Arc<Window>,
  pub surface: Surface<'static>,
  pub surface_config: SurfaceConfiguration,
}

impl WindowWrapper {
  pub async fn new(gpu: &GpuWrapper, event_loop: &ActiveEventLoop) -> Result<WindowWrapper, WindowWrapperError> {
    let window_attr = WindowAttributes::default();

    let window = event_loop.create_window(window_attr)?;
    let window = Arc::new(window);

    let surface = gpu.instance.create_surface(window.clone())?;

    let surface_config = SurfaceConfiguration {
      usage: TextureUsages::RENDER_ATTACHMENT,
      format: TextureFormat::Rgba8UnormSrgb,
      view_formats: vec![TextureFormat::Rgba8UnormSrgb],
      alpha_mode: CompositeAlphaMode::Auto,
      width: window.inner_size().width,
      height: window.inner_size().height,
      desired_maximum_frame_latency: 2,
      present_mode: PresentMode::AutoVsync,
    };

    surface.configure(&gpu.device, &surface_config);

    Ok(WindowWrapper {
      window,
      surface,
      surface_config,
    })
  }

  pub fn resize(&mut self, new_size: PhysicalSize<u32>, device: &Device) {
    let (width, height) = new_size.into();
    self.surface_config.width = width;
    self.surface_config.height = height;

    self.surface.configure(device, &self.surface_config);
  }
}

#[derive(thiserror::Error, Debug)]
pub enum WindowWrapperError {
  #[error("OS error: {0}")]
  OsError(#[from] OsError),
  #[error("Create surface error: {0}")]
  CreateSurfaceError(#[from] CreateSurfaceError),
}
