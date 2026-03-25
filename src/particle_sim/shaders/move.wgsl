const MIN_SPEED = 10.0;
const MAX_SPEED = 150.0;

const OUTSIDE_FORCE_STRENGTH: f32 = 1.0;
const ACCEL_FORCE_STRENGTH: f32 = 5.0;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<uniform> window: Window;
@group(0) @binding(2) var<storage, read> particlesSrc: array<Particle>;
@group(0) @binding(3) var<storage, read_write> particlesDst: array<Particle>;

@compute
@workgroup_size(64)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let total = arrayLength(&particlesSrc);
  let id = global_invocation_id.x;
  if (id >= total) {
    return;
  }

  var pos: vec2<f32> = particlesSrc[id].pos;
  var vel: vec2<f32> = particlesSrc[id].vel;
  var total_force: vec2<f32> = vec2(0.0);

  // force that speeds up particles, so slow particles won't stay slow for long
  let accel_force = ACCEL_FORCE_STRENGTH * normalize(vel);

  // force that is applied to particles outside of the window, directed to the window center
  // we use future position to stop applying force once the particle is moving towards the window
  let fut_pos = pos + vel * 0.5;
  let multiplier = OUTSIDE_FORCE_STRENGTH * max(0.0f, sdf_box(fut_pos));
  let direction = normalize((window.top_left + window.bottom_right) * 0.5 - pos);
  let outside_force = direction * multiplier;

  total_force = outside_force + accel_force;

  vel += total_force * params.dt;
  let speed = length(vel);
  vel = normalize(vel) * clamp(speed, MIN_SPEED, MAX_SPEED);

  pos += vel * params.dt;

  particlesDst[id] = Particle(pos, vel);
}

fn sdf_box(p: vec2<f32>) -> f32 {
  let radius_multiplier = 0.95;

  let center = (window.top_left + window.bottom_right) * 0.5;
  let half_size = (window.bottom_right - window.top_left) * 0.5 * radius_multiplier;
  let d = abs(p - center) - half_size;
  return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

struct Params {
  dt: f32,
};

struct Particle {
  pos: vec2<f32>,
  vel: vec2<f32>,
};

struct Window {
  top_left: vec2<f32>,
  bottom_right: vec2<f32>,
}

