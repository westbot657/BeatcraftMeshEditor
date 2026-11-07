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