struct Particle {
  pos : vec2<f32>,
  vel : vec2<f32>,
};

@group(0) @binding(0) var<storage, read> particles : array<Particle>;

@vertex
fn main_vs(
  @builtin(instance_index) id : u32,
  @location(0) vertex : vec2<f32>,
) -> @builtin(position) vec4<f32> {
  let p = particles[id];

  let angle = -atan2(p.vel.x, p.vel.y);
  let rotated_vertex = vec2<f32>(
    vertex.x * cos(angle) - vertex.y * sin(angle),
    vertex.x * sin(angle) + vertex.y * cos(angle)
  );

  return vec4<f32>(rotated_vertex + p.pos, 0.0, 1.0);
}

@fragment
fn main_fs() -> @location(0) vec4<f32> {
  return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
