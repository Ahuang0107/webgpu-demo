use std::collections::HashMap;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Default)]
pub struct Input {
    keyboard_inputs: HashMap<KeyCode, KeyState>,
    cursor_pos: PhysicalPosition<f64>,
    mouse_inputs: HashMap<MouseButton, KeyState>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum KeyState {
    JustPressed,
    Pressed,
}

#[allow(unused)]
impl Input {
    pub fn cursor_pos(&self) -> &PhysicalPosition<f64> {
        &self.cursor_pos
    }

    pub fn if_keyboard_pressed(&self, key_code: &KeyCode) -> bool {
        self.keyboard_inputs
            .get(key_code)
            .map(|state| state == &KeyState::Pressed)
            .unwrap_or(false)
    }
    pub fn if_keyboard_just_pressed(&self, key_code: &KeyCode) -> bool {
        self.keyboard_inputs
            .get(key_code)
            .map(|state| state == &KeyState::JustPressed)
            .unwrap_or(false)
    }
    pub fn if_mouse_pressed(&self, mouse: &MouseButton) -> bool {
        self.mouse_inputs
            .get(mouse)
            .map(|state| state == &KeyState::Pressed)
            .unwrap_or(false)
    }
    pub fn if_mouse_just_pressed(&self, mouse: &MouseButton) -> bool {
        self.mouse_inputs
            .get(mouse)
            .map(|state| state == &KeyState::JustPressed)
            .unwrap_or(false)
    }
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    if !event.repeat {
                        match event.state {
                            ElementState::Pressed => {
                                if event.repeat {
                                    *self.keyboard_inputs.get_mut(&key_code).unwrap() =
                                        KeyState::Pressed;
                                } else {
                                    self.keyboard_inputs.insert(key_code, KeyState::JustPressed);
                                }
                            }
                            ElementState::Released => {
                                self.keyboard_inputs.remove(&key_code);
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { .. } => {}
            WindowEvent::MouseInput { button, state, .. } => match state {
                ElementState::Pressed => {
                    if let Some(mouse_state) = self.mouse_inputs.get_mut(&button) {
                        *mouse_state = KeyState::Pressed;
                    } else {
                        self.mouse_inputs.insert(*button, KeyState::JustPressed);
                    }
                }
                ElementState::Released => {
                    self.mouse_inputs.remove(&button);
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = *position;
            }
            _ => {}
        }
    }

    /// 经过一轮判断后，就需要将所有 just 的状态都修改掉
    pub fn fresh(&mut self) {
        for (_, state) in self.keyboard_inputs.iter_mut() {
            if state == &KeyState::JustPressed {
                *state = KeyState::Pressed;
            }
        }
        for (_, state) in self.mouse_inputs.iter_mut() {
            if state == &KeyState::JustPressed {
                *state = KeyState::Pressed;
            }
        }
    }
}
