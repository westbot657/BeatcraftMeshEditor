use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OptVec2(pub [Option<f32>; 2]);
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OptVec3(pub [Option<f32>; 3]);
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OptVec4(pub [Option<f32>; 4]);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionModName {
    Chroma,

    AudioLink,

    #[serde(untagged)]
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementModName {
    Chroma,
    #[serde(rename = "Noodle Extensions")]
    Noodle,
    Vivify,
    //////,
    #[serde(rename = "Mapping Extensions")]
    MappingExtensions,

    AudioLink,

    #[serde(untagged)]
    Custom(String),
}

impl SuggestionModName {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Chroma => "Chroma",
            Self::AudioLink => "AudioLink",
            Self::Custom(s) => s.as_str(),
        }
    }
}

impl RequirementModName {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Chroma => "Chroma",
            Self::Noodle => "Noodle Extensions",
            Self::Vivify => "Vivify",
            //////////// => "////////",
            Self::MappingExtensions => "Mapping Extensions",
            Self::AudioLink => "AudioLink",
            Self::Custom(s) => s.as_str(),
        }
    }
}

