struct VertexInput {
    @location(0) particle_position: vec4<f32>,
    @location(1) particle_velocity: vec4<f32>,
    @location(2) particle_color: vec4<f32>,
    @location(3) quad_vertex_position: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) velocity: vec4<f32>,
    @location(1) color: vec4<f32>
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
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  
    return vec4<f32>(in.color.rgb, 1.0);
}