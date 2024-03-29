pub mod camera;
pub mod gpu;
pub mod input;
pub mod math;
pub mod particle_gpu;
pub mod particle_system;
pub mod texture;
pub mod time;
use std::time::Duration;

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use self::{
    camera::FatCamera, gpu::Gpu, input::Input, math::UVec2, particle_system::ParticleSystem,
    texture::Texture, time::Time,
};

pub struct App {
    pub time: Time,
    pub particle_system: ParticleSystem,
    size: UVec2,
    pub fat_cam: FatCamera,
    pub input: Input,
    frame: usize,
}

impl App {
    pub fn new(sim_size: UVec2, gpu: &Gpu) -> App {
        let fat_cam = FatCamera::new(
            sim_size,
            gpu,
            30.0,
            0.4,
            cgmath::Deg(90.0),
            (0.0, 0.0, 70.0).into(),
        );
        let particle_system = ParticleSystem::new(&gpu, sim_size, &fat_cam);
        let time = Time::new(Duration::from_secs_f32(1.0));

        let input = Input::new();
        App {
            time,
            particle_system,
            size: sim_size,
            fat_cam,
            input,
            frame: 0,
        }
    }

    pub fn handle_input(&mut self, event: &WindowEvent) {
        self.input.handle_input(event, self.time.render_ticks());
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, gpu: &mut Gpu) {
        if new_size.width > 0 && new_size.height > 0 {
            gpu.size = new_size;
            gpu.config.width = new_size.width;
            gpu.config.height = new_size.height;
            gpu.surface.configure(&gpu.device, &gpu.config);
        }
        self.size.x = new_size.width;
        self.size.y = new_size.height;
        self.fat_cam.projection.resize(self.size.x, self.size.y);
        self.particle_system.particle_gpu.depth_texture =
            Texture::create_depth_texture(&gpu.device, &gpu.config, "depth_texture");
    }

    pub fn tick(&mut self, gpu: &Gpu) {
        self.input.clear(self.time.render_ticks());
        self.fat_cam.controller.process_input(&self.input);
        self.fat_cam.update_camera(&gpu);
        self.particle_system
            .render(&gpu, &self.fat_cam, &mut self.time);
        let fps_data = self.time.get_fps();
        match fps_data {
            Some(fps) => println!("FPS: {}", fps.render_fps),
            None => {}
        }
    }
}
