use std::time::Duration;

use cgmath::{InnerSpace, Rad, Vector3};

use crate::app::input::Input;

use std::f32::consts::FRAC_PI_2;

use super::FPSCamera;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct FPSCameraController {
    amount_right: f32,
    amount_forward: f32,
    amount_up: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    speed: f32,
    sensitivity: f32,
}

impl FPSCameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_up: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_input(&mut self, input: &Input) {
        let m = input.movement;
        self.amount_forward = m.z;
        self.amount_right = m.x;
        self.amount_up = m.y;
        let delta = input.mouse_delta();
        self.rotate_horizontal = delta.x;
        self.rotate_vertical = -delta.y;
    }

    pub fn update_camera(&mut self, camera: &mut FPSCamera, dt: Duration) {
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
        camera.pitch += Rad(self.rotate_vertical) * -self.sensitivity * dt;

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
