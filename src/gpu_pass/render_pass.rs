use crate::{
  app::{gpu_wrapper::GpuWrapper, window_wrapper::WindowWrapper},
  gpu_pass::gpu_pass::GpuPass,
};
use std::collections::HashMap;
use tracing::error;
use wgpu::{
  Buffer, BufferUsages, Color, CommandEncoder, FragmentState, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PrimitiveState,
  RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, StoreOp, TextureView, VertexBufferLayout, VertexState,
  VertexStepMode, include_wgsl,
  util::{BufferInitDescriptor, DeviceExt},
  vertex_attr_array,
};

pub struct RenderPass {
  vertex_buffer: Buffer,
  particle_buffer: Buffer,
  pipeline: RenderPipeline,
  //temp
  count: u32,
}

impl GpuPass for RenderPass {
  fn new(gpu: &GpuWrapper, window: &WindowWrapper, buffers: &HashMap<&'static str, Buffer>) -> RenderPass {
    let device = &gpu.device;

    let shader = device.create_shader_module(include_wgsl!("shaders/draw.wgsl"));

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Render pipeline layout"),
      bind_group_layouts: &[],
      immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Render pipeline"),
      layout: Some(&pipeline_layout),
      vertex: VertexState {
        module: &shader,
        entry_point: Some("main_vs"),
        compilation_options: Default::default(),
        buffers: &[
          VertexBufferLayout {
            array_stride: 2 * 2 * 4, // x_pos, y_pos, x_vel, y_vel; 4 bytes each
            step_mode: VertexStepMode::Instance,
            attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x2],
          },
          VertexBufferLayout {
            array_stride: 2 * 4, // single vertex x & y positions; 4 bytes each
            step_mode: VertexStepMode::Vertex,
            attributes: &vertex_attr_array![2 => Float32x2],
          },
        ],
      },
      fragment: Some(FragmentState {
        module: &shader,
        entry_point: Some("main_fs"),
        compilation_options: Default::default(),
        targets: &[Some(window.surface_config.view_formats[0].into())],
      }),
      primitive: PrimitiveState::default(),
      depth_stencil: None,
      multisample: MultisampleState::default(),
      multiview_mask: None,
      cache: None,
    });

    #[rustfmt::skip]
    let vertex_buffer_data: [f32; 6] = [
      -0.01, -0.02,
      0.01, -0.02,
      0.00, 0.02
    ];

    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::bytes_of(&vertex_buffer_data),
      usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
    });

    //temp
    let count = 100;
    let particle_buffer = match buffers.get("test particle buffer") {
      Some(particle_buffer) => particle_buffer.clone(),
      None => {
        error!("Could not find particle buffer");

        let mut fallback_data = vec![0.0f32; (4 * count) as usize];
        for i in 0..(count as usize) {
          let pos_range = -1.0..1.0;
          let vel_range = -0.1..0.1;
          fallback_data[i] = rand::random_range(pos_range.clone());
          fallback_data[i + 1] = rand::random_range(pos_range.clone());
          fallback_data[i + 2] = rand::random_range(vel_range.clone());
          fallback_data[i + 3] = rand::random_range(vel_range.clone());
        }

        device.create_buffer_init(&BufferInitDescriptor {
          label: Some("Empty particle buffer"),
          contents: &bytemuck::cast_slice(&fallback_data),
          usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        })
      }
    };

    RenderPass {
      vertex_buffer,
      particle_buffer,
      pipeline,
      count,
    }
  }

  fn run(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
    let color_attachments = [Some(RenderPassColorAttachment {
      view: &view,
      depth_slice: None,
      resolve_target: None,
      ops: Operations {
        load: LoadOp::Clear(Color::BLACK),
        store: StoreOp::Store,
      },
    })];

    let render_pass_descriptor = RenderPassDescriptor {
      label: Some("Render pipeline descriptor"),
      color_attachments: &color_attachments,
      depth_stencil_attachment: None,
      timestamp_writes: None,
      occlusion_query_set: None,
      multiview_mask: None,
    };

    let mut rpass = encoder.begin_render_pass(&render_pass_descriptor);
    rpass.set_pipeline(&self.pipeline);
    rpass.set_vertex_buffer(0, self.particle_buffer.slice(..));
    rpass.set_vertex_buffer(1, self.vertex_buffer.slice(..));
    rpass.draw(0..3, 0..self.count);
  }
}
