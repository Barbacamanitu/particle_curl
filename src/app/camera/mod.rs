pub mod controller;
pub mod projection;
use std::time::Duration;

use cgmath::{Matrix4, Point3, Rad, Vector3};
use wgpu::util::DeviceExt;

use self::{controller::CameraController, projection::Projection};

use super::{gpu::Gpu, math::UVec2};

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

pub struct FatCamera {
    pub camera: Camera,
    pub controller: CameraController,
    pub projection: Projection,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub uniform: CameraUniform,
}

impl FatCamera {
    pub fn update_camera(&mut self, gpu: &Gpu) {
        self.controller
            .update_camera(&mut self.camera, Duration::from_secs_f32(1.0 / 60.0));
        self.uniform = CameraUniform::from_camera(&self.camera, &self.projection);
        gpu.queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
        println!("Camera: {:?}", self.camera);
    }

    pub fn new(size: UVec2, gpu: &Gpu) -> FatCamera {
        let camera = Camera::new((0.0, 0.0, 1.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = Projection::new(
            size.x,
            size.y,
            cgmath::Deg(80.0),
            0.000001,
            10000000000000000000.0,
        );
        let controller = CameraController::new(2.5, 0.3);
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
        FatCamera {
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

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
    pub fn from_camera(camera: &Camera, projection: &Projection) -> CameraUniform {
        CameraUniform {
            view_proj: (projection.calc_matrix() * camera.calc_matrix()).into(),
        }
    }
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        let forward = Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw);

        Matrix4::look_to_rh(self.position, forward, Vector3::unit_y())
    }
}
