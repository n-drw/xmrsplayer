/// A Sample State
use crate::helper::*;
use xmrs::prelude::*;
use xmrs::sample::Sample;

#[cfg(feature = "micromath")]
#[allow(unused_imports)]
use micromath::F32Ext;
#[cfg(feature = "libm")]
#[allow(unused_imports)]
use num_traits::float::Float;

#[derive(Clone)]
pub struct StateSample<'a> {
    sample: &'a Sample,
    finetune: f32,
    /// current seek position
    position: f64,
    /// step is freq / rate
    step: f64,
    /// For ping-pong samples: true is -->, false is <--
    ping: bool,
    // Output frequency
    rate: f32,
}

impl<'a> StateSample<'a> {
    pub fn new(sample: &'a Sample, rate: f32) -> Self {
        let position = if sample.len() == 0 { -1.0 } else { 0.0 };
        let finetune = sample.finetune;
        Self {
            sample,
            finetune,
            position,
            step: 0.0,
            ping: true,
            rate,
        }
    }

    pub fn reset(&mut self) {
        self.position = if self.sample.len() == 0 { -1.0 } else { 0.0 };
        self.ping = true;
    }

    pub fn set_step(&mut self, frequency: f32) {
        self.step = frequency as f64 / self.rate as f64;
    }

    pub fn set_position(&mut self, position: usize) {
        if position >= self.sample.len() {
            self.disable();
        } else {
            self.position = position as f64;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.position >= 0.0
    }

    pub fn disable(&mut self) {
        self.position = -1.0;
    }

    pub fn get_panning(&self) -> f32 {
        self.sample.panning
    }

    pub fn get_volume(&self) -> f32 {
        self.sample.volume
    }

    /// use sample finetune or force if finetune arg!=0
    pub fn get_finetuned_note(&self) -> f32 {
        self.sample.relative_note as f32 + self.finetune
    }

    pub fn set_finetune(&mut self, finetune: f32) {
        self.finetune = finetune;
    }    

    fn tick(&mut self) -> (f32, f32) {
        let t= self.position.fract() as f32;

        match self.sample.meta_at(self.position as usize) {
            Some( (_pos, u) ) => {
                self.position += self.step;
                match self.sample.meta_at(self.position as usize + 1) {
                    Some( (pos2, v)) => {
                        return (lerp(u.0, v.0, t), lerp(u.1, v.1, t));
                    },
                    None => {
                        self.disable();
                        return u;
                    }
                }

            },
            None => {
                #[cfg(feature = "std")]
                println!("This can't happen?");
                self.disable();
                return (0.0, 0.0);  // FIXME: may create crack?
            }
        }
    }
}

impl<'a> Iterator for StateSample<'a> {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= 0.0 {
            Some(self.tick())
        } else {
            None
        }
    }
}
