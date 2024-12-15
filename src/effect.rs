pub trait EffectPlugin {
    fn tick0(&mut self, param1: f32, param2: f32) -> f32;
    fn tick(&mut self) -> f32;
    fn in_progress(&self) -> bool;
    fn retrigger(&mut self) -> f32;
    fn value(&self) -> f32;
}
