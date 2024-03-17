use crate::Image;

pub struct Scene;

impl Image for Scene {
    fn render(&self, x: usize, y: usize) -> f32 {
        (x ^ y) as u8 as f32
    }
}
