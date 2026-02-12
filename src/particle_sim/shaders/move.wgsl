struct Params {
  dt: f32,
};

struct Particle {
  pos: vec2<f32>,
  vel: vec2<f32>,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> particlesSrc: array<Particle>;
@group(0) @binding(2) var<storage, read_write> particlesDst: array<Particle>;

@compute
@workgroup_size(64)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let total = arrayLength(&particlesSrc);
  let id = global_invocation_id.x;
  if (id >= total) {
    return;
  }

  var pos : vec2<f32> = particlesSrc[id].pos;
  var vel : vec2<f32> = particlesSrc[id].vel;

  pos += vel * params.dt;

  particlesDst[id] = Particle(pos, vel);
}
