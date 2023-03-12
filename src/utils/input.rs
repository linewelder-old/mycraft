use cgmath::Vector3;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

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

    pub fn update(&mut self, input: &KeyboardInput) {
        if let Some(code) = input.virtual_keycode {
            let state = input.state == ElementState::Pressed;
            if code == self.pos {
                self.pos_pressed = state;
            } else if code == self.neg {
                self.neg_pressed = state;
            }
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

pub struct Input3dDesc {
    pub pos_x: VirtualKeyCode,
    pub neg_x: VirtualKeyCode,
    pub pos_y: VirtualKeyCode,
    pub neg_y: VirtualKeyCode,
    pub pos_z: VirtualKeyCode,
    pub neg_z: VirtualKeyCode,
}

pub struct Input3d {
    input_x: Input1d,
    input_y: Input1d,
    input_z: Input1d,
}

impl Input3d {
    pub const fn new(desc: Input3dDesc) -> Self {
        let input_x = Input1d::new(desc.pos_x, desc.neg_x);
        let input_y = Input1d::new(desc.pos_y, desc.neg_y);
        let input_z = Input1d::new(desc.pos_z, desc.neg_z);

        Input3d {
            input_x,
            input_y,
            input_z,
        }
    }

    pub fn update(&mut self, input: &KeyboardInput) {
        self.input_x.update(input);
        self.input_y.update(input);
        self.input_z.update(input);
    }

    pub fn get_value(&self) -> Vector3<f32> {
        Vector3 {
            x: self.input_x.get_value(),
            y: self.input_y.get_value(),
            z: self.input_z.get_value(),
        }
    }
}
