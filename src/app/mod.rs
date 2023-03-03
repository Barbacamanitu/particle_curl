pub mod gpu;
pub mod input;
pub mod math;
pub mod particle_gpu;
pub mod particle_system;
pub mod time;

use std::time::Duration;

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use self::{gpu::Gpu, input::Input, math::UVec2, particle_system::ParticleSystem, time::Time};

pub struct App {
    input: Input,
    pub time: Time,
    pub particle_system: ParticleSystem,
    size: UVec2,
}

impl App {
    pub fn new(sim_size: UVec2, gpu: &Gpu) -> App {
        //let mut renderer = Renderer::new(&gpu, sim_size);
        let mut particle_system = ParticleSystem::new(&gpu, sim_size);
        let time = Time::new(
            1,
            Duration::from_secs(1),
            Duration::from_millis(10),
            Duration::from_millis(1),
        );

        let input = Input::new();
        App {
            input,
            time,
            particle_system,
            size: sim_size,
        }
    }

    pub fn handle_input(&mut self, event: &WindowEvent) {
        self.input.handle_input(event);
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
    }
}
