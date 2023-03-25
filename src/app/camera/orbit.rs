use std::time::Duration;

use cgmath::{Matrix4, Point3, Vector3};
use wgpu::util::DeviceExt;

use crate::app::{gpu::Gpu, math::UVec2};

use super::{orbit_controller::OrbitCameraController, projection::Projection, CameraUniform};

#[derive(Debug)]
pub struct OrbitCamera {
    pub horizontal: f32,
    pub vertical: f32,
    pub distance: f32,
}

impl OrbitCamera {
    pub fn position(&self) -> Point3<f32> {
        let phi = self.vertical;
        let theta = self.horizontal;
        let d = self.distance;
        let x = phi.sin() * theta.cos() * d;
        let y = phi.sin() * theta.sin() * d;
        let z = phi.cos() * d;
        [x, y, z].into()
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.position(), [0.0, 0.0, 0.0].into(), Vector3::unit_y())
    }
    pub fn new() -> OrbitCamera {
        OrbitCamera {
            horizontal: 0.0,
            vertical: 0.0,
            distance: 1.0,
        }
    }
}

pub struct FatOrbitCamera {
    pub camera: OrbitCamera,
    pub controller: OrbitCameraController,
    pub projection: Projection,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub uniform: CameraUniform,
}

impl FatOrbitCamera {
    pub fn update_camera(&mut self, gpu: &Gpu) {
        self.controller
            .update_camera(&mut self.camera, Duration::from_secs_f32(1.0 / 60.0));
        self.uniform = CameraUniform::from_orbit_camera(&self.camera, &self.projection);
        gpu.queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
        println!("Camera: {:?}", self.camera);
    }

    pub fn new(size: UVec2, gpu: &Gpu) -> FatOrbitCamera {
        let camera = OrbitCamera::new();
        let projection = Projection::new(
            size.x,
            size.y,
            cgmath::Deg(80.0),
            0.000001,
            10000000000000000000.0,
        );
        let controller = OrbitCameraController::new(2.5, 0.1);
        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });

        let uniform = CameraUniform::new();
        let camera_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        FatOrbitCamera {
            camera,
            controller,
            projection,
            bind_group_layout,
            buffer: camera_buffer,
            bind_group: camera_bind_group,
            uniform,
        }
    }
}
