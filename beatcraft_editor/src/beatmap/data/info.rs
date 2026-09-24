use bs_mapping_data::info_v4::{AudioInfoV4, ColorSchemeV4, DifficultyBeatmapV4, InfoV4, MapAuthorsV4, SongInfoV4};
use bs_mapping_data::{InfoVersion, MapCharacteristic, MapDifficulty};
use glam::Vec4;


/// General info that can coerce into V2 and V4
pub struct Info {
    pub song: Option<SongInfo>,
    pub audio: Option<AudioInfo>,
    pub preview_filename: Option<String>,
    pub cover_filename: Option<String>,
    pub color_schemes: Vec<ColorScheme>,
    pub beatmaps: Vec<DifficultyBeatmap>,
}

pub struct SongInfo {
    pub title: String,
    pub subtitle: String,
    pub author: String,
}

pub struct AudioInfo {
    pub filename: String,
    pub duration: f32,
    pub data_filename: Option<String>,
    pub bpm: f32,
    pub lufs: f32,
    pub preview_start: f32,
    pub preview_duration: f32,
}

pub struct ColorSet {
    pub saber: Vec4,
    pub env: Vec4,
    pub boost: Vec4,
}

pub struct ColorScheme {
    pub use_override: bool,
    pub name: String,
    pub obstacles: Vec4,
    pub left: ColorSet,
    pub right: ColorSet,
}

pub struct DifficultyBeatmap {
    pub characteristic: MapCharacteristic,
    pub difficulty: MapDifficulty,
    pub authors: MapAuthors,
    pub enviornment: String,
    pub color_scheme: Option<u32>,
    pub njs: f32,
    pub jump_offset: f32,
    pub filename: Option<String>,
    pub lightshow_filename: Option<String>,
}

pub struct MapAuthors {
    pub mappers: Vec<String>,
    pub lighters: Vec<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum InfoExportError {
    #[error("Missing element: {0}")]
    MissingComponent(String),
}

impl TryFrom<Info> for InfoV4 {
    type Error = InfoExportError;
    fn try_from(value: Info) -> Result<Self, Self::Error> {
        let Some(song) = value.song else {
            return Err(InfoExportError::MissingComponent("Song Info".to_string()));
        };

        let Some(audio) = value.audio else {
            return Err(InfoExportError::MissingComponent("Audio Info".to_string()));
        };

        let Some(audio_data_filename) = audio.data_filename else {
            return Err(InfoExportError::MissingComponent("Audio Data File".to_string()));
        };

        let song_preview_filename = value.preview_filename.unwrap_or_else(|| audio.filename.clone());

        let Some(cover_image_filename) = value.cover_filename else {
            return Err(InfoExportError::MissingComponent("Cover Image".to_string()));
        };

        let color_schemes: Vec<ColorSchemeV4> = value.color_schemes.into_iter().map(Into::into).collect();

        let mut environment_names = Vec::new();
        let mut difficulty_beatmaps = Vec::new();

        for diff in value.beatmaps {
            let environment_name_idx = match environment_names.iter().enumerate().find(|(_, e)| *e == &diff.enviornment) {
                Some((idx, _)) => idx,
                None => {
                    let l = environment_names.len();
                    environment_names.push(diff.enviornment.clone());
                    l
                }
            } as u32;

            let Some(beatmap_data_filename) = diff.filename else {
                return Err(InfoExportError::MissingComponent(format!("Beatmap File Name ({}, {})", diff.characteristic, diff.difficulty)));
            };

            let Some(lightshow_data_filename) = diff.lightshow_filename else {
                return Err(InfoExportError::MissingComponent(format!("Lightshow File Name ({}, {})", diff.characteristic, diff.difficulty)));
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
                song_duration: audio.duration,
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

trait Vec4ToHex {
    fn as_hex(&self) -> String;
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



