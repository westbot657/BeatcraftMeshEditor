use ahash::HashMap;
use serde::{Deserialize, Serialize};
use crate::animation::PointDefinition;

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct AnimationV2 {
    #[serde(rename = "_position")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<PointDefinition>,
    #[serde(rename = "_localPosition")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_position: Option<PointDefinition>,
    #[serde(rename = "_rotation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<PointDefinition>,
    #[serde(rename = "_localRotation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_rotation: Option<PointDefinition>,
    #[serde(rename = "_scale")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<PointDefinition>,
    #[serde(rename = "_definitePosition")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definite_position: Option<PointDefinition>,
    #[serde(rename = "_dissolve")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolve: Option<PointDefinition>,
    #[serde(rename = "_dissolveArrow")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolve_arrow: Option<PointDefinition>,
    #[serde(rename = "_time")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<PointDefinition>,
    #[serde(rename = "_cuttable")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cuttable: Option<PointDefinition>,
    #[serde(rename = "_color")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<PointDefinition>,
    #[serde(rename = "_attenuation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attenuation: Option<PointDefinition>,
    #[serde(rename = "_fogOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fog_offset: Option<PointDefinition>,
    #[serde(rename = "_heightFogStartY")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_fog_start_y: Option<PointDefinition>,
    #[serde(rename = "_heightFogHeight")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_fog_height: Option<PointDefinition>,

    #[serde(flatten)]
    pub extra: Option<HashMap<String, PointDefinition>>,
}