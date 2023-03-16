use std::time::Duration;

use cgmath::{perspective, InnerSpace, Matrix4, Point3, Rad, Vector3};
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, VirtualKeyCode},
};

use std::f32::consts::FRAC_PI_2;

use super::{gpu::Gpu, input::Input, math::UVec2};
const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

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
        self.uniform
            .update_view_proj(&self.camera, &self.projection);
        gpu.queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
        println!("Camera: {:?}", self.camera);
    }

    pub fn new(size: UVec2, gpu: &Gpu) -> FatCamera {
        let camera = Camera::new((0.0, 0.0, -0.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = Projection::new(size.x, size.y, cgmath::Deg(45.0), 0.001, 10000.0);
        let controller = CameraController::new(0.5, 0.1);
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

    fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
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

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw),
            Vector3::unit_y(),
        )
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_right: f32,
    amount_forward: f32,
    amount_up: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_up: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_input(&mut self, input: &Input) {
        let m = input.movement;
        self.amount_forward = m.y;
        self.amount_right = m.x;
        self.rotate_horizontal = input.mouse_delta.x;
        self.rotate_vertical = -input.mouse_delta.y;
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();

        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward * (self.amount_forward) * self.speed * dt;
        camera.position += right * (self.amount_right) * self.speed * dt;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y += (self.amount_up) * self.speed * dt;

        // Rotate
        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}
