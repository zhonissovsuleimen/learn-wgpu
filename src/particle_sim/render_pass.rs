use crate::{
  app::{gpu_wrapper::GpuWrapper, resources::Resources, window_wrapper::WindowWrapper},
  gpu_pass::gpu_pass::GpuPass,
  particle_sim::{particle::Particle, shared::PARTICLES},
};
use tracing::error;
use wgpu::{
  BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer,
  BufferBindingType, BufferUsages, Color, CommandEncoder, Device, FragmentState, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor,
  PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages, StoreOp, TextureView,
  VertexBufferLayout, VertexState, VertexStepMode, include_wgsl,
  util::{BufferInitDescriptor, DeviceExt},
  vertex_attr_array,
};

#[derive(Default)]
pub struct RenderPass {
  vertex_buffer: Option<Buffer>,
  pipeline: Option<RenderPipeline>,

  bind_group_layout: Option<BindGroupLayout>,
  bind_group: Option<BindGroup>,
}

impl GpuPass for RenderPass {
  fn run(&mut self, encoder: &mut CommandEncoder, window: &WindowWrapper, gpu: &GpuWrapper, view: &TextureView, resources: &mut Resources) {
    let Some(particles) = resources.get::<Particle>(PARTICLES) else {
      error!("No particle buffer available");
      return;
    };

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
      label: Some("Render pass descriptor"),
      color_attachments: &color_attachments,
      depth_stencil_attachment: None,
      timestamp_writes: None,
      occlusion_query_set: None,
      multiview_mask: None,
    };

    let device = &gpu.device;
    let vertex_buffer = self.vertex_buffer.get_or_insert_with(|| RenderPass::init_vertex_buffer(device));
    let bind_group_layout = self.bind_group_layout.get_or_insert_with(|| RenderPass::init_bind_group_layout(device));
    let bind_group = self
      .bind_group
      .get_or_insert_with(|| RenderPass::init_bind_group(device, bind_group_layout, &particles.buffer));
    let pipeline = self
      .pipeline
      .get_or_insert_with(|| RenderPass::init_pipeline(device, bind_group_layout, window));

    let mut rpass = encoder.begin_render_pass(&render_pass_descriptor);

    rpass.set_pipeline(&pipeline);
    rpass.set_bind_group(0, &*bind_group, &[]);
    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
    rpass.draw(0..3, 0..particles.count);
  }
}

impl RenderPass {
  fn init_pipeline(device: &Device, bind_group_layout: &BindGroupLayout, window: &WindowWrapper) -> RenderPipeline {
    let shader = device.create_shader_module(include_wgsl!("shaders/draw.wgsl"));
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Render pipeline layout"),
      bind_group_layouts: &[bind_group_layout],
      immediate_size: 0,
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Render pipeline"),
      layout: Some(&pipeline_layout),
      vertex: VertexState {
        module: &shader,
        entry_point: Some("main_vs"),
        compilation_options: Default::default(),
        buffers: &[VertexBufferLayout {
          array_stride: 2 * 4, // single vertex x & y positions; 4 bytes each
          step_mode: VertexStepMode::Vertex,
          attributes: &vertex_attr_array![0 => Float32x2],
        }],
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
    })
  }

  fn init_vertex_buffer(device: &Device) -> Buffer {
    #[rustfmt::skip]
    let vertex_buffer_data: [f32; 6] = [
      -0.01, -0.02,
      0.01, -0.02,
      0.00, 0.02
    ];

    device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::bytes_of(&vertex_buffer_data),
      usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
    })
  }

  fn init_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: Some("Particle bind group layout"),
      entries: &[BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::VERTEX,
        ty: BindingType::Buffer {
          ty: BufferBindingType::Storage { read_only: true },
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      }],
    })
  }

  fn init_bind_group(device: &Device, bind_group_layout: &BindGroupLayout, particle_buffer: &Buffer) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
      label: Some("Particle Bind Group"),
      layout: &bind_group_layout,
      entries: &[BindGroupEntry {
        binding: 0,
        resource: particle_buffer.as_entire_binding(),
      }],
    })
  }
}
