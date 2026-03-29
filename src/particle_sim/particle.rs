use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Particle {
  pos: [f32; 2],
  vel: [f32; 2],
  color: [f32; 4],
}

impl Particle {
  pub fn random() -> Particle {
    let pos_range = 0.0..1024.0;
    let vel_range = -10.0..10.0;

    let x = rand::random_range(pos_range.clone());
    let y = rand::random_range(pos_range.clone());

    let vel_x = rand::random_range(vel_range.clone());
    let vel_y = rand::random_range(vel_range.clone());

    let r = rand::random::<f32>();
    let g = rand::random::<f32>();
    let b = rand::random::<f32>();

    Particle {
      pos: [x, y],
      vel: [vel_x, vel_y],
      color: [r, g, b, 1.0],
    }
  }
}
