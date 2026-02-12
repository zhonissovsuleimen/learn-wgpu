//repr C needed cause wgsl expects c-like padding (i think), and this is a compiler hint for that
#[repr(C)]
#[derive(Clone)]
pub struct Particle {
  pos: [f32; 2],
  vel: [f32; 2],
}

//todo buffer layouts/stide/binding type, as a trait?
