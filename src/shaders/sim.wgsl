struct Particle {
    @location(0) position: vec3<f32>,
    @location(1) velocity: vec3<f32>,
    @location(2) color: vec3<f32>,
};


struct Particles {
    particles: array<Particle>,
};

let DT: f32 = 0.033333333;

@group(0) @binding(0) var<storage, read> particles_src : Particles;
@group(0) @binding(1) var<storage, read_write> particles_dst : Particles;

@compute @workgroup_size(16)
fn main(
  @builtin(global_invocation_id) global_id : vec3<u32>,
) {
    let index: u32 = global_id.x;

    let total = arrayLength(&particles_src.particles);
    if (index >= total) {
        return;
    }

    particles_dst.particles[index].position = vec3<f32>(0.1,0.0,0.0);
}