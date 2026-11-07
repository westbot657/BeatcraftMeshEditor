use glam::Vec3;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl HitBox {
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
}

pub trait Lerp<V>
where
    V: Copy
        + std::ops::Mul<Output = V>
        + std::ops::Div<Output = V>
        + std::ops::Add<Output = V>
        + std::ops::Sub<Output = V>
{
    fn lerp(a: V, b: V, t: V) -> V {
        a + (b - a) * t
    }
    fn inv_lerp(a: V, b: V, x: V) -> V {
        (x - a) / (b - a)
    }
}

impl Lerp<f32> for f32 {}

impl Lerp<f64> for f64 {}