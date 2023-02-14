use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

pub struct Input1d {
    pos: VirtualKeyCode,
    neg: VirtualKeyCode,

    pos_pressed: bool,
    neg_pressed: bool,
}

impl Input1d {
    pub const fn new(pos: VirtualKeyCode, neg: VirtualKeyCode) -> Self {
        Input1d {
            pos,
            neg,
            pos_pressed: false,
            neg_pressed: false,
        }
    }

    pub fn update(&mut self, event: &WindowEvent) -> bool {
        if let WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    state,
                    virtual_keycode: Some(code),
                    ..
                },
            ..
        } = event
        {
            let state = *state == ElementState::Pressed;
            if *code == self.pos {
                self.pos_pressed = state;
            } else if *code == self.neg {
                self.neg_pressed = state;
            } else {
                return false;
            }

            true
        } else {
            false
        }
    }

    pub fn get_value(&self) -> f32 {
        let mut value = 0.;
        if self.pos_pressed {
            value += 1.;
        }
        if self.neg_pressed {
            value -= 1.;
        }

        value
    }
}
