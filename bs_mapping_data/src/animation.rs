use std::fmt::Formatter;
use std::str::FromStr;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::de::{Error, Visitor, SeqAccess};
use serde::ser::SerializeSeq;
use crate::easing::Easing;

/// WARNING:
/// serializing this is NOT the same as serializing the value of one of the variants.
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub enum Animation {
    #[cfg(feature = "v2")]
    V2(crate::animation_v2::AnimationV2),
    #[cfg(feature = "v3")]
    V3(crate::animation_v3::AnimationV3),
}
impl Animation {
    pub fn serialize_variant(&self) -> Result<String, serde_json::Error> {
        match self {
            #[cfg(feature = "v2")]
            Self::V2(v2) => serde_json::to_string(v2),
            #[cfg(feature = "v3")]
            Self::V3(v3) => serde_json::to_string(v3),
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "v2")]
impl From<crate::animation_v2::AnimationV2> for Animation {
    fn from(value: crate::animation_v2::AnimationV2) -> Self {
        Self::V2(value)
    }
}

#[cfg(feature = "v3")]
impl From<crate::animation_v3::AnimationV3> for Animation {
    fn from(value: crate::animation_v3::AnimationV3) -> Self {
        Self::V3(value)
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Spline {
    #[serde(rename = "splineCatmullRom")]
    CatmullRom,

    #[serde(untagged)]
    Unknown(String),
}

#[derive(thiserror::Error, Debug)]
#[error("value is probably not a spline: '{0}'")]
pub struct SplineParseError(String);

impl FromStr for Spline {
    type Err = SplineParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "splineCatmullRom" => Self::CatmullRom,
            s if s.starts_with("spline") => Self::Unknown(s.to_string()),
            _ => return Err(SplineParseError(s.to_string())),
        })
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PointValue {
    String(String),
    F32(f32),
    Array(Vec<Self>),
}


#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct PointDef {
    pub values: Vec<PointValue>,
    pub time: f32,
    pub easing: Option<Easing>,
    pub spline: Option<Spline>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlainPointDef {
    pub values: Vec<PointValue>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PointDefinition {
    Reference(String),
    SinglePoint(PlainPointDef),
    Points(Vec<PointDef>),
}

impl Serialize for PointDef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_seq(Some(
            self.values.len()
                + 1
                + if self.easing.is_some() { 1 } else { 0 }
                + if self.spline.is_some() { 1 } else { 0 }
        ))?;
        for val in self.values.iter() {
            s.serialize_element(val)?;
        }
        s.serialize_element(&self.time)?;
        if let Some(easing) = self.easing {
            s.serialize_element(&easing)?;
        }
        if let Some(spline) = self.spline.as_ref() {
            s.serialize_element(&spline)?;
        }
        s.end()
    }
}

impl<'de> Deserialize<'de> for PointDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(PointDefVisitor)
    }
}

struct PointDefVisitor;

impl<'de> Visitor<'de> for PointDefVisitor {
    type Value = PointDef;
    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "a tracks point definition")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = if let Some(l) = seq.size_hint() {
            Vec::with_capacity(l)
        } else {
            Vec::new()
        };
        while let Some(v) = seq.next_element()? {
            values.push(v);
        }

        let p: PointValue = values.pop().ok_or_else(|| A::Error::custom("Point definition expected at least 1 point value and a time value"))?;
        if values.is_empty() {
            return Err(A::Error::custom("Invalid Point definition structure"));
        }
        match &p {
            PointValue::F32(time) => Ok(PointDef {
                values,
                time: *time,
                easing: None,
                spline: None,
            }),
            PointValue::String(maybe_spline) => {
                if maybe_spline.starts_with("ease") {
                    let easing: Easing = maybe_spline.parse().map_err(|e| A::Error::custom(e))?;

                    let time: PointValue = values.pop().ok_or_else(|| A::Error::custom("Invalid Point definition structure"))?;

                    match &time {
                        PointValue::F32(time) => Ok(PointDef {
                            values,
                            time: *time,
                            easing: Some(easing),
                            spline: None,
                        }),
                        _ => Err(A::Error::custom("Point definition expected a time float value before easing value"))
                    }
                } else {
                    let spline: Spline = maybe_spline.parse().map_err(|e| A::Error::custom(e))?;

                    let p: PointValue = values.pop().ok_or_else(|| A::Error::custom("Invalid Point definition structure"))?;
                    if values.is_empty() {
                        return Err(A::Error::custom("Invalid Point definition structure"));
                    }
                    match &p {
                        PointValue::F32(time) => Ok(PointDef {
                            values,
                            time: *time,
                            easing: None,
                            spline: Some(spline),
                        }),
                        PointValue::String(s) => {
                            let easing: Easing = s.parse().map_err(|e| A::Error::custom(e))?;

                            let p: PointValue = values.pop().ok_or_else(|| A::Error::custom("Invalid Point definition structure"))?;
                            if values.is_empty() {
                                return Err(A::Error::custom("Invalid Point definition structure"));
                            }
                            match &p {
                                PointValue::F32(time) => Ok(PointDef {
                                    values,
                                    time: *time,
                                    easing: Some(easing),
                                    spline: Some(spline),
                                }),
                                _ => Err(A::Error::custom("Point definition expected a time float value"))
                            }
                        }
                        _ => Err(A::Error::custom("Point definition expected a time float value"))
                    }
                }
            }
            PointValue::Array(_) => Err(A::Error::custom("Point definition expected a time float value"))
        }


    }
}

