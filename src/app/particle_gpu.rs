//Low level gpu stuff for particles. Keeps main particle system file less cluttered.

use std::time;

use bytemuck::{Pod, Zeroable};
use rand::Rng;
use wgpu::util::DeviceExt;

use super::{gpu::Gpu, texture::Texture};

pub const NUM_PARTICLES: usize = 55512;
pub const PARTICLES_PER_GROUP: u32 = 64;
const PARTICLE_SIZE: f32 = 0.02;
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

pub struct ParticleGPU {
    pub quad_vertex_buffer: wgpu::Buffer,
    pub quad_index_buffer: wgpu::Buffer,
    pub particle_buffers: Vec<wgpu::Buffer>,
    pub particle_bind_groups: Vec<wgpu::BindGroup>,
    pub particle_texture: Texture,
    pub texture_bind_group: wgpu::BindGroup,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl ParticleGPU {
    fn generate_particle_buffers(gpu: &Gpu) -> Vec<wgpu::Buffer> {
        let mut buffers: Vec<wgpu::Buffer> = Vec::new();
        let mut particle_data: Vec<Particle> = Vec::new();
        let mut rng = rand::thread_rng();

        let start = time::Instant::now();
        for i in 0..NUM_PARTICLES {
            let x = rng.gen_range(0.0..1.0) - 0.5;
            let y = rng.gen_range(0.0..1.0) - 0.5;
            let z = rng.gen_range(0.0..1.0) - 0.5;

            let x_v = rng.gen_range(0.0..0.2) - 0.1;
            let y_v = rng.gen_range(0.0..0.2) - 0.1;
            let z_v = 0.0;

            let r = rng.gen_range(0.0..1.0);
            let g = rng.gen_range(0.0..1.0);
            let b = rng.gen_range(0.0..1.0);
            particle_data.push(Particle {
                position: [x, y, z, 0.0],
                velocity: [x_v, y_v, z_v, 0.0],
                color: [r, g, b, 1.0],
            });
        }
        for i in 0..2 {
            buffers.push(
                gpu.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Particle Buffer {}", i)),
                        contents: bytemuck::cast_slice(&particle_data),
                        usage: wgpu::BufferUsages::VERTEX
                            | wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_DST,
                    }),
            );
        }
        let elapsed = start.elapsed().as_secs();
        println!(
            "Generated {} particles in {} seconds.",
            NUM_PARTICLES, elapsed
        );
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

    pub fn new(gpu: &Gpu, compute_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
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

        let particle_buffers = Self::generate_particle_buffers(gpu);

        let particle_bind_groups =
            Self::generate_particle_bind_groups(gpu, compute_bind_group_layout, &particle_buffers);
        ParticleGPU {
            quad_vertex_buffer,
            quad_index_buffer,
            particle_buffers,
            particle_texture,
            texture_bind_group,
            texture_bind_group_layout,
            particle_bind_groups,
        }
    }
}
