use std::time::Duration;

use cgmath::{InnerSpace, Rad, Vector3};

use crate::app::input::Input;

use std::f32::consts::FRAC_PI_2;

use super::orbit::OrbitCamera;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct OrbitCameraController {
    horizontal: f32,
    vertical: f32,
    inout: f32,
    speed: f32,
    sensitivity: f32,
}

impl OrbitCameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            horizontal: 0.0,
            vertical: 0.0,
            inout: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_input(&mut self, input: &Input) {
        let m = input.movement;
        self.horizontal = input.mouse_delta.x * self.sensitivity;
        self.vertical = input.mouse_delta.y * self.sensitivity;
        self.inout = m.z;
    }

    pub fn update_camera(&mut self, camera: &mut OrbitCamera, dt: Duration) {}
}
