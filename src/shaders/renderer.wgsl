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




fn psin(x: f32) -> f32 {
    return (sin(x)+1.0)/2.0;
}

fn vel2col(v: vec4<f32>) -> vec4<f32> {
    let scalar = 0.425;
    let lv = length(v);
    let r = 0.5;
    let g = 0.5;
    let b = 0.5;
    return vec4<f32>(r,g,b,0.8);
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

    let col = vel2col(model.particle_velocity);
    
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