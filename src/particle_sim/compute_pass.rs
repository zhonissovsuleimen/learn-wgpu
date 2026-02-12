use crate::{
  app::{gpu_wrapper::GpuWrapper, resources::Resources, window_wrapper::WindowWrapper},
  gpu_pass::{buffer_wrapper::BufferWrapper, gpu_pass::GpuPass},
  particle_sim::{particle::Particle, shared::PARTICLES},
};
use std::time::Instant;
use wgpu::{
  BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer,
  BufferBindingType, BufferSize, BufferUsages, CommandEncoder, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device,
  PipelineLayoutDescriptor, ShaderStages, TextureView, include_wgsl,
  util::{BufferInitDescriptor, DeviceExt},
};

#[derive(Default)]
pub struct ComputePass {
  params_buffer: Option<Buffer>,
  pipeline: Option<ComputePipeline>,

  bind_group_layout: Option<BindGroupLayout>,
  bind_group_a: Option<BindGroup>,
  bind_group_b: Option<BindGroup>,

  buffer_a: Option<BufferWrapper<Particle>>,
  buffer_b: Option<BufferWrapper<Particle>>,
  write_to_buffer_a: bool,

  last_run: Option<Instant>,
}

impl GpuPass for ComputePass {
  fn run(&mut self, encoder: &mut CommandEncoder, _window: &WindowWrapper, gpu: &GpuWrapper, _view: &TextureView, resources: &mut Resources) {
    let (_, _, device, queue) = gpu.into();

    let params_buffer = self.params_buffer.get_or_insert_with(|| ComputePass::init_params_buffer(device));
    let particle_buffer_a = self.buffer_a.get_or_insert_with(|| ComputePass::init_buffer(device));
    let particle_buffer_b = self.buffer_b.get_or_insert_with(|| ComputePass::init_buffer(device));

    let bind_group_layout = self
      .bind_group_layout
      .get_or_insert_with(|| ComputePass::init_bind_group_layout(device, params_buffer, &particle_buffer_a.buffer, &particle_buffer_b.buffer));
    let bind_group_a = self.bind_group_a.get_or_insert_with(|| {
      ComputePass::init_bind_group_a(
        device,
        bind_group_layout,
        params_buffer,
        &particle_buffer_a.buffer,
        &particle_buffer_b.buffer,
      )
    });
    let bind_group_b = self.bind_group_b.get_or_insert_with(|| {
      ComputePass::init_bind_group_b(
        device,
        bind_group_layout,
        params_buffer,
        &particle_buffer_a.buffer,
        &particle_buffer_b.buffer,
      )
    });

    let dt = match self.last_run {
      Some(last) => last.elapsed().as_secs_f32(),
      None => 0.0f32,
    };
    let new_params_data = [dt];
    queue.write_buffer(params_buffer, 0, bytemuck::cast_slice(&new_params_data));

    let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
      label: Some("Compute pass descriptor"),
      timestamp_writes: None,
    });

    // a lil ugly might fix later
    let clone;
    let workgroup_count;
    const WORKGROUP_SIZE: u32 = 64;

    if self.write_to_buffer_a {
      cpass.set_bind_group(0, &*bind_group_b, &[]);
      clone = particle_buffer_a.clone();
    } else {
      cpass.set_bind_group(0, &*bind_group_a, &[]);
      clone = particle_buffer_b.clone();
    }
    workgroup_count = (clone.count + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;
    resources.insert(PARTICLES, clone);

    let pipeline = self.pipeline.get_or_insert_with(|| ComputePass::init_pipeline(device, bind_group_layout));
    cpass.set_pipeline(pipeline);
    cpass.dispatch_workgroups(workgroup_count, 1, 1);

    self.write_to_buffer_a = !self.write_to_buffer_a;
    self.last_run = Some(Instant::now());
  }
}

impl ComputePass {
  fn init_params_buffer(device: &Device) -> Buffer {
    let params_arr = [
      0.0f32, //dt
    ];

    device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Params buffer"),
      contents: bytemuck::cast_slice(&params_arr),
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
  }

  fn init_buffer(device: &Device) -> BufferWrapper<Particle> {
    let count = 1000;
    let mut data = vec![0.0f32; (4 * count) as usize];

    let pos_range = -1.0..1.0;
    let vel_range = -0.1..0.1;
    for i in 0..count {
      data[4 * i] = rand::random_range(pos_range.clone());
      data[4 * i + 1] = rand::random_range(pos_range.clone());
      data[4 * i + 2] = rand::random_range(vel_range.clone());
      data[4 * i + 3] = rand::random_range(vel_range.clone());
    }

    let buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Particle buffer"),
      contents: &bytemuck::cast_slice(&data),
      usage: BufferUsages::VERTEX | BufferUsages::STORAGE | BufferUsages::COPY_DST,
    });

    BufferWrapper::new(buffer, count as u32)
  }

  fn init_bind_group_layout(device: &Device, params_buffer: &Buffer, particle_buffer_a: &Buffer, particle_buffer_b: &Buffer) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: Some("Compute bind group layout"),
      entries: &[
        BindGroupLayoutEntry {
          binding: 0,
          visibility: ShaderStages::COMPUTE,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: BufferSize::new(params_buffer.size()),
          },
          count: None,
        },
        BindGroupLayoutEntry {
          binding: 1,
          visibility: ShaderStages::COMPUTE,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: BufferSize::new(particle_buffer_a.size()),
          },
          count: None,
        },
        BindGroupLayoutEntry {
          binding: 2,
          visibility: ShaderStages::COMPUTE,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: BufferSize::new(particle_buffer_b.size()),
          },
          count: None,
        },
      ],
    })
  }

  fn init_bind_group_a(
    device: &Device,
    bind_group_layout: &BindGroupLayout,
    params_buffer: &Buffer,
    particle_buffer_a: &Buffer,
    particle_buffer_b: &Buffer,
  ) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
      label: Some("Compute bind group A"),
      layout: &bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: params_buffer.as_entire_binding(),
        },
        BindGroupEntry {
          binding: 1,
          resource: particle_buffer_a.as_entire_binding(),
        },
        BindGroupEntry {
          binding: 2,
          resource: particle_buffer_b.as_entire_binding(),
        },
      ],
    })
  }

  fn init_bind_group_b(
    device: &Device,
    bind_group_layout: &BindGroupLayout,
    params_buffer: &Buffer,
    particle_buffer_a: &Buffer,
    particle_buffer_b: &Buffer,
  ) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
      label: Some("Compute bind group B"),
      layout: &bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: params_buffer.as_entire_binding(),
        },
        BindGroupEntry {
          binding: 1,
          resource: particle_buffer_b.as_entire_binding(),
        },
        BindGroupEntry {
          binding: 2,
          resource: particle_buffer_a.as_entire_binding(),
        },
      ],
    })
  }

  fn init_pipeline(device: &Device, bind_group_layout: &BindGroupLayout) -> ComputePipeline {
    let shader = device.create_shader_module(include_wgsl!("shaders/move.wgsl"));

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Compute pipeline layout"),
      bind_group_layouts: &[&bind_group_layout],
      immediate_size: 0,
    });

    device.create_compute_pipeline(&ComputePipelineDescriptor {
      label: Some("Compute pipeline"),
      layout: Some(&pipeline_layout),
      module: &shader,
      entry_point: Some("main"),
      compilation_options: Default::default(),
      cache: None,
    })
  }
}
