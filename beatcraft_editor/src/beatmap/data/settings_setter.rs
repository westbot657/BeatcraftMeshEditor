use super::{BeatmapDataError, convert_u8};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomSettingsV2 {
    #[serde(rename = "_playerOptions")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_options: Option<PlayerOptionsV2>,
    #[serde(rename = "_modifiers")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modifiers: Option<ModifiersOptionsV2>,
    #[serde(rename = "_graphics")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graphics: Option<GraphicsOptionsV2>,
    #[serde(rename = "_chroma")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chroma: Option<ChromaOptionsV2>,
    #[serde(rename = "_countersPlus")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counters_plus: Option<CountersPlusOptionsV2>,
    #[serde(rename = "_uiTweaks")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_tweaks: Option<UiTweaksOptionsV2>,
    #[serde(rename = "_noteTweaks")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_tweaks: Option<NoteTweaksOptionsV2>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerOptionsV2 {
    #[serde(rename = "_leftHanded")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_handed: Option<bool>,
    #[serde(rename = "_playerHeight")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_height: Option<f32>,
    #[serde(rename = "_automaticPlayerHeight")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automatic_player_height: Option<bool>,
    #[serde(rename = "_sfxVolume")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sfx_volume: Option<f32>,
    #[serde(rename = "_reduceDebris")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reduce_debris: Option<bool>,
    #[serde(rename = "_noTextsAndHuds")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_texts_and_huds: Option<bool>,
    #[serde(rename = "_noFailEffects")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_fail_effects: Option<bool>,
    #[serde(rename = "_advancedHud")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advanced_hud: Option<bool>,
    #[serde(rename = "_autoRestart")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_restart: Option<bool>,
    #[serde(rename = "_saberTrailIntensity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saber_trail_intensity: Option<f32>,
    #[serde(flatten)]
    pub note_jump_duration_type_settings: Option<NoteJumpDurationTypeSettingsV2>,
    #[serde(rename = "_hideNoteSpawnEffect")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_note_spawn_effect: Option<bool>,
    #[serde(rename = "_adaptiveSfx")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adaptive_sfx: Option<bool>,
    #[serde(rename = "_environmentEffectsFilterDefaultPreset")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_effects_filter_default_preset: Option<EnvironmentEffectsV2>,
    #[serde(rename = "_environmentEffectsFilterExpertPlusPreset")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_effects_filter_expert_plus_preset: Option<EnvironmentEffectsV2>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum EnvironmentEffectsV2 {
    AllEffects,
    StrobeFilter,
    NoEffects,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "_noteJumpDurationTypeSettings")]
pub enum NoteJumpDurationTypeSettingsV2 {
    Dynamic {
        #[serde(rename = "_noteJumpStartBeatOffset")]
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note_jump_start_beat_offset: Option<f32>,
    },
    Static {
        #[serde(rename = "_noteJumpFixedDuration")]
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note_jump_fixed_duration: Option<f32>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModifiersOptionsV2 {
    #[serde(rename = "_energyType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_type: Option<EnergyTypeV2>,
    #[serde(rename = "_noFailOn0Energy")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_fail_on_0_energy: Option<bool>,
    #[serde(rename = "_instaFail")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insta_fail: Option<bool>,
    #[serde(rename = "_failOnSaberClash")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_on_saber_clash: Option<bool>,
    #[serde(rename = "_enablesObstacleType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled_obstacle_type: Option<ObstacleType>,
    #[serde(rename = "_fastNotes")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Forces NJS to 20.
    pub fast_notes: Option<bool>,
    #[serde(rename = "_strictAngles")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict_angles: Option<bool>,
    #[serde(rename = "_disappearingArrows")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disappearing_arrows: Option<bool>,
    #[serde(rename = "_ghostNotes")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ghost_notes: Option<bool>,
    #[serde(rename = "_noBombs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_bombs: Option<bool>,
    #[serde(rename = "_songSpeed")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub song_speed: Option<SongSpeed>,
    #[serde(rename = "_noArrows")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_arrows: Option<bool>,
    #[serde(rename = "_proMode")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pro_mode: Option<bool>,
    #[serde(rename = "_zenMode")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zen_mode: Option<bool>,
    #[serde(rename = "_smallCubes")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub small_cubes: Option<bool>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum EnergyTypeV2 {
    Bar,
    Battery,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ObstacleType {
    All,
    FullHeightOnly,
    NoObstacles,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum SongSpeed {
    Normal,
    Faster,
    Slow,
    SuperFast,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentOptionsV2 {
    #[serde(rename = "_overrideEnvironments")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_environments: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorOptionsV2 {
    #[serde(rename = "_overrideDefaultColors")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_default_colors: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphicsOptionsV2 {
    #[serde(rename = "_mirrorGraphicsSettings")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror_graphics_settings: Option<MirrorGraphicsSettings>,
    #[serde(rename = "_mainEffectGraphicsSettings")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// "Bloom Post Process". Disabling switches to baked/fake "Quest style" bloom.
    pub main_effect_graphics_settings: Option<MainEffectGraphicsSettings>,
    #[serde(rename = "_smokeGraphicsSettings")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Also enables depth texture / "Soft Particles" when used.
    pub smoke_graphics_settings: Option<SmokeGraphicsSettings>,
    #[serde(rename = "_burnMarkTrailsEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Hidden setting: hides burn trails left by sabers.
    pub burn_mark_trails_enabled: Option<bool>,
    #[serde(rename = "_screenDisplacementEffectsEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screen_displacement_effects_enabled: Option<bool>,
    #[serde(rename = "_maxShockwaveParticles")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_shockwave_particles: Option<MaxShockwaveParticles>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
#[repr(u8)]
pub enum MirrorGraphicsSettings {
    Off = 0,
    Low = 1,
    Medium = 2,
    High = 3,
}
convert_u8! { MirrorGraphicsSettings : 0..=3 }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
#[repr(u8)]
pub enum MainEffectGraphicsSettings {
    Off = 0,
    On = 1,
}
convert_u8! { MainEffectGraphicsSettings : 0 | 1 }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
#[repr(u8)]
pub enum SmokeGraphicsSettings {
    Off = 0,
    On = 1,
}
convert_u8! { SmokeGraphicsSettings : 0 | 1 }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
#[repr(u8)]
pub enum MaxShockwaveParticles {
    Off = 0,
    Low = 1,
    High = 2,
}
convert_u8! { MaxShockwaveParticles : 0..=2 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChromaOptionsV2 {
    #[serde(rename = "_disableChromaEvents")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_chroma_events: Option<bool>,
    #[serde(rename = "_disableEnvironmentEnhancements")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_environment_enhancements: Option<bool>,
    #[serde(rename = "_disableNoteColoring")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_note_coloring: Option<bool>,
    #[serde(rename = "_forceZenModeWalls")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_zen_mode_walls: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CountersPlusOptionsV2 {
    #[serde(rename = "_mainEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main_enabled: Option<bool>,
    #[serde(rename = "_mainParentedToBaseGameHUD")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main_parented_to_base_game_hud: Option<bool>,
    #[serde(rename = "_missedEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missed_enabled: Option<bool>,
    #[serde(rename = "_progressEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_enabled: Option<bool>,
    #[serde(rename = "_scoreEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_enabled: Option<bool>,
    #[serde(rename = "_personalBestEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub personal_best_enabled: Option<bool>,
    #[serde(rename = "_speedEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed_enabled: Option<bool>,
    #[serde(rename = "_cutEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cut_enabled: Option<bool>,
    #[serde(rename = "_spinometerEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spinometer_enabled: Option<bool>,
    #[serde(rename = "_notesLeftEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes_left_enabled: Option<bool>,
    #[serde(rename = "_failEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_enabled: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiTweaksOptionsV2 {
    #[serde(rename = "_multiplierEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multiplier_enabled: Option<bool>,
    #[serde(rename = "_energyEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_enabled: Option<bool>,
    #[serde(rename = "_comboEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combo_enabled: Option<bool>,
    #[serde(rename = "_positionEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_enabled: Option<bool>,
    #[serde(rename = "_progressEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_enabled: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoteTweaksOptionsV2 {
    #[serde(rename = "_enabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(rename = "_enableBombOutlines")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_bomb_outlines: Option<bool>,
    #[serde(rename = "_enableNoteOutlines")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_note_outlines: Option<bool>,
    #[serde(rename = "_enableAccDot")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_acc_dot: Option<bool>,
    #[serde(rename = "_enableDots")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_dots: Option<bool>,
    #[serde(rename = "_enableChainDots")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_chain_dots: Option<bool>,
    #[serde(rename = "_fixDotsIfNoodle")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix_dots_if_noodle: Option<bool>,
    #[serde(rename = "_enableFog")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_fog: Option<bool>,
    #[serde(rename = "_enableHeightFog")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_height_fog: Option<bool>,
    #[serde(rename = "_noteScaleX")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_scale_x: Option<f32>,
    #[serde(rename = "_noteScaleY")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_scale_y: Option<f32>,
    #[serde(rename = "_noteScaleZ")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_scale_z: Option<f32>,
    #[serde(rename = "_arrowScaleX")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrow_scale_x: Option<f32>,
    #[serde(rename = "_arrowScaleY")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrow_scale_y: Option<f32>,
    #[serde(rename = "_dotScaleX")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dot_scale_x: Option<f32>,
    #[serde(rename = "_dotScaleY")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dot_scale_y: Option<f32>,
    #[serde(rename = "_linkScale")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_scale: Option<f32>,
    #[serde(rename = "_bombScale")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bomb_scale: Option<f32>,
}
