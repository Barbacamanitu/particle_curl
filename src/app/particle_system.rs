use std::mem;

use rand::Rng;
use wgpu::util::DeviceExt;

use super::camera::FatCamera;
use super::particle_gpu::{self, Particle, ParticleGPU};
use super::time::Time;
use super::{gpu::Gpu, math::UVec2};

pub const NUM_PARTICLES: usize = 1000000;
const SPAWN_SIZE: [f32; 3] = [100.0, 100.0, 0.0];

pub struct ParticleSystem {
    particle_gpu: ParticleGPU,
    pub size: UVec2,
}

impl ParticleSystem {
    fn create_particle_data() -> Vec<Particle> {
        let mut particle_data: Vec<Particle> = Vec::new();
        let mut rng = rand::thread_rng();
        let scalar = 100.0;
        for i in 0..NUM_PARTICLES {
            let x = (rng.gen_range(0.0..1.0) - 0.5) * scalar;
            let y = (rng.gen_range(0.0..1.0) - 0.5) * scalar;
            let z = (rng.gen_range(0.0..1.0) - 0.5) * 0.0;

            let x_v = 0.0;
            let y_v = 0.0;
            let z_v = 0.0;

            let r = 0.0;
            let g = 0.0;
            let b = 0.0;
            particle_data.push(Particle {
                position: [x, y, z, 1.0],
                velocity: [x_v, y_v, z_v, 0.0],
                color: [r, g, b, 1.0],
            });
        }
        particle_data
    }

    pub fn new(gpu: &Gpu, size: UVec2, fat_cam: &FatCamera) -> ParticleSystem {
        let particle_gpu = ParticleGPU::new(gpu, fat_cam, &Self::create_particle_data());

        ParticleSystem { particle_gpu, size }
    }

    pub fn render(&mut self, gpu: &Gpu, fat_cam: &FatCamera, time: &mut Time) {
        time.render_tick();
        self.run_compute(gpu, time);
        self.run_render(gpu, fat_cam, time);
    }

    fn run_render(&mut self, gpu: &Gpu, fat_cam: &FatCamera, time: &Time) {
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

            render_pass.set_pipeline(&self.particle_gpu.render_pipeline);
            render_pass.set_bind_group(0, &self.particle_gpu.texture_bind_group, &[]); // NEW!
            render_pass.set_bind_group(1, &fat_cam.bind_group, &[]);
            render_pass.set_vertex_buffer(
                0,
                self.particle_gpu.particle_buffers[(time.render_ticks() + 1) % 2].slice(..),
            );
            render_pass.set_vertex_buffer(1, self.particle_gpu.quad_vertex_buffer.slice(..));

            render_pass.set_index_buffer(
                self.particle_gpu.quad_index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            ); // 1.
            render_pass.draw_indexed(0..6, 0, 0..NUM_PARTICLES as u32);
            // 2.
        }
        encoder.pop_debug_group();
        gpu.queue.submit([encoder.finish()]);
        output.present();
    }

    fn run_compute(&mut self, gpu: &Gpu, time: &Time) {
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        encoder.push_debug_group("Particle System Compute");
        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            compute_pass.set_pipeline(&self.particle_gpu.compute_pipeline);
            compute_pass.set_bind_group(
                0,
                &self.particle_gpu.particle_bind_groups[time.render_ticks() % 2],
                &[],
            );

            compute_pass.dispatch_workgroups(self.particle_gpu.work_group_count, 1, 1);
        }
        encoder.pop_debug_group();
        gpu.queue.submit([encoder.finish()]);
    }
}
