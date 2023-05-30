use std::{f32::consts::PI, rc::Rc};

use cgmath::{Vector3, Zero};

use crate::{consts::*, context::Context, rendering::uniform::Uniform};

pub struct Sky {
    uniform: Uniform<SkyUniform>,
    time: f32,
}

#[repr(C, align(16))]
pub struct SkyUniform {
    pub sun_direction: Vector3<f32>,
    pub time: f32,
    pub sun_light: f32,
}

impl Sky {
    pub fn new(context: Rc<Context>) -> Self {
        let uniform = SkyUniform {
            sun_direction: Vector3::zero(),
            time: 0.,
            sun_light: 1.,
        };

        Sky {
            uniform: Uniform::new(context, "Sky Uniform", uniform),
            time: 0.,
        }
    }

    fn get_uniform_data(&self) -> SkyUniform {
        let angle = self.time * 2. * PI;
        let sun_direction = Vector3::new(0., angle.cos(), angle.sin());

        let dayness = ((0.5 * PI * sun_direction.y).sin() + 1.) / 2.;
        let sun_light = dayness * dayness * (1. - MIDNIGHT_SUNLIGHT) + MIDNIGHT_SUNLIGHT;

        SkyUniform {
            sun_direction,
            time: self.time,
            sun_light,
        }
    }

    pub fn update(&mut self, delta: std::time::Duration) {
        self.time += delta.as_secs_f32() / DAY_LENGTH_SECS;
        self.uniform.write(self.get_uniform_data());
    }

    #[inline]
    pub fn get_uniform(&self) -> &Uniform<SkyUniform> {
        &self.uniform
    }
}
