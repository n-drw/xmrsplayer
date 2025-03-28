pub trait EffectPlugin {
    fn tick0(&mut self, param1: f32, param2: f32) -> f32;
    fn tick(&mut self) -> f32;
    fn in_progress(&self) -> bool;
    fn retrigger(&mut self) -> f32;

    fn clamp(&self, value: f32) -> f32;
    fn value(&self) -> f32;
}

pub trait EffectXM2EffectPlugin {
    /// special is used for specific abstraction case
    fn xm_convert(param: u8, special: u8) -> Option<(Option<f32>, Option<f32>)>;
    fn xm_update_effect(&mut self, param: u8, special1: u8, special2: f32);
}
