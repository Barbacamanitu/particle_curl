
//Parameters
let noise_scale: f32 = 0.030;
let speed_multiplier: f32 = 15.0;
let max_extent: f32 = 300.0;

//Constants
let pi: f32 = 3.14159;
let rot1: mat3x3<f32> = mat3x3<f32>(vec3<f32>(-0.37, 0.36, 0.85),vec3<f32>(-0.14,-0.93, 0.34),vec3<f32>(0.92, 0.01,0.4));
let rot2: mat3x3<f32> = mat3x3<f32>(vec3<f32>(-0.55,-0.39, 0.74),vec3<f32>( 0.33,-0.91,-0.24),vec3<f32>(0.77, 0.12,0.63));
let rot3: mat3x3<f32> = mat3x3<f32>(vec3<f32>(-0.71, 0.52,-0.47),vec3<f32>(-0.08,-0.72,-0.68),vec3<f32>(-0.7,-0.45,0.56));


struct Particle {
    position: vec4<f32>,
    velocity: vec4<f32>,
    color: vec4<f32>,
};


struct Particles {
    particles: array<Particle>,
};


let DT: f32 = 0.016;
//
fn random3(c: vec3<f32>) -> vec3<f32> {
	var j: f32 = 4096.0*sin(dot(c,vec3<f32>(17.0, 59.4, 15.0)));
	var r: vec3<f32>;
	r.z = fract(512.0*j);
	j *= .125;
	r.x = fract(512.0*j);
	j *= .125;
	r.y = fract(512.0*j);
	return r-vec3<f32>(0.5);
}

/* skew constants for 3d simplex functions */
let F3: f32 =  0.3333333;
let G3: f32 =  0.1666667;

/* 3d simplex noise */
fn simplex3d(p: vec3<f32>) -> f32 {
	 /* 1. find current tetrahedron T and it's four vertices */
	 /* s, s+i1, s+i2, s+1.0 - absolute skewed (integer) coordinates of T vertices */
	 /* x, x1, x2, x3 - unskewed coordinates of p relative to each of T vertices*/
	 
	 /* calculate s and x */
	 let s: vec3<f32> = floor(p + dot(p, vec3<f32>(F3)));
	 let x: vec3<f32> = p - s + dot(s, vec3<f32>(G3));
	 
	 /* calculate i1 and i2 */
	 let e : vec3<f32> = step(vec3<f32>(0.0), x - x.yzx);
	 let i1: vec3<f32> = e*(1.0 - e.zxy);
	 let i2: vec3<f32> = 1.0 - e.zxy*(1.0 - e);
	 	
	 /* x1, x2, x3 */
	 let x1: vec3<f32> = x - i1 + G3;
	 let x2: vec3<f32> = x - i2 + 2.0*G3;
	 let x3: vec3<f32> = x - 1.0 + 3.0*G3;
	 
	 /* 2. find four surflets and store them in d */
	 var w: vec4<f32>;
     var d: vec4<f32>;
	 
	 /* calculate surflet weights */
	 w.x = dot(x, x);
	 w.y = dot(x1, x1);
	 w.z = dot(x2, x2);
	 w.w = dot(x3, x3);
	 
	 /* w fades from 0.6 at the center of the surflet to 0.0 at the margin */
	 w = max(vec4<f32>(0.6) - w, vec4<f32>(0.0));
	 
	 /* calculate surflet components */
	 d.x = dot(random3(s), x);
	 d.y = dot(random3(s + i1), x1);
	 d.z = dot(random3(s + i2), x2);
	 d.w = dot(random3(s + 1.0), x3);
	 
	 /* multiply d by w^4 */
	 w *= w;
	 w *= w;
	 d *= w;
	 
	 /* 3. return the sum of the four surflets */
	 return dot(d, vec4<f32>(52.0));
}

/* directional artifacts can be reduced by rotating each octave */
fn simplex3d_fractal(m: vec3<f32>) -> f32 {
    return 0.5333333*simplex3d(m*rot1)
			+0.2666667*simplex3d(2.0*m*rot2)
			+0.1333333*simplex3d(4.0*m*rot3)
			+0.0666667*simplex3d(8.0*m);
}

fn vector_field(p: vec3<f32>, scale: f32) -> vec3<f32> {
    let pos: vec3<f32> = p * scale;

    let x_p = pos;
    let y_p = pos + vec3<f32>(1000.0,0.0,0.0);
    let z_p = pos + vec3<f32>(2000.0,0.0,0.0);

    let x_n = simplex3d_fractal(x_p);
    let y_n = simplex3d_fractal(y_p);
    let z_n = simplex3d_fractal(z_p);
    let direction = vec3<f32>(x_n,y_n,0.0);
    return direction;
}

fn curl(pos: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(0.0,0.0,0.0);
}


fn compute_vel(pos: vec3<f32>) -> vec3<f32> {
    let vf = vector_field(pos,noise_scale) + vec3<f32>(0.0,0.6,0.0);
    return (vf * speed_multiplier);
}

fn clamp_position(pos: vec4<f32>) -> vec4<f32> {
    var p = pos;
    if (p.x < -max_extent) {
        p.x += max_extent;
    }
    if (p.y < -max_extent) {
        p.y += max_extent;
    }

    if (p.z < -max_extent) {
        p.z += max_extent;
    }

    if (p.x > max_extent) {
        p.x -= max_extent;
    }
    if (p.y > max_extent) {
        p.y -= max_extent;
    }
     if (p.z > max_extent) {
        p.z -= max_extent;
    }

    return vec4<f32>(p.xyz,1.0);
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
    let new_velocity = compute_vel(p.position.xyz);

    p.velocity = vec4<f32>(new_velocity,0.0);
    p.position = p.position + p.velocity * DT;
    p.position = clamp_position(p.position);
    particles_dst.particles[index] = p;
}


