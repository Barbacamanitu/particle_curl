//Low level gpu stuff for particles. Keeps main particle system file less cluttered.

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use super::gpu::Gpu;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Particle {
    pub position: [f32; 4],
    pub velocity: [f32; 4],
    pub color: [f32; 4],
}

const QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.01, 0.01, 0.0, 0.0],
    },
    Vertex {
        position: [-0.01, -0.01, 0.0, 0.0],
    },
    Vertex {
        position: [0.01, -0.01, 0.0, 0.0],
    },
    Vertex {
        position: [0.01, 0.01, 0.0, 0.0],
    },
];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

const PARTICLES: &[Particle] = &[
    Particle {
        position: [-0.5, 0.0, 0.0, 0.0],
        velocity: [0.01, 0.0, 0.0, 0.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Particle {
        position: [0.5, 0.0, 0.0, 0.0],
        velocity: [0.01, 0.01, 0.0, 0.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    Particle {
        position: [0.0, 0.25, 0.0, 0.0],
        velocity: [-0.01, 0.01, 0.0, 0.0],
        color: [0.0, 0.0, 1.0, 1.0],
    },
];

pub struct ParticleGPU {
    pub quad_vertex_buffer: wgpu::Buffer,
    pub quad_index_buffer: wgpu::Buffer,
    pub particle_position_buffer: wgpu::Buffer,
}

impl ParticleGPU {
    pub fn new(gpu: &Gpu) -> Self {
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

        let particle_position_buffer =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("particle buffer"),
                    contents: bytemuck::cast_slice(&PARTICLES),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST,
                });

        ParticleGPU {
            quad_vertex_buffer,
            quad_index_buffer,
            particle_position_buffer,
        }
    }
}
