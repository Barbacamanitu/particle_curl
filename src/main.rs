mod app;
use app::{gpu::Gpu, math::UVec2, App};

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::time::Duration;

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let sim_size = UVec2::new(1024, 1024);
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(sim_size.x, sim_size.y))
        .with_title("GPU_Particles")
        .with_position(PhysicalPosition::new(0, 0))
        .build(&event_loop)
        .unwrap();
    let mut gpu = Gpu::new(&window);
    let mut app = App::new(sim_size, &gpu);
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                //.Handle gui events

                app.handle_input(event);
                match event {
                    WindowEvent::Resized(physical_size) => {
                        app.resize(*physical_size, &mut gpu);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        app.resize(**new_inner_size, &mut gpu);
                    }
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                while app.time.can_update() {
                    app.particle_system.update(&gpu);
                    app.time.update_tick();
                }
                app.fat_cam.update_camera(&gpu);
                app.particle_system.render(&gpu, &app.fat_cam);
                app.time.render_tick();
                let fps_data = app.time.get_fps();
                match fps_data {
                    Some(fps) => println!(
                        "Render FPS: {}, Updates per second: {}",
                        fps.render_fps, fps.update_fps
                    ),
                    None => {}
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}
