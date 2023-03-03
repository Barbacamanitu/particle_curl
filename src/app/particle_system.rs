use std::mem;

use super::particle_gpu::{self, ParticleGPU};
use super::{gpu::Gpu, math::UVec2};
use rand::Rng;
use wgpu::util::DeviceExt;

const NUM_PARTICLES: usize = 512;
const PARTICLES_PER_GROUP: u32 = 2;

pub struct ParticleSystem {
    particle_gpu: ParticleGPU,
    //compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    frame_counter: usize,
    work_group_count: u32,
    pub size: UVec2,
}

impl ParticleSystem {
    pub fn new(gpu: &Gpu, size: UVec2) -> ParticleSystem {
        //let compute_pipeline = ParticleSystem::build_compute_pipeline(gpu);
        let render_pipeline = ParticleSystem::build_render_pipeline(gpu);

        let frame_counter = 0;
        let work_group_count =
            ((NUM_PARTICLES as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        let particle_gpu = ParticleGPU::new(gpu);
        ParticleSystem {
            particle_gpu,
            //compute_pipeline,
            render_pipeline,
            frame_counter,
            work_group_count,
            size,
        }
    }

    /*fn build_compute_pipeline(gpu: &Gpu) -> wgpu::ComputePipeline {
        let shader_src = include_str!("../shaders/sim.wgsl");
        let compute_shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Sim Shader"),
                source: wgpu::ShaderSource::Wgsl(shader_src.into()),
            });
        let compute_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    (NUM_PARTICLES * mem::size_of::<Particle>()) as _,
                                ),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    (NUM_PARTICLES * mem::size_of::<Particle>()) as _,
                                ),
                            },
                            count: None,
                        },
                    ],
                    label: None,
                });

        let compute_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("particle system compute"),
                    bind_group_layouts: &[&compute_bind_group_layout],
                    push_constant_ranges: &[],
                });

        gpu.device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Compute pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &compute_shader,
                entry_point: "main",
            })
    }*/

    fn build_render_pipeline(gpu: &Gpu) -> wgpu::RenderPipeline {
        let shader_src = include_str!("../shaders/renderer.wgsl");
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Render Shader"),
                source: wgpu::ShaderSource::Wgsl(shader_src.into()),
            });
        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),

                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: 4 * 12,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: 4 * 4,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![3 => Float32x4],
                        },
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        // 4.
                        format: gpu.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // 2.
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None, // 1.
                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview: None, // 5.
            });
        render_pipeline
    }

    pub fn render(&mut self, gpu: &Gpu) {
        self.run_render(gpu);
    }

    pub fn update(&mut self, gpu: &Gpu) {
        self.frame_counter += 1;
        //self.run_compute(gpu);
    }

    fn run_render(&mut self, gpu: &Gpu) {
        let output = gpu.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        encoder.push_debug_group("Particle System Render");
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.particle_gpu.particle_position_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.particle_gpu.quad_vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.particle_gpu.quad_index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            ); // 1.
            render_pass.draw_indexed(0..6, 0, 0..3); // 2.
        }
        encoder.pop_debug_group();
        gpu.queue.submit([encoder.finish()]);
        output.present();
    }

    fn run_compute(&mut self, gpu: &Gpu) { /*
                                           let mut encoder = gpu
                                               .device
                                               .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                                   label: Some("Render Encoder"),
                                               });
                                           encoder.push_debug_group("Particle System Compute");
                                           {
                                               let mut compute_pass =
                                                   encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
                                               compute_pass.set_pipeline(&self.compute_pipeline);
                                               compute_pass.set_bind_group(0, &self.bind_groups.particles[0], &[]);
                                               compute_pass.set_bind_group(1, &self.bind_groups.particles[1], &[]);
                                               compute_pass.dispatch_workgroups(256, 1, 1);
                                           }
                                           encoder.pop_debug_group();
                                           */
    }
}
