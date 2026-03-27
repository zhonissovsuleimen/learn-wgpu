const RADIUS = 200.0;
const MAX_SPEED = 50.0;

const OUTSIDE_STRENGTH: f32 = 1.0;
const ACCEL_STRENGTH: f32 = 0.1;
const ALIGNMENT_STRENGTH: f32 = 10.0;

const COHESION_FAR_STRENGTH: f32 = 1.0;
const COHESION_FAR_RADIUS: f32 = 150.0;
const COHESION_CLOSE_STRENGTH: f32 = 20.0;
const COHESION_CLOSE_RADIUS: f32 = 100.0;

const SEPARATION_STRENGTH: f32 = 20.0;
const SEPARATION_RADIUS: f32 = 20.0;

const XENOPHOBIA_STRENGTH: f32 = 20.0;
const XENOPHOBIA_START_RADIUS: f32 = 200.0;
const XENOPHOBIA_END_RADIUS: f32 = 250.0;

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
  let fut_pos = pos + safe_normalize(vel) * RADIUS;
  var total_force: vec2<f32> = vec2(0.0);

  // force that speeds up particles, so slow particles won't stay slow for long
  let accel_force = ACCEL_STRENGTH * safe_normalize(vel);

  // force that is applied to particles outside of the window, directed to the window center
  let multiplier = OUTSIDE_STRENGTH * max(0.0f, sdf_box(fut_pos));
  let direction = safe_normalize((window.top_left + window.bottom_right) * 0.5 - pos);
  let outside_force = direction * multiplier;

  // force used to make out distinct groups
  var xenophobia: vec2<f32> = vec2(0.0);

  // typical boids forces
  var alignment: vec2<f32> = vec2(0.0);
  var cohesion_far: vec2<f32> = vec2(0.0);
  var cohesion_close: vec2<f32> = vec2(0.0);
  var separation: vec2<f32> = vec2(0.0);

  let i_max = arrayLength(&particlesSrc);
  for (var i = 0u; i < i_max; i += 1u) {
    let other = particlesSrc[i];
    let dist = length(other.pos - pos);

    if dist <= RADIUS {
      alignment += other.vel;
    }

    if dist <= COHESION_FAR_RADIUS {
      cohesion_far += other.pos - pos;
    }

    if dist <= COHESION_CLOSE_RADIUS {
      cohesion_close += other.pos - pos;
    }

    if dist <= SEPARATION_RADIUS {
      separation += pos - other.pos;
    }

    if dist >= XENOPHOBIA_START_RADIUS && dist <= XENOPHOBIA_END_RADIUS {
      xenophobia += pos - other.pos;
    }
  }

  let alignment_force = safe_normalize(alignment) * ALIGNMENT_STRENGTH;
  let cohesion_far_force = safe_normalize(cohesion_far) * COHESION_FAR_STRENGTH;
  let cohesion_close_force = safe_normalize(cohesion_close) * COHESION_CLOSE_STRENGTH;
  let separation_force = safe_normalize(separation) * SEPARATION_STRENGTH;
  let xenophobia_force = safe_normalize(xenophobia) * XENOPHOBIA_STRENGTH;

  total_force = outside_force + accel_force + alignment_force + cohesion_far_force + cohesion_close_force + separation_force + xenophobia_force;

  vel += total_force * params.dt;
  let speed = length(vel);
  vel = safe_normalize(vel) * clamp(speed, 0.0, MAX_SPEED);

  pos += vel * params.dt;

  particlesDst[id] = Particle(pos, vel);
}

fn sdf_box(p: vec2<f32>) -> f32 {
  let center = (window.top_left + window.bottom_right) * 0.5;
  let half_size = (window.bottom_right - window.top_left) * 0.5;
  let d = abs(p - center) - half_size;
  return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

fn safe_normalize(v: vec2<f32>) -> vec2<f32> {
  let len = length(v);
  return select(vec2(0.0), v / len, len > 0.0001);
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


