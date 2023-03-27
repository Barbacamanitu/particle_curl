pub mod controller;
pub mod projection;
use std::time::Duration;

use cgmath::{InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Vector3};
use wgpu::util::DeviceExt;

use self::{controller::FPSCameraController, projection::Projection};

use super::{gpu::Gpu, math::UVec2};

#[derive(Debug)]
pub struct FPSCamera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

pub struct CameraMatrices {
    pub view: CameraMatrix,
    pub projection: CameraMatrix,
    pub view_inverse: CameraMatrix,
}

pub struct CameraMatrixBuffers {
    pub view: wgpu::Buffer,
    pub projection: wgpu::Buffer,
    pub view_inverse: wgpu::Buffer,
}

pub struct FatCamera {
    pub camera: FPSCamera,
    pub controller: FPSCameraController,
    pub projection: Projection,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub matrices: CameraMatrices,
    pub matrix_buffers: CameraMatrixBuffers,
}

impl FatCamera {
    fn calc_camera_matrices(&self) -> CameraMatrices {
        let view_matrix = CameraMatrix::from_camera(&self.camera);
        println!("View matrix: {:?}", view_matrix);
        CameraMatrices {
            view: view_matrix,
            projection: CameraMatrix::from_projection(&self.projection),
            view_inverse: CameraMatrix::from_camera_inverse(&self.camera),
        }
    }
    pub fn update_camera(&mut self, gpu: &Gpu) {
        self.controller
            .update_camera(&mut self.camera, Duration::from_secs_f32(1.0 / 60.0));
        self.matrices = self.calc_camera_matrices();
        gpu.queue.write_buffer(
            &self.matrix_buffers.view,
            0,
            bytemuck::cast_slice(&[self.matrices.view]),
        );
        gpu.queue.write_buffer(
            &self.matrix_buffers.projection,
            0,
            bytemuck::cast_slice(&[self.matrices.projection]),
        );
        gpu.queue.write_buffer(
            &self.matrix_buffers.view_inverse,
            0,
            bytemuck::cast_slice(&[self.matrices.view_inverse]),
        );
    }

    fn create_matrix_buffers(gpu: &Gpu, matrices: &CameraMatrices) -> CameraMatrixBuffers {
        let view_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera View Matrix Buffer"),
                contents: bytemuck::cast_slice(&[matrices.view]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let projection_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera View Matrix Buffer"),
                contents: bytemuck::cast_slice(&[matrices.projection]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let view_inverse_buffer =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera View Matrix Buffer"),
                    contents: bytemuck::cast_slice(&[matrices.view_inverse]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
        CameraMatrixBuffers {
            view: view_buffer,
            projection: projection_buffer,
            view_inverse: view_inverse_buffer,
        }
    }

    pub fn new(
        size: UVec2,
        gpu: &Gpu,
        speed: f32,
        sensitivity: f32,
        fovy: cgmath::Deg<f32>,
    ) -> FatCamera {
        let camera = FPSCamera::new((0.0, 0.0, 100.0), cgmath::Deg(-90.0), cgmath::Deg(0.0));
        let projection = Projection::new(size.x, size.y, fovy, 0.0001, 100000.0);
        let controller = FPSCameraController::new(speed, sensitivity);
        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                    label: Some("camera_bind_group_layout"),
                });

        let matrices = CameraMatrices {
            view: CameraMatrix::from_camera(&camera),
            projection: CameraMatrix::from_projection(&projection),
            view_inverse: CameraMatrix::from_camera_inverse(&camera),
        };
        let matrix_buffers = FatCamera::create_matrix_buffers(&gpu, &matrices);
        let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: matrix_buffers.view.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: matrix_buffers.projection.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: matrix_buffers.view_inverse.as_entire_binding(),
                },
            ],
            label: Some("camera_bind_group"),
        });
        FatCamera {
            camera,
            controller,
            projection,
            bind_group_layout,
            bind_group: camera_bind_group,
            matrices,
            matrix_buffers,
        }
    }
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraMatrix {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    mat: [[f32; 4]; 4],
}

impl CameraMatrix {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            mat: cgmath::Matrix4::identity().into(),
        }
    }
    pub fn from_camera(camera: &FPSCamera) -> CameraMatrix {
        CameraMatrix {
            mat: (camera.calc_matrix()).into(),
        }
    }

    pub fn from_projection(projection: &Projection) -> CameraMatrix {
        CameraMatrix {
            mat: projection.calc_matrix().into(),
        }
    }

    pub fn from_camera_inverse(camera: &FPSCamera) -> CameraMatrix {
        let view_matrix = camera.calc_matrix();
        let inv = view_matrix.invert().unwrap();
        CameraMatrix { mat: inv.into() }
    }
}

impl FPSCamera {
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

        let direction =
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize();
        let mat = Matrix4::look_to_rh(self.position, direction, Vector3::unit_y());
        mat
    }
}
