use winit::event::{MouseButton, WindowEvent};

use super::math::{FVec2, FVec3};

#[derive(Debug)]
pub struct Input {
    pub mouse_down: bool,
    pub movement: FVec3,
    pub mouse_delta: (usize, FVec2),
    last_mouse_pos: FVec2,
    pub scroll_delta: f32,
}

impl Input {
    pub fn new() -> Input {
        Input {
            mouse_down: false,
            movement: FVec3::default(),
            mouse_delta: (0, FVec2::default()),
            last_mouse_pos: FVec2::default(),
            scroll_delta: 0.0,
        }
    }

    pub fn clear(&mut self, render_ticks: usize) {
        let (last_tick, delta) = self.mouse_delta;
        if render_ticks > last_tick {
            //Clear mouse move state if there's been a tick since the last update
            self.mouse_delta = (render_ticks, FVec2::default());
        }
    }

    pub fn mouse_delta(&self) -> FVec2 {
        self.mouse_delta.1
    }

    #[allow(deprecated)]
    pub fn handle_input(&mut self, event: &WindowEvent, render_ticks: usize) {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                input,
                is_synthetic: _,
            } => match input.state {
                winit::event::ElementState::Pressed => {
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::A) {
                        self.movement.x = -1.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::D) {
                        self.movement.x = 1.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::W) {
                        self.movement.z = 1.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::S) {
                        self.movement.z = -1.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Q) {
                        self.movement.y = 1.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::E) {
                        self.movement.y = -1.0;
                    }
                }
                winit::event::ElementState::Released => {
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::A) {
                        self.movement.x = 0.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::D) {
                        self.movement.x = 0.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::W) {
                        self.movement.z = 0.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::S) {
                        self.movement.z = 0.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Q) {
                        self.movement.y = 0.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::E) {
                        self.movement.y = 0.0;
                    }
                }
            },
            WindowEvent::CursorMoved {
                device_id: _,
                position,
                modifiers: _,
            } => {
                //Update mouse position
                let mouse_pos = FVec2::new(position.x as f32, position.y as f32);

                if self.mouse_down {
                    self.mouse_delta = (
                        render_ticks,
                        (self.last_mouse_pos - mouse_pos) * FVec2::new(-1.0, 1.0),
                    );
                } else {
                    self.mouse_delta = (render_ticks, FVec2::default());
                }
                self.last_mouse_pos = mouse_pos;
            }

            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
                modifiers: _,
            } => match delta {
                winit::event::MouseScrollDelta::LineDelta(_x, y) => {
                    self.scroll_delta = *y;
                }
                winit::event::MouseScrollDelta::PixelDelta(_pos) => {}
            },
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
                modifiers: _,
            } => match state {
                winit::event::ElementState::Pressed => match button {
                    MouseButton::Right => {
                        self.mousedown();
                    }
                    _ => {}
                },
                winit::event::ElementState::Released => match button {
                    MouseButton::Right => {
                        self.mouseup(render_ticks);
                    }
                    _ => {}
                },
            },
            _ => {}
        }
    }
    pub fn mouseup(&mut self, render_ticks: usize) {
        self.mouse_down = false;
        self.mouse_delta = (render_ticks, FVec2::default());
    }

    fn mousedown(&mut self) {
        self.mouse_down = true;
    }
}
