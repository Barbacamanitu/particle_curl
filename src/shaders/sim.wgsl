struct Particle {
    position: vec4<f32>,
    velocity: vec4<f32>,
    color: vec4<f32>,
};


struct Particles {
    particles: array<Particle>,
};

let DT: f32 = 0.008;
let o: f32 = 10.0;
let p: f32 = 28.0;
let b: f32 = 2.6666;


fn compute_velocity(pos: vec3<f32>) -> vec3<f32> {
    let x = o * (pos.y-pos.x);
    let y = pos.x*( p - pos.z) - pos.y;
    let z = (pos.x * pos.y) - (b * pos.z);
    let vel = vec3<f32>(x,y,z);
    return vel;
}

@group(0) @binding(0) var<storage, read> particles_src : Particles;
@group(0) @binding(1) var<storage, read_write> particles_dst : Particles;

@compute @workgroup_size(64)
fn main(
  @builtin(global_invocation_id) global_id : vec3<u32>,
) {
    let index: u32 = global_id.x;

    let total = arrayLength(&particles_src.particles);
    if (index >= total) {
        return;
    }
    var p = particles_src.particles[index];
    let newVel = compute_velocity(p.position.xyz);
    p.velocity = vec4<f32>(newVel.xyz,0.0);
    p.position = p.position + p.velocity * DT;
    
    particles_dst.particles[index] = p;
}


