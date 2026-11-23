use ahash::HashMap;
use serde::{Deserialize, Serialize};
use crate::animation::PointDefinition;

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationV3 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_position: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_world_rotation: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_rotation: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_position: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definite_position: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolve: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolve_arrow: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactable: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attenuation: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fog_offset: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_fog_start_y: Option<PointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_fog_height: Option<PointDefinition>,

    #[serde(flatten)]
    pub extra: Option<HashMap<String, PointDefinition>>,
}