use std::collections::HashMap;
use bitflags::parser::ParseHex;
use bs_mapping_data::info_v4::{AudioInfoV4, ColorSchemeV4, DifficultyBeatmapV4, InfoV4, MapAuthorsV4, SongInfoV4};
use bs_mapping_data::{AnyEnvironment, Environment, InfoFile, InfoVersion, MapCharacteristic, MapDifficulty, RGBAColor};
use glam::Vec4;
use num_traits::ToPrimitive;
use bs_mapping_data::custom_info_v2::{DifficultyBeatmapCustomDataV2, DifficultySetCustomDataV2, InfoCustomDataV2};
use bs_mapping_data::info_v2::{CharacteristicSetV2, ColorSchemeInnerV2, ColorSchemeV2, DifficultyBeatmapV2, InfoV2};

/// General info that can coerce into V2 and V4
#[derive(Clone)]
pub struct Info {
    pub song: Option<SongInfo>,
    pub audio: Option<AudioInfo>,
    pub preview_filename: Option<String>,
    pub cover_filename: Option<String>,
    pub color_schemes: Vec<ColorScheme>,
    pub beatmaps: Vec<DifficultyBeatmap>,
    pub custom_data: Option<InfoCustomDataV2>,
    pub custom_characteristic_data: HashMap<MapCharacteristic, DifficultySetCustomDataV2>,
}

#[derive(Clone)]
pub struct SongInfo {
    pub title: String,
    pub subtitle: String,
    pub author: String,
}

#[derive(Clone)]
pub struct AudioInfo {
    pub filename: String,
    pub duration: Option<f32>,
    pub data_filename: Option<String>,
    pub bpm: f32,
    pub lufs: f32,
    pub preview_start: f32,
    pub preview_duration: f32,
}

#[derive(Clone)]
pub struct ColorSet {
    pub saber: Vec4,
    pub env: Vec4,
    pub boost: Vec4,
}

#[derive(Clone)]
pub struct ColorScheme {
    pub use_override: bool,
    pub name: String,
    pub obstacles: Vec4,
    pub left: ColorSet,
    pub right: ColorSet,
}

#[derive(Clone)]
pub struct DifficultyBeatmap {
    pub characteristic: MapCharacteristic,
    pub difficulty: MapDifficulty,
    pub authors: MapAuthors,
    pub environment: AnyEnvironment,
    pub color_scheme: Option<u32>,
    pub njs: f32,
    pub jump_offset: f32,
    pub filename: Option<String>,
    pub lightshow_filename: Option<String>,
    pub custom_data: Option<DifficultyBeatmapCustomDataV2>,
}

#[derive(Clone)]
pub struct MapAuthors {
    pub mappers: Vec<String>,
    pub lighters: Vec<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum InfoExportError {
    #[error("Missing element: {0}")]
    MissingComponent(String),
}

#[derive(thiserror::Error, Debug)]
pub enum InfoV2ExportError {
    #[error("{0}")]
    ExportError(#[from] InfoExportError),
    #[error("Multiple environments not supported in v2")]
    MultipleEnvironments(Vec<String>),
}

#[derive(thiserror::Error, Debug)]
pub enum InfoV4ExportError {
    #[error("{0}")]
    ExportError(#[from] InfoExportError),
}

impl TryFrom<Info> for InfoV2 {
    type Error = InfoV2ExportError;
    fn try_from(mut value: Info) -> Result<Self, Self::Error> {

        let Some(SongInfo {
            title,
            subtitle,
            author
        }) = value.song else {
            return Err(InfoExportError::MissingComponent("Song Info".to_string()).into());
        };

        let Some(AudioInfo {
            filename,
            duration: _,
            data_filename: _,
            bpm,
            lufs: _,
            preview_start,
            preview_duration
        }) = value.audio else {
            return Err(InfoExportError::MissingComponent("Audio Info".to_string()).into());
        };

        let Some(cover_image_filename) = value.cover_filename else {
            return Err(InfoExportError::MissingComponent("Cover Image".to_string()).into());
        };

        let color_schemes: Vec<ColorSchemeV2> = value.color_schemes
            .into_iter()
            .map(Into::into)
            .collect();

        let mut environment_names: Vec<AnyEnvironment> = Vec::new();
        let mut environment_360 = None;
        let mut beatmap_sets = HashMap::new();

        let mut mappers = Vec::new();
        let mut lighters = Vec::new();

        let mut default_rank = 10;
        for mut diff in value.beatmaps {
            for mapper in diff.authors.mappers {
                if !mappers.contains(&mapper) {
                    mappers.push(mapper);
                }
            }
            for lighter in diff.authors.lighters {
                if !lighters.contains(&lighter) {
                    lighters.push(lighter);
                }
            }

            let Some(beatmap_filename) = diff.filename else {
                return Err(InfoExportError::MissingComponent(format!("Beatmap File Name ({}, {})", diff.characteristic, diff.difficulty)).into())
            };

            let difficulty_rank = diff.difficulty.rank()
                .unwrap_or({ default_rank += 1; default_rank-1 });

            let list: &mut Vec<_> = beatmap_sets
                .entry(diff.characteristic)
                .or_default();

            let idx = environment_names
                .iter()
                .enumerate()
                .find_map(|(i, name)|
                    (name.data_name() == diff.environment.data_name())
                        .then_some(i)
                );

            if let AnyEnvironment::Env360(e) = &diff.environment {
                environment_360 = Some(e.clone());
            }

            let idx = match idx {
                Some(i) => i,
                None => {
                    let i = environment_names.len();
                    environment_names.push(diff.environment);
                    i
                }
            };

            list.push(DifficultyBeatmapV2 {
                difficulty: diff.difficulty,
                difficulty_rank,
                beatmap_filename,
                note_jump_movement_speed: diff.njs,
                note_jump_start_beat_offset: diff.jump_offset,
                custom_data: diff.custom_data,
                beatmap_color_scheme_index: diff.color_scheme,
                environment_name_index: Some(idx as u32),
            })
        }

        let mut difficulty_beatmap_sets = Vec::new();
        for (char, set) in beatmap_sets {
            difficulty_beatmap_sets.push(CharacteristicSetV2 {
                custom_data: value.custom_characteristic_data.remove(&char),
                beatmap_characteristic_name: char,
                difficulty_beatmaps: set,
            })
        }

        for lighter in lighters {
            if !mappers.contains(&lighter) {
                mappers.push(lighter);
            }
        }

        let environment = environment_names
            .iter()
            .find_map(|e| match e {
                AnyEnvironment::Standard(e) => Some(e.clone()),
                _ => None,
            })
            .unwrap_or_default();

        Ok(Self {
            version: InfoVersion::V2_1_0,
            song_name: title,
            song_sub_name: subtitle,
            song_author_name: author,
            level_author_name: mappers.join(", "),
            bpm,
            song_time_offset: 0.0,
            shuffle: 0.0,
            shuffle_period: 0.0,
            preview_start_time: preview_start,
            preview_duration,
            song_filename: filename,
            cover_image_filename,
            environment,
            environment_360,
            environment_names,
            color_schemes,
            custom_data: value.custom_data,
            difficulty_beatmap_sets,
        })
    }
}

impl TryFrom<Info> for InfoV4 {
    type Error = InfoV4ExportError;
    fn try_from(value: Info) -> Result<Self, Self::Error> {
        let Some(song) = value.song else {
            return Err(InfoExportError::MissingComponent("Song Info".to_string()).into());
        };

        let Some(audio) = value.audio else {
            return Err(InfoExportError::MissingComponent("Audio Info".to_string()).into());
        };

        let Some(audio_data_filename) = audio.data_filename else {
            return Err(InfoExportError::MissingComponent("Audio Data File".to_string()).into());
        };

        let song_preview_filename = value.preview_filename.unwrap_or_else(|| audio.filename.clone());

        let Some(cover_image_filename) = value.cover_filename else {
            return Err(InfoExportError::MissingComponent("Cover Image".to_string()).into());
        };

        let Some(song_duration) = audio.duration else {
            return Err(InfoExportError::MissingComponent("song duration".to_string()).into());
        };

        let color_schemes: Vec<ColorSchemeV4> = value.color_schemes
            .into_iter()
            .map(Into::into)
            .collect();

        let mut environment_names = Vec::new();
        let mut difficulty_beatmaps = Vec::new();

        for diff in value.beatmaps {
            let environment_name_idx = match environment_names.iter().enumerate().find(|(_, e)| *e == &diff.environment) {
                Some((idx, _)) => idx,
                None => {
                    let l = environment_names.len();
                    environment_names.push(diff.environment.clone());
                    l
                }
            } as u32;

            let Some(beatmap_data_filename) = diff.filename else {
                return Err(InfoExportError::MissingComponent(format!("Beatmap File Name ({}, {})", diff.characteristic, diff.difficulty)).into());
            };

            let Some(lightshow_data_filename) = diff.lightshow_filename else {
                return Err(InfoExportError::MissingComponent(format!("Lightshow File Name ({}, {})", diff.characteristic, diff.difficulty)).into());
            };


            difficulty_beatmaps.push(DifficultyBeatmapV4 {
                characteristic: diff.characteristic,
                difficulty: diff.difficulty,
                beatmap_authors: diff.authors.into(),
                environment_name_idx,
                beatmap_color_scheme_idx: diff.color_scheme.map(|u| u as i32).unwrap_or(-1),
                note_jump_movement_speed: diff.njs,
                note_jump_start_beat_offset: diff.jump_offset,
                beatmap_data_filename,
                lightshow_data_filename,
            });
        }

        Ok(Self {
            version: InfoVersion::V4_0_1,
            song: SongInfoV4 {
                title: song.title,
                sub_title: song.subtitle,
                author: song.author,
            },
            audio: AudioInfoV4 {
                song_filename: audio.filename,
                song_duration,
                audio_data_filename,
                bpm: audio.bpm,
                lufs: audio.lufs,
                preview_start_time: audio.preview_start,
                preview_duration: audio.preview_duration,
            },
            song_preview_filename,
            cover_image_filename,
            environment_names,
            color_schemes,
            difficulty_beatmaps,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InfoConversionError {
    #[error("{0}")]
    IntParse(#[from] core::num::ParseIntError),
    #[error("invalid environment index: {0}")]
    EnvIndex(u32)
}

impl TryFrom<InfoFile> for Info {
    type Error = InfoConversionError;
    fn try_from(value: InfoFile) -> Result<Self, Self::Error> {
        match value {
            InfoFile::V2(v2) => v2.try_into(),
            InfoFile::V4(v4) => v4.try_into(),
        }
    }
}

impl TryFrom<InfoV2> for Info {
    type Error = InfoConversionError;
    fn try_from(value: InfoV2) -> Result<Self, Self::Error> {

        let mut beatmaps = Vec::new();
        let mut custom_characteristic_data = HashMap::new();

        for set in value.difficulty_beatmap_sets {
            for diff in set.difficulty_beatmaps {

                let environment = value.environment_names
                    .get(diff.environment_name_index.unwrap_or_default() as usize)
                    .cloned()
                    .unwrap_or_default();
                
                beatmaps.push(DifficultyBeatmap {
                    characteristic: set.beatmap_characteristic_name.clone(),
                    difficulty: diff.difficulty,
                    authors: MapAuthors {
                        mappers: value.level_author_name
                            .split(",")
                            .map(|s| s.trim().to_string())
                            .collect(),
                        lighters: Vec::new(),
                    },
                    environment,
                    color_scheme: diff.beatmap_color_scheme_index,
                    njs: diff.note_jump_movement_speed,
                    jump_offset: diff.note_jump_start_beat_offset,
                    filename: Some(diff.beatmap_filename),
                    lightshow_filename: None,
                    custom_data: diff.custom_data,
                })
            }
            if let Some(cd) = set.custom_data {
                custom_characteristic_data.insert(set.beatmap_characteristic_name, cd);
            }
        }

        Ok(Self {
            song: Some(SongInfo {
                title: value.song_name,
                subtitle: value.song_sub_name,
                author: value.song_author_name,
            }),
            audio: Some(AudioInfo {
                filename: value.song_filename.clone(),
                duration: None,
                data_filename: None,
                bpm: value.bpm,
                lufs: 0.0,
                preview_start: value.preview_start_time,
                preview_duration: value.preview_duration,
            }),
            preview_filename: Some(value.song_filename),
            cover_filename: Some(value.cover_image_filename),
            color_schemes: value.color_schemes.into_iter().map(Into::into).collect(),
            beatmaps,
            custom_data: value.custom_data,
            custom_characteristic_data,
        })
    }
}

impl TryFrom<InfoV4> for Info {
    type Error = InfoConversionError;
    fn try_from(value: InfoV4) -> Result<Self, Self::Error> {

        let mut beatmaps = Vec::new();

        for map in value.difficulty_beatmaps {
            beatmaps.push(DifficultyBeatmap {
                characteristic: map.characteristic,
                difficulty: map.difficulty,
                authors: MapAuthors {
                    mappers: map.beatmap_authors.mappers,
                    lighters: map.beatmap_authors.lighters,
                },
                environment: value.environment_names
                    .get(map.environment_name_idx as usize)
                    .ok_or(InfoConversionError::EnvIndex(map.environment_name_idx))?.clone(),
                color_scheme: map.beatmap_color_scheme_idx.to_u32(),
                njs: map.note_jump_movement_speed,
                jump_offset: map.note_jump_start_beat_offset,
                filename: Some(map.beatmap_data_filename),
                lightshow_filename: Some(map.lightshow_data_filename),
                custom_data: None,
            })
        }

        Ok(Self {
            song: Some(SongInfo {
                title: value.song.title,
                subtitle: value.song.sub_title,
                author: value.song.author,
            }),
            audio: Some(AudioInfo {
                filename: value.audio.song_filename,
                duration: Some(value.audio.song_duration),
                data_filename: Some(value.audio.audio_data_filename),
                bpm: value.audio.bpm,
                lufs: value.audio.lufs,
                preview_start: value.audio.preview_start_time,
                preview_duration: value.audio.preview_duration,
            }),
            preview_filename: Some(value.song_preview_filename),
            cover_filename: Some(value.cover_image_filename),
            color_schemes: value.color_schemes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            beatmaps,
            custom_data: None,
            custom_characteristic_data: Default::default(),
        })
    }
}

trait Vec4ToHex {
    fn as_hex(&self) -> String;
    fn as_rgba(&self) -> RGBAColor;
}
impl Vec4ToHex for Vec4 {
    fn as_hex(&self) -> String {
        let r = ((self.x * 255.) as u32) << 24;
        let g = ((self.y * 255.) as u32) << 16;
        let b = ((self.z * 255.) as u32) << 8;
        let a = (self.w * 255.) as u32;
        let rgba = r | g | b | a;
        format!("{rgba:X}")
    }
    fn as_rgba(&self) -> RGBAColor {
        RGBAColor {
            r: self.x,
            g: self.y,
            b: self.z,
            a: self.w,
        }
    }
}
trait ToVec4 {
    fn to_vec4(&self) -> Result<Vec4, core::num::ParseIntError>;
}
impl ToVec4 for RGBAColor {
    fn to_vec4(&self) -> Result<Vec4, core::num::ParseIntError> {
        Ok(Vec4::new(self.r, self.g, self.b, self.a))
    }
}
impl ToVec4 for String {
    fn to_vec4(&self) -> Result<Vec4, core::num::ParseIntError> {
        u32::from_str_radix(self, 16).map(|v| {
            let r = (v >> 24) as f32 / 255.;
            let g = ((v >> 16) & 0xFF) as f32 / 255.;
            let b = ((v >> 8) & 0xFF) as f32 / 255.;
            let a = (v & 0xFF) as f32 / 255.;
            Vec4::new(r, g, b, a)
        })
    }
}

impl From<ColorScheme> for ColorSchemeV2 {
    fn from(value: ColorScheme) -> Self {
        Self {
            use_override: value.use_override,
            color_scheme: ColorSchemeInnerV2 {
                color_scheme_id: value.name,
                saber_a_color: value.left.saber.as_rgba(),
                saber_b_color: value.right.saber.as_rgba(),
                environment_color_0: value.obstacles.as_rgba(),
                environment_color_1: value.left.env.as_rgba(),
                obstacle_color: value.right.env.as_rgba(),
                environment_color_0_boost: value.left.boost.as_rgba(),
                environment_color_1_boost: value.right.boost.as_rgba(),
            },
        }
    }
}
impl From<ColorSchemeV2> for ColorScheme {
    fn from(value: ColorSchemeV2) -> Self {
        Self {
            use_override: value.use_override,
            name: value.color_scheme.color_scheme_id,
            obstacles: value.color_scheme.obstacle_color.to_vec4().unwrap(),
            left: ColorSet {
                saber: value.color_scheme.saber_a_color.to_vec4().unwrap(),
                env: value.color_scheme.environment_color_0.to_vec4().unwrap(),
                boost: value.color_scheme.environment_color_0_boost.to_vec4().unwrap(),
            },
            right: ColorSet {
                saber: value.color_scheme.saber_b_color.to_vec4().unwrap(),
                env: value.color_scheme.environment_color_1.to_vec4().unwrap(),
                boost: value.color_scheme.environment_color_1_boost.to_vec4().unwrap(),
            },
        }
    }
}
impl TryFrom<ColorSchemeV4> for ColorScheme {
    type Error = core::num::ParseIntError;
    fn try_from(value: ColorSchemeV4) -> Result<Self, Self::Error> {
        Ok(Self {
            use_override: value.use_override,
            name: value.color_scheme_name,
            obstacles: value.obstacles_color.to_vec4()?,
            left: ColorSet {
                saber: value.saber_a_color.to_vec4()?,
                env: value.environment_color_0.to_vec4()?,
                boost: value.environment_color_0_boost.to_vec4()?,
            },
            right: ColorSet {
                saber: value.saber_b_color.to_vec4()?,
                env: value.environment_color_1.to_vec4()?,
                boost: value.environment_color_1_boost.to_vec4()?,
            },
        })
    }
}

impl From<ColorScheme> for ColorSchemeV4 {
    fn from(value: ColorScheme) -> Self {
        Self {
            use_override: value.use_override,
            color_scheme_name: value.name,
            saber_a_color: value.left.saber.as_hex(),
            saber_b_color: value.right.saber.as_hex(),
            obstacles_color: value.obstacles.as_hex(),
            environment_color_0: value.left.env.as_hex(),
            environment_color_1: value.right.env.as_hex(),
            environment_color_0_boost: value.left.boost.as_hex(),
            environment_color_1_boost: value.right.boost.as_hex(),
        }
    }
}

impl From<MapAuthors> for MapAuthorsV4 {
    fn from(value: MapAuthors) -> Self {
        Self {
            mappers: value.mappers,
            lighters: value.lighters,
        }
    }
}



