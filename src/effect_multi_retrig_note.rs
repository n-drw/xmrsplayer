use crate::effect::*;
use crate::helper::*;
use core::default::Default;

#[cfg(feature = "micromath")]
#[allow(unused_imports)]
use micromath::F32Ext;
#[cfg(feature = "libm")]
#[allow(unused_imports)]
use num_traits::float::Float;

#[derive(Clone, Default)]
pub struct MultiRetrigNote {
    speed: f32,
    vol_change: u8,
}

impl MultiRetrigNote {
    fn value(&self, vol: f32) -> f32 {
        return match self.vol_change {
            0x1 => vol - 1.0/64.0,
            0x2 => vol - 2.0/64.0,
            0x3 => vol - 4.0/64.0,
            0x4 => vol - 8.0/64.0,
            0x5 => vol - 16.0/64.0,
            0x6 => vol * (2.0/3.0)/64.0,
            0x7 => vol * (1.0/2.0)/64.0,
            0x8 => vol,   // does not change the volume
            0x9 => vol + 1.0/64.0,
            0xA => vol + 2.0/64.0,
            0xB => vol + 4.0/64.0,
            0xC => vol + 8.0/64.0,
            0xD => vol + 16.0/64.0,
            0xE => vol * (3.0/2.0)/64.0,
            0xF => vol * (2.0/1.0)/64.0,
            _ => vol,
        };
    }
}

#[derive(Clone, Default)]
pub struct EffectMultiRetrigNote {
    data: MultiRetrigNote,
    tick: f32,
}

impl EffectMultiRetrigNote {
    pub fn new(speed: f32, vol_change: u8) -> Self {
        Self {
            data: MultiRetrigNote {
                speed,
                vol_change,
            },
            tick: 0.0,
        }
    }
}

impl EffectPlugin for EffectMultiRetrigNote {
    fn tick0(&mut self, speed: f32, vol_change: f32) -> f32 {
        self.data.speed = speed;
        if vol_change != 0.0 {
            self.data.vol_change = vol_change as u8;
        }
        self.tick = 1.0;
        self.value()
    }

    fn tick(&mut self) -> f32 {
        self.tick += 1.0;
        self.tick %= self.data.speed;
        self.tick
    }

    fn in_progress(&self) -> bool {
        self.data.speed != 0.0
    }

    fn retrigger(&mut self) -> f32 {
        self.tick = 0.0;
        0.0
    }

    fn clamp(&self, vol: f32) -> f32 {
        if self.tick as f32 >= self.data.speed {
            vol
        } else {
            let mut v = self.data.value(vol);
            clamp(&mut v);
            v
        }
    }

    fn value(&self) -> f32 {
        0.0
    }
}

impl EffectXM2EffectPlugin for EffectMultiRetrigNote {
    fn xm_convert(param: u8, _special: u8) -> Option<(Option<f32>, Option<f32>)> {
        let speed = if param & 0x0F == 0 {
            None
        } else {
            Some((param & 0x0F) as f32)
        };

        let vol = if param >> 4 == 0 {
            None
        } else {
            Some((param >> 4) as f32 / 16.0)
        };

        if speed != None || vol != None {
            Some((speed, vol))
        } else {
            None
        }
    }

    fn xm_update_effect(&mut self, param: u8, _special1: u8, _special2: f32) {
        match EffectMultiRetrigNote::xm_convert(param, 0) {
            Some(elem) => {
                if let Some(speed) = elem.0 {
                    self.data.speed = speed;
                }
                if let Some(vol_change) = elem.1 {
                    self.data.vol_change = (16.0 * vol_change) as u8;
                }
            }
            None => {}
        }
    }
}
