use crate::app::{gpu_wrapper::GpuWrapper, window_wrapper::WindowWrapper};
use wgpu::{
  BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer,
  BufferBindingType, BufferUsages, Color, CommandEncoder, Device, FragmentState, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor,
  PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages, StoreOp, TextureView,
  VertexBufferLayout, VertexState, VertexStepMode, include_wgsl,
  util::{BufferInitDescriptor, DeviceExt},
  vertex_attr_array,
};

pub struct RenderPass {
  vertex_buffer: Buffer,
  pipeline: RenderPipeline,

  particle_count: u32,

  bind_group: BindGroup,
}

impl RenderPass {
  pub fn run(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
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

    let mut rpass = encoder.begin_render_pass(&render_pass_descriptor);

    rpass.set_pipeline(&self.pipeline);
    rpass.set_bind_group(0, &self.bind_group, &[]);
    rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    rpass.draw(0..3, 0..self.particle_count);
  }

  pub fn init(gpu: &GpuWrapper, window: &WindowWrapper, window_buffer: &Buffer, particle_buffer: Buffer, particle_count: u32) -> RenderPass {
    let device = &gpu.device;
    let vertex_buffer = RenderPass::init_vertex_buffer(device);

    let layout = RenderPass::init_bind_group_layout(device);
    let bind_group = RenderPass::init_bind_group(device, &layout, &particle_buffer, &window_buffer);
    let pipeline = RenderPass::init_pipeline(device, &layout, window);

    RenderPass {
      vertex_buffer,
      pipeline,
      particle_count,
      bind_group,
    }
  }

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
        targets: &[Some(window.surface_config.format.into())],
      }),
      primitive: PrimitiveState::default(),
      depth_stencil: None,
      multisample: MultisampleState::default(),
      multiview_mask: None,
      cache: None,
    })
  }

  fn init_vertex_buffer(device: &Device) -> Buffer {
    let size = 3.5f32;
    #[rustfmt::skip]
    let vertex_buffer_data: [f32; 6] = [
      -1.0 * size, -2.0 * size,
      size, -2.0 * size,
      0.0, 2.0 * size,
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
      entries: &[
        BindGroupLayoutEntry {
          binding: 0,
          visibility: ShaderStages::VERTEX,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
        BindGroupLayoutEntry {
          binding: 1,
          visibility: ShaderStages::VERTEX,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
      ],
    })
  }

  fn init_bind_group(device: &Device, bind_group_layout: &BindGroupLayout, particle_buffer: &Buffer, window_buffer: &Buffer) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
      label: Some("Particle Bind Group"),
      layout: &bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: particle_buffer.as_entire_binding(),
        },
        BindGroupEntry {
          binding: 1,
          resource: window_buffer.as_entire_binding(),
        },
      ],
    })
  }
}
