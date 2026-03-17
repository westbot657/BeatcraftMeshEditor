use glam::Quat;
use serde::{Deserialize, Deserializer, Serialize, Serializer};


pub fn quat_to_value<S: Serializer>(value: &Quat, s: S) ->Result<S::Ok, S::Error> {
    value.to_array().serialize(s)
}

pub fn value_to_quat<'de, D: Deserializer<'de>>(d: D) -> Result<Quat, D::Error> {
    let v = <[f32; 4]>::deserialize(d)?;
    Ok(Quat::from_array(v))
}



