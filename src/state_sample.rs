/// A Sample State
use crate::helper::*;
use xmrs::sample::Sample;

#[cfg(feature = "micromath")]
#[allow(unused_imports)]
use micromath::F32Ext;
#[cfg(feature = "libm")]
#[allow(unused_imports)]
use num_traits::float::Float;

#[cfg(feature = "use_f64")]
type FixedOrFloat = f64;

#[cfg(not(feature = "use_f64"))]
type FixedOrFloat = u32;

/*
with u32, sample size max:
 6 : 26 bits = 64 MB
 7 : 25 bits = 32 MB
 8 : 24 bits = 16 MB
 9 : 23 bits =  8 MB
10 : 22 bits =  4 MB
11 : 21 bits =  2 MB
12 : 20 bits =  1 MB
*/
#[cfg(not(feature = "use_f64"))]
const M: FixedOrFloat = 8; // multiplicator (2^M): Here we choose 8 because 32 - 8 = 24 bits <=> 2^24 = 16 MB compatible with historical maximum ft2 sample size.

#[derive(Clone)]
pub struct StateSample<'a> {
    sample: &'a Sample,
    finetune: f32,
    /// current seek position
    position: FixedOrFloat,
    /// step is freq / rate
    step: Option<FixedOrFloat>,
    // Output frequency
    rate: f32,
}

impl<'a> StateSample<'a> {
    pub fn new(sample: &'a Sample, rate: f32) -> Self {
        let position = StateSample::default_position();
        let finetune = sample.finetune;
        Self {
            sample,
            finetune,
            position,
            step: None,
            rate,
        }
    }

    #[inline(always)]
    fn default_position() -> FixedOrFloat {
        #[cfg(feature = "use_f64")]
        {
            0.0
        }
        #[cfg(not(feature = "use_f64"))]
        {
            0
        }
    }

    pub fn reset(&mut self) {
        self.position = StateSample::default_position();
        self.step = None;
    }

    pub fn set_step(&mut self, frequency: f32) {
        if self.sample.len() == 0 {
            self.disable();
        } else {
            #[cfg(feature = "use_f64")]
            {
                self.step = Some(frequency as FixedOrFloat / self.rate as FixedOrFloat);
            }
            #[cfg(not(feature = "use_f64"))]
            {
                self.step = Some(((1 << M) as f32 * (frequency / self.rate)) as FixedOrFloat);
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.step.is_some()
    }

    pub fn disable(&mut self) {
        self.step = None;
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
        let useek = self.sample.meta_seek(self.get_position() as usize);
        let u = self.sample.at(useek.1);
        #[cfg(feature = "use_f64")]
        {
            let t = self.get_position_fraction();
            let vseek = self.sample.meta_seek(self.get_position() as usize + 1);
            let v = self.sample.at(vseek.1);
            self.increment_position();
            return (lerp(u.0, v.0, t as f32), lerp(u.1, v.1, t as f32));
        }
        #[cfg(not(feature = "use_f64"))]
        {
            self.position = ((useek.0 as FixedOrFloat) << M) | self.get_position_fraction(); // update current to the smallest position
            let t = self.get_position_fraction() as f32 / (1 << M) as f32;
            let vseek = self.sample.meta_seek(self.get_position() as usize + 1);
            let v = self.sample.at(vseek.1);
            self.increment_position();
            return (lerp(u.0, v.0, t), lerp(u.1, v.1, t));
        }
    }

    pub fn set_position(&mut self, position: usize) {
        if position >= self.sample.len() {
            self.disable();
        } else {
            #[cfg(feature = "use_f64")]
            {
                self.position = position as FixedOrFloat;
            }
            #[cfg(not(feature = "use_f64"))]
            {
                self.position = (position << M) as FixedOrFloat;
            }
        }
    }

    #[inline(always)]
    fn increment_position(&mut self) -> FixedOrFloat {
        if let Some(step) = self.step {
            #[cfg(feature = "use_f64")]
            {
                self.position += step;
            }
            #[cfg(not(feature = "use_f64"))]
            {
                self.position += step;
            }
        }
        return self.position;
    }

    #[inline(always)]
    fn get_position(&mut self) -> FixedOrFloat {
        #[cfg(feature = "use_f64")]
        {
            return self.position;
        }
        #[cfg(not(feature = "use_f64"))]
        {
            return self.position >> M;
        }
    }

    #[inline(always)]
    fn get_position_fraction(&self) -> FixedOrFloat {
        #[cfg(feature = "use_f64")]
        {
            self.position.fract()
        }
        #[cfg(not(feature = "use_f64"))]
        {
            self.position & ((1 << M) - 1)
        }
    }
}

impl<'a> Iterator for StateSample<'a> {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_enabled() {
            Some(self.tick())
        } else {
            None
        }
    }
}
