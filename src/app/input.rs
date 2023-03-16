use winit::event::{MouseButton, WindowEvent};

use super::math::FVec2;

#[derive(Debug)]
pub struct Input {
    pub mouse_down: bool,
    pub movement: FVec2,
    pub mouse_delta: FVec2,
    last_mouse_pos: FVec2,
    pub scroll_delta: f32,
}

impl Input {
    pub fn new() -> Input {
        Input {
            mouse_down: false,
            movement: FVec2::default(),
            mouse_delta: FVec2::default(),
            last_mouse_pos: FVec2::default(),
            scroll_delta: 0.0,
        }
    }

    #[allow(deprecated)]
    pub fn handle_input(&mut self, event: &WindowEvent) {
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
                        self.movement.y = 1.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::S) {
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
                        self.movement.y = 0.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::S) {
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
                    self.mouse_delta = (self.last_mouse_pos - mouse_pos) * FVec2::new(-1.0, 1.0);
                    self.last_mouse_pos = mouse_pos;
                } else {
                    self.mouse_delta = FVec2::default();
                }
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
                        self.mouseup();
                    }
                    _ => {}
                },
            },
            _ => {}
        }
    }
    pub fn mouseup(&mut self) {
        self.mouse_down = false;
    }

    fn mousedown(&mut self) {
        self.mouse_down = true;
    }
}
