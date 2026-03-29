use crate::app::gpu_wrapper::GpuWrapper;
use std::time::Instant;
use wgpu::{
  BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer,
  BufferBindingType, BufferSize, BufferUsages, CommandEncoder, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device,
  PipelineLayoutDescriptor, Queue, ShaderStages, include_wgsl,
  util::{BufferInitDescriptor, DeviceExt},
};

pub struct ComputePass {
  params_buffer: Buffer,
  pipeline: ComputePipeline,

  bind_group_a: BindGroup,
  bind_group_b: BindGroup,

  particle_buffer_a: Buffer,
  particle_buffer_b: Buffer,
  particle_count: u32,

  write_to_buffer_a: bool,
  last_run: Option<Instant>,
}

impl ComputePass {
  pub fn init(gpu: &GpuWrapper, window_buffer: &Buffer) -> ComputePass {
    let device = &gpu.device;
    let params_buffer = ComputePass::init_params_buffer(device);

    let count = 500;
    let particle_data = ComputePass::init_particle_data(count);
    let particle_buffer_a = ComputePass::init_particle_buffer(device, particle_data.clone());
    let particle_buffer_b = ComputePass::init_particle_buffer(device, particle_data);

    let layout = ComputePass::init_bind_group_layout(device, &params_buffer, &window_buffer, &particle_buffer_a, &particle_buffer_b);
    let bind_group_a = ComputePass::init_bind_group_a(device, &params_buffer, &window_buffer, &particle_buffer_a, &particle_buffer_b, &layout);
    let bind_group_b = ComputePass::init_bind_group_b(device, &params_buffer, &window_buffer, &particle_buffer_a, &particle_buffer_b, &layout);

    let pipeline = ComputePass::init_pipeline(device, &layout);
    ComputePass {
      params_buffer,
      pipeline,
      bind_group_a,
      bind_group_b,
      particle_buffer_a: particle_buffer_a,
      particle_buffer_b: particle_buffer_b,
      particle_count: count,
      write_to_buffer_a: false,
      last_run: None,
    }
  }

  pub fn run(&mut self, encoder: &mut CommandEncoder, gpu: &GpuWrapper) {
    let queue = &gpu.queue;
    self.update_params_buffer(queue);

    let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
      label: Some("Compute pass descriptor"),
      timestamp_writes: None,
    });

    const WORKGROUP_SIZE: u32 = 64;

    if self.write_to_buffer_a {
      cpass.set_bind_group(0, &self.bind_group_b, &[]);
    } else {
      cpass.set_bind_group(0, &self.bind_group_a, &[]);
    };

    let workgroup_count = (self.particle_count + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;

    cpass.set_pipeline(&self.pipeline);
    cpass.dispatch_workgroups(workgroup_count, 1, 1);

    self.write_to_buffer_a = !self.write_to_buffer_a;
    self.last_run = Some(Instant::now());
  }

  pub fn get_particle_buffer(&self) -> (&Buffer, u32) {
    if self.write_to_buffer_a {
      (&self.particle_buffer_a, self.particle_count)
    } else {
      (&self.particle_buffer_b, self.particle_count)
    }
  }

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

  fn init_particle_data(count: u32) -> Vec<f32> {
    let count = count as usize;
    let mut data = vec![0.0f32; 4 * count];

    let pos_range = 0.0..1024.0;
    let vel_range = -10.0..10.0;
    for i in 0..count {
      data[4 * i] = rand::random_range(pos_range.clone());
      data[4 * i + 1] = rand::random_range(pos_range.clone());
      data[4 * i + 2] = rand::random_range(vel_range.clone());
      data[4 * i + 3] = rand::random_range(vel_range.clone());
    }

    data.to_vec()
  }

  fn init_particle_buffer(device: &Device, data: Vec<f32>) -> Buffer {
    device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Particle buffer"),
      contents: &bytemuck::cast_slice(&data),
      usage: BufferUsages::VERTEX | BufferUsages::STORAGE | BufferUsages::COPY_DST,
    })
  }

  fn init_bind_group_layout(
    device: &Device,
    params_buffer: &Buffer,
    window_buffer: &Buffer,
    particle_buffer_a: &Buffer,
    particle_buffer_b: &Buffer,
  ) -> BindGroupLayout {
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
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: BufferSize::new(window_buffer.size()),
          },
          count: None,
        },
        BindGroupLayoutEntry {
          binding: 2,
          visibility: ShaderStages::COMPUTE,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: BufferSize::new(particle_buffer_a.size()),
          },
          count: None,
        },
        BindGroupLayoutEntry {
          binding: 3,
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
    params_buffer: &Buffer,
    window_buffer: &Buffer,
    particle_buffer_a: &Buffer,
    particle_buffer_b: &Buffer,
    layout: &BindGroupLayout,
  ) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
      label: Some("Compute bind group A"),
      layout: layout,
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: params_buffer.as_entire_binding(),
        },
        BindGroupEntry {
          binding: 1,
          resource: window_buffer.as_entire_binding(),
        },
        BindGroupEntry {
          binding: 2,
          resource: particle_buffer_a.as_entire_binding(),
        },
        BindGroupEntry {
          binding: 3,
          resource: particle_buffer_b.as_entire_binding(),
        },
      ],
    })
  }

  fn init_bind_group_b(
    device: &Device,
    params_buffer: &Buffer,
    window_buffer: &Buffer,
    particle_buffer_a: &Buffer,
    particle_buffer_b: &Buffer,
    layout: &BindGroupLayout,
  ) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
      label: Some("Compute bind group A"),
      layout: layout,
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: params_buffer.as_entire_binding(),
        },
        BindGroupEntry {
          binding: 1,
          resource: window_buffer.as_entire_binding(),
        },
        BindGroupEntry {
          binding: 2,
          resource: particle_buffer_b.as_entire_binding(),
        },
        BindGroupEntry {
          binding: 3,
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

  fn update_params_buffer(&mut self, queue: &Queue) {
    let dt = match self.last_run {
      Some(last) => last.elapsed().as_secs_f32(),
      None => 0.0f32,
    };
    let new_params_data = [dt];
    queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&new_params_data));
  }
}
