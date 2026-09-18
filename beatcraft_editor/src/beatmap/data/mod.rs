use bs_mapping_data::CutDirection;
use glam::Quat;

pub mod obstacle_font;

pub trait CutDirectionExt {
    fn angle_degrees(&self) -> f32;
    fn world_angle_radians(&self) -> f32;
    fn to_quat(&self) -> Quat;
}
impl CutDirectionExt for CutDirection {
    fn angle_degrees(&self) -> f32 {
        match self {
            CutDirection::Up => 180f32,
            CutDirection::Down => 0.,
            CutDirection::Left => 90.,
            CutDirection::Right => -90.,
            CutDirection::UpLeft => 135.,
            CutDirection::UpRight => -135.,
            CutDirection::DownLeft => 45.,
            CutDirection::DownRight => -45.,
            CutDirection::Dot => 0.,
        }
    }
    fn world_angle_radians(&self) -> f32 {
        -self.angle_degrees().to_radians()
    }
    fn to_quat(&self) -> Quat {
        Quat::from_rotation_z(self.angle_degrees().to_radians())
    }
}
