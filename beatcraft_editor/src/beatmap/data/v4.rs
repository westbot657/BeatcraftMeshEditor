use serde::{Deserialize, Serialize};

use super::{ArcDataV4, ArcV4, BombNoteDataV4, BombNoteV4, BpmRegion, ChainDataV4, ChainV4, ColorNoteDataV4, ColorNoteV4, InfoVersion, MapCharacteristic, MapDifficulty, MapVersion, NJSEventDataV4, NJSEventV4, ObstacleDataV4, ObstacleV4, SpawnRotationEventDataV4, SpawnRotationEventV4};



#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct BeatmapFileV4 {
    pub version: MapVersion,
    pub color_notes: Vec<ColorNoteV4>,
    pub color_notes_data: Vec<ColorNoteDataV4>,
    pub bomb_notes: Vec<BombNoteV4>,
    pub bomb_notes_data: Vec<BombNoteDataV4>,
    pub obstacles: Vec<ObstacleV4>,
    pub obstacles_data: Vec<ObstacleDataV4>,
    pub arcs: Vec<ArcV4>,
    pub arcs_data: Vec<ArcDataV4>,
    pub chains: Vec<ChainV4>,
    pub chains_data: Vec<ChainDataV4>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub njs_events: Option<Vec<NJSEventV4>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub njs_event_data: Option<Vec<NJSEventDataV4>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_rotations: Option<Vec<SpawnRotationEventV4>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_rotations_data: Option<Vec<SpawnRotationEventDataV4>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct InfoV4 {
    pub version: InfoVersion,
    pub song: SongInfoV4,
    pub audio: AudioInfoV4,
    pub song_preview_filename: String,
    pub cover_image_filename: String,
    pub environment_names: Vec<String>,
    pub color_schemes: Vec<ColorSchemeV4>,
    pub difficulty_beatmaps: Vec<DifficultyBeatmapV4>,

}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct SongInfoV4 {
    pub title: String,
    pub sub_title: String,
    pub author: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AudioInfoV4 {
    pub song_filename: String,
    pub song_duration: f32,
    pub audio_data_filename: String,
    pub bpm: f32,
    pub lufs: f32,
    pub preview_start_time: f32,
    pub preview_duration: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct ColorSchemeV4 {
    pub use_override: bool,
    pub color_scheme_name: String,
    pub saber_a_color: String,
    pub saber_b_color: String,
    pub obstacles_color: String,
    pub environment_color_0: String,
    pub environment_color_1: String,
    pub environment_color_0_boost: String,
    pub environment_color_1_boost: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct DifficultyBeatmapV4 {
    pub characteristic: MapCharacteristic,
    pub difficulty: MapDifficulty,
    pub beatmap_authors: MapAuthorsV4,
    pub environment_name_idx: u32,
    pub beatmap_color_scheme_idx: i32,
    pub note_jump_movement_speed: f32,
    pub note_jump_start_beat_offset: f32,
    pub beatmap_data_filename: String,
    pub lightshow_data_filename: String,

}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct MapAuthorsV4 {
    pub mappers: Vec<String>,
    pub lighters: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AudioDataFileV4 {
    pub version: MapVersion,
    pub song_checksum: String,
    pub song_sample_count: usize,
    pub song_frequency: u32,
    pub bpm_data: Vec<BpmRegionV4>,

    // I swear these things are buggy in base game anyways so idc about them rn.
    lufs_data: Vec<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BpmRegionV4 {
    #[serde(rename = "si")]
    pub start_index: usize,
    #[serde(rename = "ei")]
    pub end_index: usize,
    #[serde(rename = "sb")]
    pub start_beat: f32,
    #[serde(rename = "eb")]
    pub end_beat: f32,
}

impl From<&BpmRegionV4> for BpmRegion {
    fn from(value: &BpmRegionV4) -> Self {
        Self {
            start_sample: value.start_index,
            end_sample: value.end_index,
            start_beat: value.start_beat,
            end_beat: value.end_beat,
        }
    }
}

