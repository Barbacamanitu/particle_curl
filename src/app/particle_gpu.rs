//Low level gpu stuff for particles. Keeps main particle system file less cluttered.

use std::time;

use bytemuck::{Pod, Zeroable};
use rand::Rng;
use wgpu::util::DeviceExt;

use super::{camera::FatCamera, gpu::Gpu, texture::Texture};

pub const PARTICLES_PER_GROUP: u32 = 64;
const PARTICLE_SIZE: f32 = 0.5;

//Holds all gpu state for particles.
pub struct ParticleGPU {
    pub quad_vertex_buffer: wgpu::Buffer,
    pub quad_index_buffer: wgpu::Buffer,
    pub particle_buffers: Vec<wgpu::Buffer>,
    pub particle_bind_groups: Vec<wgpu::BindGroup>,
    pub particle_texture: Texture,
    pub texture_bind_group: wgpu::BindGroup,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub render_pipeline: wgpu::RenderPipeline,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub work_group_count: u32,
    pub depth_texture: Texture,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
    pub tex_coords: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Particle {
    pub position: [f32; 4],
    pub velocity: [f32; 4],
    pub color: [f32; 4],
}

const QUAD_VERTICES: &[Vertex] = &[
    //Top left
    Vertex {
        position: [-PARTICLE_SIZE / 2.0, PARTICLE_SIZE / 2.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    //Bottom left
    Vertex {
        position: [-PARTICLE_SIZE / 2.0, -PARTICLE_SIZE / 2.0, 0.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    //Bottom right
    Vertex {
        position: [PARTICLE_SIZE / 2.0, -PARTICLE_SIZE / 2.0, 0.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    //Top right
    Vertex {
        position: [PARTICLE_SIZE / 2.0, PARTICLE_SIZE / 2.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

impl ParticleGPU {
    fn generate_particle_buffers(gpu: &Gpu, particle_data: &Vec<Particle>) -> Vec<wgpu::Buffer> {
        let mut buffers: Vec<wgpu::Buffer> = Vec::new();

        for i in 0..2 {
            buffers.push(
                gpu.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Particle Buffer {}", i)),
                        contents: bytemuck::cast_slice(particle_data),
                        usage: wgpu::BufferUsages::VERTEX
                            | wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_DST,
                    }),
            );
        }

        buffers
    }

    fn generate_particle_bind_groups(
        gpu: &Gpu,
        compute_bind_group_layout: &wgpu::BindGroupLayout,
        particle_buffers: &Vec<wgpu::Buffer>,
    ) -> Vec<wgpu::BindGroup> {
        let mut particle_bind_groups = Vec::<wgpu::BindGroup>::new();
        for i in 0..2 {
            particle_bind_groups.push(gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: particle_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: particle_buffers[(i + 1) % 2].as_entire_binding(), // bind to opposite buffer
                    },
                ],
                label: None,
            }));
        }
        particle_bind_groups
    }

    pub fn new(gpu: &Gpu, fat_cam: &FatCamera, particle_data: &Vec<Particle>) -> Self {
        let quad_vertex_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Quad vertex buffer"),
                contents: bytemuck::cast_slice(&QUAD_VERTICES),
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            });

        let quad_index_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Quad Index Buffer"),
                contents: bytemuck::cast_slice(QUAD_INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        let diffuse_bytes = include_bytes!("particle_texture.png");
        let particle_texture = Texture::from_bytes(
            &gpu.device,
            &gpu.queue,
            diffuse_bytes,
            "ParticleTexture.png",
        )
        .unwrap(); // CHANGED!

        let depth_texture =
            Texture::create_depth_texture(&gpu.device, &gpu.config, "depth_texture");

        let texture_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        let texture_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&particle_texture.view), // CHANGED!
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&particle_texture.sampler), // CHANGED!
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let particle_buffers = Self::generate_particle_buffers(gpu, particle_data);

        let render_pipeline = Self::build_render_pipeline(gpu, &texture_bind_group_layout, fat_cam);
        let (compute_bind_group_layout, compute_pipeline) =
            Self::build_compute_pipeline(gpu, particle_data.len());
        let particle_bind_groups =
            Self::generate_particle_bind_groups(gpu, &compute_bind_group_layout, &particle_buffers);

        let work_group_count =
            ((particle_data.len() as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;
        ParticleGPU {
            quad_vertex_buffer,
            quad_index_buffer,
            particle_buffers,
            particle_texture,
            texture_bind_group,
            texture_bind_group_layout,
            particle_bind_groups,
            render_pipeline,
            compute_pipeline,
            work_group_count,
            depth_texture,
        }
    }

    fn build_compute_pipeline(
        gpu: &Gpu,
        num_particles: usize,
    ) -> (wgpu::BindGroupLayout, wgpu::ComputePipeline) {
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
                                min_binding_size: wgpu::BufferSize::new((num_particles * 48) as _),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new((num_particles * 48) as _),
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

        let compute_pipeline =
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Compute pipeline"),
                    layout: Some(&compute_pipeline_layout),
                    module: &compute_shader,
                    entry_point: "main",
                });
        (compute_bind_group_layout, compute_pipeline)
    }

    fn build_render_pipeline(
        gpu: &Gpu,
        texture_bgl: &wgpu::BindGroupLayout,
        fat_cam: &FatCamera,
    ) -> wgpu::RenderPipeline {
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

                    bind_group_layouts: &[texture_bgl, &fat_cam.bind_group_layout],
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
                            array_stride: 4 * 6,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![3 => Float32x4, 4 => Float32x2],
                        },
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        // 4.
                        format: gpu.config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less, // 1.
                    stencil: wgpu::StencilState::default(), // 2.
                    bias: wgpu::DepthBiasState::default(),
                }), // 1.
                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview: None, // 5.
            });
        render_pipeline
    }
}
