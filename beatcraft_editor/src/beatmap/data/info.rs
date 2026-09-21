use bs_mapping_data::info_v4::{AudioInfoV4, InfoV4, SongInfoV4};
use bs_mapping_data::{InfoVersion, MapCharacteristic, MapDifficulty};
use glam::Vec4;


/// General info that can coerce into V2 and V4
pub struct Info {
    pub song: Option<SongInfo>,
    pub audio: Option<AudioInfo>,
    pub preview_filename: String,
    pub cover_filename: String,
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

        Ok(Self {
            version: InfoVersion::V4_0_1,
            song: SongInfoV4 {
                title: song.title,
                sub_title: song.subtitle,
                author: song.author,
            },
            audio: AudioInfoV4 {
                song_filename: audio.filename,
                song_duration: todo!(),
                audio_data_filename: todo!(),
                bpm: todo!(),
                lufs: todo!(),
                preview_start_time: todo!(),
                preview_duration: todo!(),
            },
            song_preview_filename: todo!(),
            cover_image_filename: todo!(),
            environment_names: todo!(),
            color_schemes: todo!(),
            difficulty_beatmaps: todo!(),
        })
    }
}




