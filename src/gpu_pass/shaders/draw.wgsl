@vertex
fn main_vs(
    @location(0) pos: vec2<f32>,
    @location(1) vel: vec2<f32>,
    @location(2) vertex: vec2<f32>,
) -> @builtin(position) vec4<f32> {
  let angle = -atan2(vel.x, vel.y);
  let rotated_vertex = vec2<f32>(
      vertex.x * cos(angle) - vertex.y * sin(angle),
      vertex.x * sin(angle) + vertex.y * cos(angle)
  );
  return vec4<f32>(rotated_vertex + pos, 0.0, 1.0);
}

@fragment
fn main_fs() -> @location(0) vec4<f32> {
  return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
