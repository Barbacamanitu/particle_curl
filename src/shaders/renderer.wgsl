struct VertexInput {
    @location(0) particle_position: vec4<f32>,
    @location(1) particle_velocity: vec4<f32>,
    @location(2) particle_color: vec4<f32>,
    @location(3) quad_vertex_position: vec4<f32>,
    @location(4) quad_tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) velocity: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct CameraUniform {
    _mat: mat4x4<f32>,
};

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

fn psin(x: f32) -> f32 {
    return (sin(x)+1.0)/2.0;
}

fn vel2col(v: vec4<f32>) -> vec4<f32> {
    let scalar = 1.125;
    let lv = length(v);
    let r = psin(lv * scalar);
    let g = psin(lv * 3.0 *  scalar);
    let b = psin((v.z * sin(v.x) + cos(v.y)) * 0.3 * scalar);
    return vec4<f32>(r,g,b,0.8);
}

fn pos2col(v: vec3<f32>) -> vec4<f32> {
    let n = simplex3d(v*0.1);
    return vec4<f32>(n,n,n,1.0);
}

@group(1) @binding(0) // 1.
var<uniform> camera_view: CameraUniform;
@group(1) @binding(1) // 1.
var<uniform> camera_projection: CameraUniform;
@group(1) @binding(2) // 1.
var<uniform> camera_view_inv: CameraUniform;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
   
    //Calculate corners of quad using inverse camera view matrix
    let billboard_pos = camera_view_inv._mat * model.quad_vertex_position;
    let part_pos = model.particle_position + billboard_pos;
    //Clip position calculated using view matrix, projection matrix, and and particle position
    out.clip_position = camera_projection._mat * camera_view._mat * part_pos;
    out.velocity = model.particle_velocity;

    //let col = vel2col(model.particle_velocity);
    let col = pos2col(model.particle_position.xyz);
    out.color = col;
    out.tex_coords = model.quad_tex_coords;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let p_color = in.color;
    let n_color = tex_color.rgba * p_color.rgba;
   return n_color;
}