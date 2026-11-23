use glam::{Quat, Vec3, Vec4};
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Default)]
pub struct OptVec2(pub [Option<f32>; 2]);

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Default)]
pub struct OptVec3(pub [Option<f32>; 3]);

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Default)]
pub struct OptVec4(pub [Option<f32>; 4]);

macro_rules! impl_opt_vec {
    ($name:ident, $n:literal) => {
        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                self.0.serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct V;

                impl<'de> Visitor<'de> for V {
                    type Value = $name;

                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "a sequence of at most {} optional numbers", $n)
                    }

                    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: SeqAccess<'de>,
                    {
                        let mut out = [None; $n];
                        for slot in out.iter_mut() {
                            match seq.next_element::<Option<f32>>()? {
                                Some(v) => *slot = v,
                                None => break,
                            }
                        }
                        Ok($name(out))
                    }
                }

                deserializer.deserialize_seq(V)
            }
        }
    };
}

impl_opt_vec!(OptVec2, 2);
impl_opt_vec!(OptVec3, 3);
impl_opt_vec!(OptVec4, 4);

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColorVec4 {
    #[serde(untagged)]
    Vec4(Vec4),
    #[serde(untagged)]
    Vec3(Vec3),
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum RotationVec3 {
    #[serde(untagged)]
    Vec3(OptVec3),
    #[serde(untagged)]
    Y(f32),
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionModName {
    Chroma,

    AudioLink,

    #[serde(untagged)]
    Custom(String),
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl OptVec2 {
    pub fn is_some(&self) -> bool {
        self.0.iter().any(|v| v.is_some())
    }
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }
    pub fn x(self) -> Option<f32> {
        self.0[0]
    }
    pub fn y(self) -> Option<f32> {
        self.0[1]
    }
}
impl OptVec3 {
    pub fn is_some(&self) -> bool {
        self.0.iter().any(|v| v.is_some())
    }
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }
    pub fn x(self) -> Option<f32> {
        self.0[0]
    }
    pub fn y(self) -> Option<f32> {
        self.0[1]
    }
    pub fn z(self) -> Option<f32> {
        self.0[2]
    }
    pub fn to_quat(&self, rot: glam::EulerRot) -> Quat {
        Quat::from_euler(
            rot,
            self.x().unwrap_or(0.).to_radians(),
            self.y().unwrap_or(0.).to_radians(),
            -self.z().unwrap_or(0.).to_radians(),
        )
    }
}
impl OptVec4 {
    pub fn is_some(&self) -> bool {
        self.0.iter().any(|v| v.is_some())
    }
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }
    pub fn x(self) -> Option<f32> {
        self.0[0]
    }
    pub fn y(self) -> Option<f32> {
        self.0[1]
    }
    pub fn z(self) -> Option<f32> {
        self.0[2]
    }
    pub fn w(self) -> Option<f32> {
        self.0[3]
    }
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
