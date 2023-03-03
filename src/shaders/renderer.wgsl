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

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let newPos = model.particle_position + model.quad_vertex_position;
    out.clip_position = vec4<f32>(newPos.xyz, 1.0);
    out.velocity = model.particle_velocity;
    out.color = model.particle_color;
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
    let n_color = tex_color.rgb * p_color.rgb;
   return vec4<f32>(n_color.rgb,tex_color.a);
}