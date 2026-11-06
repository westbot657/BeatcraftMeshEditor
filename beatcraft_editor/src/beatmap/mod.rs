use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use eframe::glow::{self, Context, HasContext};
use egui::{ImageSource, Ui};
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use indexmap::IndexMap;
use rand::rngs::ThreadRng;
use beatcraft_audiolib::{Audio, AudioError, AudioSystem, NativeOpenAL};
use beatcraft_editor_proc::hex;
use crate::config::ProjectType;
use crate::data::map_editing::{EditingData, GlobalEditingData, ObjectSource};
use crate::data::mesh::LightMeshData;
use crate::editor::{
    App, EditorContext, MINECRAFT_F, RoutineAction, SOURCE_CODE_F, Selection, ViewStyle,
    setup_fonts,
};
use crate::light_mesh::LightMesh;
use crate::render::{GpuMesh, GridType, InstanceData, MeshDrawCall, Renderer};
use crate::widgets::{ImageCard, MathDragValue};
use crate::{
    DB_AUDIO, DB_DATA, DB_LOGIC, DB_MAIN, MISSING_EDITOR_ICON, RefDuper, UnsafeMutRef, editor,
    get_data_folder,
};

use self::object::GameObjectExt;
use self::data::obstacle_font::{HorizontalAlign, ObstacleFont, ObstacleFontData, TextLayout, VerticalAlign};
use bs_mapping_data::custom_info_v2::DifficultyBeatmapCustomDataV2;
use bs_mapping_data::info_v2::{CharacteristicSetV2, DifficultyBeatmapV2};
use bs_mapping_data::{ArcMidAnchorMode, AudioDataFile, BeatmapFile, Color, CutDirection, InfoFile, MapCharacteristic, MapDifficulty};

use beatmap_core::{ArrowType, BeatmapController, GameObject, HitBox, ObjectType, RuntimeData};
use crate::beatmap::widgets::cut_direction_selector::{CutDirectionSelector, SelectorLayout};
use crate::beatmap::widgets::keybind_hint::{Corner, ExtensionAxis, ExtensionCorner, KeybindHint};

pub mod data;
pub mod event;
pub mod object;
pub mod render;
pub mod widgets;
#[cfg(test)]
pub mod tests;

pub static COLOR_NOTE_TOOL_ICON: ImageSource =
    egui::include_image!("../assets/textures/svg/color_note.svg");
pub static BOMB_NOTE_TOOL_ICON: ImageSource =
    egui::include_image!("../assets/textures/svg/bomb.svg");
pub static CHAIN_NOTE_TOOL_ICON: ImageSource =
    egui::include_image!("../assets/textures/svg/chain_note.svg");

pub static OBSTACLE_TOOL_ICON: ImageSource =
    egui::include_image!("../assets/textures/svg/obstacle.svg");
pub static ARC_TOOL_ICON: ImageSource =
    egui::include_image!("../assets/textures/svg/arc.svg");
pub static OBSTACLE_TEXT_TOOL_ICON: ImageSource =
    egui::include_image!("../assets/textures/svg/obstacle_text.svg");

pub static EDIT_ICON: ImageSource =
    egui::include_image!("../assets/textures/svg/edit_icon.svg");
pub static OPEN_MAP_ICON: ImageSource =
    egui::include_image!("../assets/textures/svg/open_map_icon.svg");
pub static CREATE_MAP_ICON: ImageSource =
    egui::include_image!("../assets/textures/svg/create_map_icon.svg");

pub static COLOR_NOTE_BRIGHT: ImageSource =
    egui::include_image!("../assets/textures/svg/color_note_bright.svg");
pub static COLOR_NOTE_DOT_BRIGHT: ImageSource =
    egui::include_image!("../assets/textures/svg/color_note_dot_bright.svg");

pub struct V4BeatmapProjectDiffData {
    pub mappers: Vec<String>,
    pub lighters: Vec<String>,
}

pub struct BeatmapProjectDiff {
    pub difficulty: MapDifficulty,
    pub beatmap_file: Option<PathBuf>,
    pub njs: f32,
    pub njs_offset: f32,
    pub custom_data: Option<DifficultyBeatmapCustomDataV2>,
    pub v4_data: Option<V4BeatmapProjectDiffData>,
}

pub struct BeatmapProjectSet {
    pub set: MapCharacteristic,
    pub diffs: Vec<BeatmapProjectDiff>,
}

impl From<&CharacteristicSetV2> for BeatmapProjectSet {
    fn from(value: &CharacteristicSetV2) -> Self {
        Self {
            set: value.beatmap_characteristic_name.clone(),
            diffs: value.difficulty_beatmaps.iter().map(Into::into).collect(),
        }
    }
}

impl From<&DifficultyBeatmapV2> for BeatmapProjectDiff {
    fn from(value: &DifficultyBeatmapV2) -> Self {
        let path = &value.beatmap_filename;
        tracing::debug!(target: DB_DATA, ?path, "Loaded Beatmap difficulty: {}", value.difficulty);
        Self {
            difficulty: value.difficulty.clone(),
            beatmap_file: Some(PathBuf::from(&value.beatmap_filename)),
            njs: value.note_jump_movement_speed,
            njs_offset: value.note_jump_start_beat_offset,
            custom_data: value.custom_data.clone(),
            v4_data: None,
        }
    }
}

pub struct BeatmapProject {
    pub folder: PathBuf,
    pub info_path: Option<PathBuf>,
    pub info: Option<InfoFile>,
    pub audio_info_path: Option<PathBuf>,
    pub audio_info: Option<AudioDataFile>,
    pub audio: Option<std::sync::Arc<Audio<NativeOpenAL>>>,
    pub cover_image: Option<PathBuf>,
    pub sets: Vec<BeatmapProjectSet>,
    pub controller: Option<(BeatmapController<ObjectSource>, EditingData)>,

    pub editor_data_path: Option<PathBuf>,
    pub editor_data: GlobalEditingData,

    /// Option<(sets index, diff index)>
    pub selected_diff: Option<(usize, usize)>,
}

pub struct BeatmapMeshSet {
    pub note_mesh: GpuMesh,
    pub bomb_mesh: GpuMesh,
    pub chain_head_mesh: GpuMesh,
    pub chain_body_mesh: GpuMesh,
    pub arrow_mesh: GpuMesh,
    pub dot_mesh: GpuMesh,
    pub chain_dot_mesh: GpuMesh,
    obstacle_mesh: GpuMesh,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlacementObjectType {
    ColorNote,
    BombNote,
    Obstacle,
    Chain,
    Arc,
    Text,
}

impl PlacementObjectType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ColorNote => "Color Note",
            Self::BombNote => "Bomb Note",
            Self::Obstacle => "Obstacle",
            Self::Chain => "Chain Note",
            Self::Arc => "Arc",
            Self::Text => "Obstacle Text",
        }
    }

    pub fn tool_icon(&self) -> ImageSource<'_> {
        match self {
            Self::ColorNote => COLOR_NOTE_TOOL_ICON.clone(),
            Self::BombNote => BOMB_NOTE_TOOL_ICON.clone(),
            Self::Obstacle => OBSTACLE_TOOL_ICON.clone(),
            Self::Chain => CHAIN_NOTE_TOOL_ICON.clone(),
            Self::Arc => ARC_TOOL_ICON.clone(),
            Self::Text => OBSTACLE_TEXT_TOOL_ICON.clone(),
        }
    }

    pub fn iter_all() -> impl Iterator<Item = Self> {
        [
            Self::ColorNote,
            Self::BombNote,
            Self::Obstacle,
            Self::Chain,
            Self::Arc,
            Self::Text,
        ].into_iter()
    }

    pub fn key_name(&self) -> &'static str {
        match self {
            Self::ColorNote => "tool-note",
            Self::BombNote => "tool-bomb",
            Self::Obstacle => "tool-obstacle",
            Self::Chain => "tool-chain",
            Self::Arc => "tool-arc",
            Self::Text => "tool-text",
        }
    }

}

impl Display for PlacementObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[derive(Default, Clone, PartialEq, Eq)]
pub enum ColorMode {
    #[default]
    Default,
    Color,
    Gradient,
}

pub struct ColorNotePlacementData {
    pub note_type: Color,
    pub cut_direction: CutDirection,
    pub cut_offset: f32,
    pub use_cut_offset: bool,

    pub left_color_mode: ColorMode,
    pub left_custom_color: Vec4,
    pub left_gradient: String,

    pub right_color_mode: ColorMode,
    pub right_custom_color: Vec4,
    pub right_gradient: String,
}

impl Default for ColorNotePlacementData {
    fn default() -> Self {
        Self {
            note_type: Default::default(),
            cut_direction: CutDirection::Down,
            cut_offset: 0.0,
            use_cut_offset: false,

            left_color_mode: Default::default(),
            left_custom_color: hex!(#BF2F2F),
            left_gradient: String::new(),

            right_color_mode: Default::default(),
            right_custom_color: hex!(#1F63A7),
            right_gradient: String::new(),
        }
    }
}

impl ColorNotePlacementData {
    fn note_colors(&mut self) -> NoteColorRef<'_> {
        NoteColorRef::new(
            &mut self.note_type,
            &mut self.left_color_mode,
            &mut self.left_custom_color,
            &mut self.left_gradient,
            &mut self.right_color_mode,
            &mut self.right_custom_color,
            &mut self.right_gradient,
        )
    }
}

pub struct BombPlacementData {
    pub color_mode: ColorMode,
    pub custom_color: Vec4,
    pub gradient: String,
}

impl Default for BombPlacementData {
    fn default() -> Self {
        Self {
            color_mode: Default::default(),
            custom_color: hex!(#333333),
            gradient: String::new(),
        }
    }
}

impl BombPlacementData {
    fn object_color(&mut self) -> ObjectColorRef<'_> {
        ObjectColorRef::new(&mut self.color_mode, &mut self.custom_color, &mut self.gradient)
    }
}

pub struct ObstaclePlacementData {
    pub width: f32,
    pub height: f32,
    pub duration: f32,

    pub length: f32,
    pub use_length: bool,

    pub color_mode: ColorMode,
    pub custom_color: Vec4,
    pub gradient: String,
}

impl Default for ObstaclePlacementData {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 5.0,
            duration: 1.0,

            length: 1.0,
            use_length: false,

            color_mode: Default::default(),
            custom_color: hex!(#FF2F2F),
            gradient: String::new(),
        }
    }
}

impl ObstaclePlacementData {
    fn object_color(&mut self) -> ObjectColorRef<'_> {
        ObjectColorRef::new(&mut self.color_mode, &mut self.custom_color, &mut self.gradient)
    }
}

pub struct ChainPlacementData {
    pub note_type: Color,
    pub cut_direction: CutDirection,
    pub link_count: u8,

    pub left_color_mode: ColorMode,
    pub left_custom_color: Vec4,
    pub left_gradient: String,

    pub right_color_mode: ColorMode,
    pub right_custom_color: Vec4,
    pub right_gradient: String,
}

impl Default for ChainPlacementData {
    fn default() -> Self {
        Self {
            note_type: Default::default(),
            cut_direction: CutDirection::Down,
            link_count: 0,

            left_color_mode: Default::default(),
            left_custom_color: hex!(#BF2F2F),
            left_gradient: String::new(),

            right_color_mode: Default::default(),
            right_custom_color: hex!(#1F63A7),
            right_gradient: String::new(),
        }
    }
}

impl ChainPlacementData {
    fn note_colors(&mut self) -> NoteColorRef<'_> {
        NoteColorRef::new(
            &mut self.note_type,
            &mut self.left_color_mode,
            &mut self.left_custom_color,
            &mut self.left_gradient,
            &mut self.right_color_mode,
            &mut self.right_custom_color,
            &mut self.right_gradient,
        )
    }
}

pub struct ArcPlacementData {
    pub note_type: Color,

    pub head_cut_direction: CutDirection,

    pub mid_anchor_mode: ArcMidAnchorMode,

    pub tail_cut_direction: CutDirection,

    pub left_color_mode: ColorMode,
    pub left_custom_color: Vec4,
    pub left_gradient: String,

    pub right_color_mode: ColorMode,
    pub right_custom_color: Vec4,
    pub right_gradient: String,
}

impl Default for ArcPlacementData {
    fn default() -> Self {
        Self {
            note_type: Default::default(),

            head_cut_direction: CutDirection::Down,

            mid_anchor_mode: Default::default(),

            tail_cut_direction: CutDirection::Down,

            left_color_mode: Default::default(),
            left_custom_color: hex!(#BF2F2F),
            left_gradient: String::new(),

            right_color_mode: Default::default(),
            right_custom_color: hex!(#1F63A7),
            right_gradient: String::new(),
        }
    }
}

impl ArcPlacementData {
    fn note_colors(&mut self) -> NoteColorRef<'_> {
        NoteColorRef::new(
            &mut self.note_type,
            &mut self.left_color_mode,
            &mut self.left_custom_color,
            &mut self.left_gradient,
            &mut self.right_color_mode,
            &mut self.right_custom_color,
            &mut self.right_gradient,
        )
    }
}

pub struct TextPlacementData {
    pub color_mode: ColorMode,
    pub custom_color: Vec4,
    pub gradient: String,

    pub text: String,
    pub font: ObstacleFontData,

    pub kerning: f32,
    pub word_spacing: f32,
    pub line_spacing: f32,

    pub offset: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,

    pub duration: f32,
    pub use_duration: bool,
}

impl Default for TextPlacementData {
    fn default() -> Self {
        Self {
            color_mode: Default::default(),
            custom_color: hex!(#FF2F2F),
            gradient: String::new(),

            text: Default::default(),
            font: ObstacleFontData {
                font: ObstacleFont::Simple,
                layout: TextLayout::Centered,
                horizontal_align: HorizontalAlign::Center,
                vertical_align: VerticalAlign::Center,
                letter_spacing: 0.8,
                word_spacing: 0.8,
            },

            kerning: 0.8,
            word_spacing: 0.8,
            line_spacing: 0.8,

            offset: Vec3::ZERO,
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,

            duration: 1.,
            use_duration: true,
        }
    }
}

#[derive(Default)]
pub struct PlacementData {
    pub color_note: ColorNotePlacementData,
    pub bomb: BombPlacementData,
    pub obstacle: ObstaclePlacementData,
    pub chain: ChainPlacementData,
    pub arc: ArcPlacementData,
    pub text: TextPlacementData,
}

pub struct BeatmapEditor {
    pub map: Option<BeatmapProject>,
    pub mesh_set: BeatmapMeshSet,
    pub scroll_step: f32,
    pub grid_snap: bool,
    pub editor_data: GlobalEditingData,
    pub selected_placement_type: PlacementObjectType,
    pub placement_data: PlacementData,
}

#[derive(thiserror::Error, Debug)]
pub enum MapLoadError {
    #[error("Given path is not a directory, or can't be read: {0}")]
    FileNotADirectory(String),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("JSON Parse Error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Audio Error: {0}")]
    AudioError(#[from] AudioError),
}

static DEFAULT_NOTE: &[u8] = include_bytes!("../assets/meshes/color_note.json");
static DEFAULT_ARROW: &[u8] = include_bytes!("../assets/meshes/arrow.json");
static DEFAULT_DOT: &[u8] = include_bytes!("../assets/meshes/color_note_dot.json");
static DEFAULT_CHAIN_HEAD: &[u8] = include_bytes!("../assets/meshes/chain_note_head.json");
static DEFAULT_CHAIN_LINK: &[u8] = include_bytes!("../assets/meshes/chain_note_link.json");
static DEFAULT_CHAIN_DOT: &[u8] = include_bytes!("../assets/meshes/chain_note_link_dot.json");
static OBSTACLE: &[u8] = include_bytes!("../assets/meshes/obstacle.json");

static NOTE_TEXTURE: &[u8] = include_bytes!("../assets/textures/color_note.png");
static ARROW_TEXTURE: &[u8] = include_bytes!("../assets/textures/arrow.png");

impl BeatmapMeshSet {
    pub fn new(gl: &Context, renderer: &mut Renderer) -> Result<Self, MapLoadError> {
        let dir = get_data_folder().unwrap();

        let _ = std::fs::create_dir_all(&dir);

        let note_tex = dir.join("color_note.png");
        let arrow_tex = dir.join("arrow.png");

        std::fs::write(&note_tex, NOTE_TEXTURE)?;
        std::fs::write(&arrow_tex, ARROW_TEXTURE)?;

        renderer
            .texture_paths
            .insert("builtin:color_note".to_string(), note_tex);
        renderer
            .texture_paths
            .insert("builtin:arrow".to_string(), arrow_tex);

        renderer.rebuild_atlases(gl);

        macro_rules! setup_mesh {
            ($data:ident, $name:literal) => {
                {
                    tracing::debug!(target: DB_MAIN, "Loading default mesh for {}", $name);
                    let lm: LightMeshData = serde_json::from_slice($data)?;
                    let lm: LightMesh = lm.into();
                    let mut mesh = GpuMesh::empty(gl);
                    mesh.set_from_full_light_mesh(gl, &lm, &renderer.texture_paths, &renderer.atlas_map);
                    mesh
                }
            };
        }

        let note_mesh = setup_mesh!(DEFAULT_NOTE, "Color Note");
        let bomb_mesh = setup_mesh!(DEFAULT_NOTE, "Bomb Note");
        let chain_head_mesh = setup_mesh!(DEFAULT_CHAIN_HEAD, "Chain Note Head");
        let chain_body_mesh = setup_mesh!(DEFAULT_CHAIN_LINK, "Chain Note Link");
        let arrow_mesh = setup_mesh!(DEFAULT_ARROW, "Arrow");
        let dot_mesh = setup_mesh!(DEFAULT_DOT, "Dot");
        let chain_dot_mesh = setup_mesh!(DEFAULT_CHAIN_DOT, "Chain Dot");
        let obstacle_mesh = setup_mesh!(OBSTACLE, "Obstacle");

        Ok(Self {
            note_mesh,
            bomb_mesh,
            chain_head_mesh,
            chain_body_mesh,
            arrow_mesh,
            dot_mesh,
            chain_dot_mesh,
            obstacle_mesh,
        })
    }
}

impl BeatmapEditor {
    pub fn new(
        audio_system: &mut AudioSystem<NativeOpenAL>,
        map: Option<PathBuf>,
        gl: &Context,
        renderer: &mut Renderer,
        volume: f32,
    ) -> Result<Self, MapLoadError> {
        let mut s = Self {
            map: None,
            mesh_set: BeatmapMeshSet::new(gl, renderer)?,
            scroll_step: 0.125,
            grid_snap: true,
            editor_data: Default::default(),
            selected_placement_type: PlacementObjectType::ColorNote,
            placement_data: Default::default(),
        };

        if let Some(map) = map {
            s.load(audio_system, map, gl, renderer, volume)?
        }

        Ok(s)
    }

    pub fn load(
        &mut self,
        s: &mut AudioSystem<NativeOpenAL>,
        map: PathBuf,
        _gl: &Context,
        _renderer: &mut Renderer,
        volume: f32,
    ) -> Result<(), MapLoadError> {
        let span = tracing::debug_span!("load beatmap");
        let _guard = span.enter();

        tracing::debug!(target: DB_DATA, "Loading beatmap");

        if !map.is_dir() {
            return Err(MapLoadError::FileNotADirectory(
                map.to_string_lossy().to_string(),
            ));
        }

        let mut info_file = None;
        let mut audio_info_path = None;
        for file in map.read_dir()? {
            let file = file?;
            let name = file.file_name().to_string_lossy().to_lowercase();

            if name == "info.dat" {
                info_file = Some(file.path());
            }
            if name == "bpminfo.dat" {
                audio_info_path = Some(file.path());
            }
        }

        let mut sets = Vec::new();
        let mut cover_image = None;
        let mut audio_info = None;
        let mut info_data = None;
        let mut audio_path = None;

        if let Some(path) = info_file.as_deref() {
            let data = std::fs::read(path)?;
            let info: InfoFile = serde_json::from_slice(&data)?;

            match &info {
                InfoFile::V2(v2) => {
                    sets = v2.difficulty_beatmap_sets.iter().map(Into::into).collect();
                    cover_image = Some(PathBuf::from(&v2.cover_image_filename));
                    audio_path = Some(PathBuf::from(&v2.song_filename));
                }
                InfoFile::V4(v4) => {
                    let mut sts = IndexMap::new();
                    for diff in v4.difficulty_beatmaps.iter() {
                        let entry = sts.entry(&diff.characteristic).or_insert(Vec::new());
                        entry.push(BeatmapProjectDiff {
                            difficulty: diff.difficulty.clone(),
                            beatmap_file: Some(PathBuf::from(&diff.beatmap_data_filename)),
                            njs: diff.note_jump_movement_speed,
                            njs_offset: diff.note_jump_start_beat_offset,
                            custom_data: None,
                            v4_data: Some(V4BeatmapProjectDiffData {
                                mappers: diff.beatmap_authors.mappers.clone(),
                                lighters: diff.beatmap_authors.lighters.clone(),
                            }),
                        });
                    }
                    for (ch, diffs) in sts.into_iter() {
                        sets.push(BeatmapProjectSet {
                            set: ch.clone(),
                            diffs,
                        });
                    }
                    audio_info_path = Some(PathBuf::from(&v4.audio.audio_data_filename));
                    cover_image = Some(PathBuf::from(&v4.cover_image_filename));
                    audio_path = Some(PathBuf::from(&v4.audio.song_filename));
                }
            }

            info_data = Some(info);
        }

        if let Some(path) = audio_info_path.as_deref() {
            let path = map.join(path);
            let data = std::fs::read(path)?;
            let info: AudioDataFile = serde_json::from_slice(&data)?;
            audio_info = Some(info);
        }

        let mut audio = None;
        if let Some(path) = audio_path {
            let ad = Audio::new(s, &map.join(path))?;
            ad.set_volume(volume);
            audio = Some(ad);
        }

        let project = BeatmapProject {
            folder: map,
            info_path: info_file,
            info: info_data,
            audio_info_path,
            audio_info,
            audio,
            cover_image,
            sets,
            controller: None,
            editor_data_path: None,
            editor_data: Default::default(),
            selected_diff: None,
        };

        self.map = Some(project);

        Ok(())
    }
}

struct NoteColorRef<'c> {
    note_type: &'c mut Color,

    left_mode: &'c mut ColorMode,
    left_color: &'c mut Vec4,
    left_gradient: &'c mut String,

    right_mode: &'c mut ColorMode,
    right_color: &'c mut Vec4,
    right_gradient: &'c mut String,
}

impl<'c> NoteColorRef<'c> {
    fn new(
        note_type: &'c mut Color,

        left_mode: &'c mut ColorMode,
        left_color: &'c mut Vec4,
        left_gradient: &'c mut String,

        right_mode: &'c mut ColorMode,
        right_color: &'c mut Vec4,
        right_gradient: &'c mut String,
    ) -> Self {
        Self {
            note_type,
            left_mode,
            left_color,
            left_gradient,
            right_mode,
            right_color,
            right_gradient,
        }
    }

    fn left(&mut self) -> ObjectColorRef<'_> {
        ObjectColorRef::new(self.left_mode, self.left_color, self.left_gradient)
    }

    fn right(&mut self) -> ObjectColorRef<'_> {
        ObjectColorRef::new(self.right_mode, self.right_color, self.right_gradient)
    }
}

struct ObjectColorRef<'c> {
    mode: &'c mut ColorMode,
    color: &'c mut Vec4,
    gradient: &'c mut String,
}

impl<'c> ObjectColorRef<'c> {
    fn new(
        mode: &'c mut ColorMode,
        color: &'c mut Vec4,
        gradient: &'c mut String,
    ) -> Self {
        Self {
            mode,
            color,
            gradient,
        }
    }
}

fn note_color_selector<'c>(
    app: &mut App,
    ui: &mut Ui,
    width: f32,
    w2: f32,
    w3: f32,
    mut data: NoteColorRef<'c>,
    enable_gradient: bool,
) {
    let left_str = app.data.locale.get("button-beatmap_editor-place-left").to_string();
    let right_str = app.data.locale.get("button-beatmap_editor-place-right").to_string();

    let mut hovered = false;
    let mut rects = Vec::new();

    let row = ui.horizontal(|ui| {
        let r = ui.add_sized(
            [w2, 20.],
            egui::Button::new(left_str).selected(*data.note_type == Color::Red),
        );
        hovered |= r.hovered();
        rects.push(("set-note-left", r.rect));
        if r.clicked() {
            *data.note_type = Color::Red;
        }

        let r = ui.add_sized(
            [w2, 20.],
            egui::Button::new(right_str).selected(*data.note_type == Color::Blue),
        );
        hovered |= r.hovered();
        rects.push(("set-note-right", r.rect));
        if r.clicked() {
            *data.note_type = Color::Blue;
        }
    });
    hovered |= row.response.hovered();

    for (name, rect) in rects {
        KeybindHint::new(name, rect, &row.response)
            .placement(ExtensionCorner::Exterior(Corner::TopRight, ExtensionAxis::Vertical))
            .gap(-10.)
            .scale(0.75)
            .show(ui, &app.data.keymaps);
    }

    if hovered {
        app.check_note_type_bind(data.note_type)
    }

    let obj = match *data.note_type {
        Color::Red => data.left(),
        Color::Blue => data.right(),
    };

    object_color_selector(app, ui, width, w3, obj, enable_gradient);
}

fn object_color_selector<'c>(
    app: &mut App,
    ui: &mut Ui,
    width: f32,
    w3: f32,
    obj: ObjectColorRef<'c>,
    enable_gradient: bool,
) {
    let color_str = app.data.locale.get("label-beatmap_editor-place-color").to_string();

    let default_color_str = app.data.locale.get("button-beatmap_editor-place-default-color").to_string();
    let rgba_str = app.data.locale.get("button-beatmap_editor-place-custom-color").to_string();
    let gradient_str = app.data.locale.get("button-beatmap_editor-place-gradient-color").to_string();
    let gradient_tooltip = app.data.locale.get("tooltip-beatmap_editor-place-gradient-color-disabled").to_string();

    let mut hovered = false;
    let mut rects = Vec::new();

    ui.label(color_str);

    let row = ui.horizontal(|ui| {
        let r = ui.add_sized(
            [w3, 20.],
            egui::Button::new(default_color_str).selected(*obj.mode == ColorMode::Default),
        );
        hovered |= r.hovered();
        rects.push(("set-color-default", r.rect));
        if r.clicked() {
            *obj.mode = ColorMode::Default;
        }

        let r = ui.add_sized(
            [w3, 20.],
            egui::Button::new(rgba_str).selected(*obj.mode == ColorMode::Color),
        );
        hovered |= r.hovered();
        rects.push(("set-color-rgba", r.rect));
        if r.clicked() {
            *obj.mode = ColorMode::Color;
        }

        let g = ui.add_enabled_ui(false, |ui| {
            let r = ui
                .add_sized(
                    [w3, 20.],
                    egui::Button::new(gradient_str).selected(*obj.mode == ColorMode::Gradient),
                )
                .on_disabled_hover_text(gradient_tooltip);
            if r.clicked() {
                *obj.mode = ColorMode::Gradient;
            }
            r.rect
        });
        hovered |= g.response.hovered();
        if enable_gradient {
            rects.push(("set-color-gradient", g.inner));
        }
    });
    hovered |= row.response.hovered();

    for (name, rect) in rects {
        KeybindHint::new(name, rect, &row.response)
            .placement(ExtensionCorner::Exterior(Corner::TopRight, ExtensionAxis::Vertical))
            .gap(-10.)
            .scale(0.75)
            .show(ui, &app.data.keymaps);
    }

    if hovered {
        app.check_object_color_bind(obj.mode, enable_gradient);
    }

    match *obj.mode {
        ColorMode::Default => {
            let padding = ui.spacing().item_spacing.y;
            ui.add_space(20. + padding);
        }
        ColorMode::Color => {
            let rgba = obj.color;
            let mut col = egui::Rgba::from_rgba_premultiplied(rgba.x, rgba.y, rgba.z, rgba.w);
            let spacing = ui.spacing().interact_size;
            ui.spacing_mut().interact_size = egui::Vec2::new(width, 20.);
            egui::color_picker::color_edit_button_rgba(ui, &mut col, egui::color_picker::Alpha::OnlyBlend);
            ui.spacing_mut().interact_size = spacing;
            *rgba = Vec4::new(col.r(), col.g(), col.b(), col.a())
        }
        ColorMode::Gradient => {
            // TODO: dropdown selector
        }
    }
}

fn format_time(seconds: f32) -> String {
    let rem = seconds % 60.;
    let minutes = seconds as u32 / 60;
    let hours = minutes / 60;
    let minutes = minutes % 60;
    format!("{hours:>02}:{minutes:>02}:{rem:06.3}")
}

impl App {
    fn await_beatmap_open(&mut self) {
        tracing::debug!(target: DB_LOGIC, "Spawning thread for opening beatmap project");
        let (sx, rx) = mpsc::channel();
        let title = self.data.locale.get("open-beatmap-title").to_string();
        thread::spawn(move || {
            let Some(map_folder) = rfd::FileDialog::new().set_title(title).pick_folder() else {
                tracing::debug!(target: DB_LOGIC, "Canceled opening beatmap");
                return;
            };
            tracing::debug!(target: DB_LOGIC, ?map_folder, "Opening beatmap");
            let _ = sx.send(map_folder);
        });
        self.add_routine(Box::new(move |s, gl| match rx.try_recv() {
            Err(mpsc::TryRecvError::Empty) => RoutineAction::None,
            Err(mpsc::TryRecvError::Disconnected) => RoutineAction::Remove,
            Ok(folder) => {
                let rd = RefDuper;
                let s2 = unsafe { rd.detach_mut_ref(s) };
                if let Some(map) = s.map_editor.map.take()
                    && let Some(audio) = map.audio
                {
                    audio.stop();
                    drop(audio);
                    s.audio_system.remove_dead_audio();
                }
                s.render.renderer.beatmap.seek(0.);
                s.history.clear();
                if let Err(e) = s.load_beatmap(
                    &mut s2.audio_system,
                    folder,
                    gl,
                    &mut s2.render.renderer,
                    s2.data.audio_volume,
                ) {
                    let st = s.data.locale.get("failed-to-load-beatmap").to_string();
                    s.set_status(None, st, 2.);
                    tracing::error!(target: DB_MAIN, "Failed to load beatmap: {e}");
                }
                RoutineAction::Remove
            }
        }));
    }

    fn check_tool_keybinds(&mut self, tool: &mut PlacementObjectType) {
        let _scope = self.data.keymaps.scope_ctx(vec!["hover-tool-panel"]);
        for t in PlacementObjectType::iter_all() {
            if self.data.keymaps.pressed(t.key_name()) {
                *tool = t;
            }
        }
    }

    fn check_cut_dir_bind(&mut self, cut_dir: &mut CutDirection) {
        let _scope = self.data.keymaps.scope_ctx(vec!["hover-cut-dir"]);
        const DIRS: [(&'static str, CutDirection); 9] = [
            ("cut-dir-ul", CutDirection::UpLeft),
            ("cut-dir-u", CutDirection::Up),
            ("cut-dir-ur", CutDirection::UpRight),
            ("cut-dir-l", CutDirection::Left),
            ("cut-dir-dot", CutDirection::Dot),
            ("cut-dir-r", CutDirection::Right),
            ("cut-dir-dl", CutDirection::DownLeft),
            ("cut-dir-d", CutDirection::Down),
            ("cut-dir-dr", CutDirection::DownRight),
        ];
        for (key, dir) in DIRS {
            if self.data.keymaps.pressed(key) {
                *cut_dir = dir;
            }
        }
    }

    fn check_note_type_bind(&mut self, color: &mut Color) {
        let _scope = self.data.keymaps.scope_ctx(vec!["hover-note-type"]);
        if self.data.keymaps.pressed("set-note-left") {
            *color = Color::Red;
        }
        if self.data.keymaps.pressed("set-note-right") {
            *color = Color::Blue;
        }
    }

    fn check_object_color_bind(&mut self, mode: &mut ColorMode, enable_gradient: bool) {
        let _scope = self.data.keymaps.scope_ctx(vec!["hover-note-color"]);
        if self.data.keymaps.pressed("set-color-default") {
            *mode = ColorMode::Default;
        }
        if self.data.keymaps.pressed("set-color-rgba") {
            *mode = ColorMode::Color;
        }
        if enable_gradient && self.data.keymaps.pressed("set-color-gradient") {
            *mode = ColorMode::Gradient;
        }
    }

    pub fn draw_beatmap_editor(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
        _shift: bool,
        _ctrl: bool,
        alt: bool,
    ) {
        let gl = frame.gl().unwrap();

        egui::TopBottomPanel::top("menu_bar_beatmap_editor").show(ctx, |ui| {
            ui.add_space(2.);
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button(self.data.locale.get("file-menu-label").to_string(), |ui| {
                    let mut options = [
                        (self.data.locale.get("open-beatmap").to_string(), None),
                        (self.data.locale.get("menu-label").to_string(), None),
                    ];
                    Self::pad_menu_text(&mut options);
                    let [(open_map, _), (menu, _)] = options;
                    if ui.button(open_map).clicked() {
                        self.await_beatmap_open();
                    }
                    if ui.button(menu).clicked() {
                        tracing::debug!(target: DB_LOGIC, "Returning to menu");
                        self.context = EditorContext::None;
                        self.render.renderer.beatmap.seek(0.);
                        self.history.clear();
                        if let Some(map) = self.map_editor.map.take()
                            && let Some(audio) = map.audio
                        {
                            audio.stop();
                            drop(audio);
                            self.audio_system.remove_dead_audio();
                            self.state.playback_speed = 1.;
                        }
                    }
                });
                ui.menu_button(self.data.locale.get("settings-label").to_string(), |ui| {
                    if ui.button(self.data.locale.get("settings-button")).clicked() {
                        self.open_settings();
                    }
                    let mut minecraft_font: bool = ui.memory_mut(|m| {
                        m.data
                            .get_persisted("use_minecraft_font".into())
                            .unwrap_or(true)
                    });
                    let old = minecraft_font;
                    ui.checkbox(
                        &mut minecraft_font,
                        self.data.locale.get("minecraft-font-label"),
                    );
                    if old != minecraft_font {
                        if minecraft_font {
                            setup_fonts(&[MINECRAFT_F, SOURCE_CODE_F], ctx);
                        } else {
                            setup_fonts(&[SOURCE_CODE_F, MINECRAFT_F], ctx);
                        }
                        ui.memory_mut(|m| {
                            m.data
                                .insert_persisted("use_minecraft_font".into(), minecraft_font)
                        });
                    }
                });
                let ml = self.data.locale.get("menu-label").to_string();
                ui.menu_button(ml.to_string(), |ui| {
                    if ui.button(ml).clicked() {
                        tracing::debug!(target: DB_LOGIC, "Returning to menu");
                        self.context = EditorContext::None;
                        self.render.renderer.beatmap.seek(0.);
                        self.history.clear();
                        if let Some(map) = self.map_editor.map.take()
                            && let Some(audio) = map.audio
                        {
                            audio.stop();
                            drop(audio);
                            self.audio_system.remove_dead_audio();
                            self.state.playback_speed = 1.;
                        }
                    }
                    let close_map = self.data.locale.get("close-map").to_string();
                    let close_diff = self.data.locale.get("close-difficulty").to_string();
                    'map_scope: {
                        let map = self.map_editor.map.as_mut();
                        'btn1: {
                            if ui.add_enabled(
                                map.is_some(),
                                egui::Button::new(close_map)
                            ).clicked() {
                                let Some(map) = map else { break 'btn1 };
                                self.render.renderer.beatmap.seek(0.);
                                self.history.clear();
                                if let Some(audio) = map.audio.take() {
                                    audio.stop();
                                    drop(audio);
                                    self.audio_system.remove_dead_audio();
                                    self.state.playback_speed = 1.;
                                }
                                self.map_editor.map = None;
                                break 'map_scope;
                            }
                        }
                        let has_controller = match map.as_ref() {
                            Some(m) => m.controller.is_some(),
                            None => false
                        };
                        'btn2: {
                            if ui.add_enabled(
                                has_controller,
                                egui::Button::new(close_diff)
                            ).clicked() {
                                let Some(map) = map else { break 'btn2 };
                                self.render.renderer.beatmap.seek(0.);
                                map.controller = None;
                                self.history.clear();
                                if let Some(audio) = map.audio.take() {
                                    audio.stop();
                                    drop(audio);
                                    self.audio_system.remove_dead_audio();
                                    self.state.playback_speed = 1.;
                                }
                                self.history.clear();
                            }
                        }

                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("scrub_controls")
            .exact_height(200.)
            .resizable(false)
            .show(ctx, |ui| {
                let s = unsafe { UnsafeMutRef::new(self) };
                match self.map_editor.map.as_ref() {
                    None => {
                        // No map selected
                    }
                    Some(map) => match map.controller.as_ref() {
                        None => {
                            // No difficulty selected
                        },
                        Some((controller, data)) => {
                            // In editor

                            if let Some(audio) = map.audio.as_ref()
                                && let Some(len) = audio.length_seconds()
                            {
                                let t = audio.position_seconds();
                                let time = format!("{} / {}", format_time(t), format_time(len));
                                ui.label(time);
                            }

                            ui.add_space(5.);

                            let mut rect = ui.available_rect_before_wrap();
                            rect.set_height(50.);

                            let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());

                            if resp.hovered() {
                                let scroll = ctx.input(|i| i.raw_scroll_delta.y);
                                if scroll != 0.0 {
                                    if alt {
                                        if let Some(audio) = map.audio.as_ref()
                                            && let Some(length_secs) = audio.length_seconds()
                                        {
                                            let length_beats =
                                                controller.runtime_data.seconds_to_beat(length_secs);
                                            let cursor = self.render.renderer.beatmap.beat() / length_beats;
                                            self.render
                                                .renderer
                                                .beatmap
                                                .spectrogram_zoom(scroll, cursor);
                                        }
                                    } else {
                                        self.render
                                            .renderer
                                            .beatmap
                                            .scroll(scroll.signum(), self.map_editor.scroll_step, self.map_editor.grid_snap);
                                    }
                                }
                            }

                            if (resp.clicked() || resp.dragged())
                                && let Some(pos) = resp.interact_pointer_pos()
                                && let Some(audio) = map.audio.as_ref()
                                && let Some(length_secs) = audio.length_seconds()
                            {
                                let frac_x = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);

                                let (start, end) = self.render.renderer.beatmap.spectrogram_range();
                                let u = start + (end - start) * frac_x;
                                let seek_secs = length_secs * u;
                                let beat = controller.runtime_data.seconds_to_beat(seek_secs);

                                let _ = audio.seek(seek_secs);
                                self.render.renderer.beatmap.seek(beat);
                            }

                            ui.painter().add(egui::PaintCallback {
                                rect,
                                callback: std::sync::Arc::new(eframe::egui_glow::CallbackFn::new(
                                    move |_info, painter| {
                                        if let Some(map) = s.map_editor.map.as_ref()
                                            && let Some((controller, _)) = map.controller.as_ref()
                                            && let Some(audio) = map.audio.as_ref()
                                            && let Some(length) = audio.length_seconds()
                                        {
                                            let gl = painter.gl();
                                            let sec = controller
                                                .runtime_data
                                                .beat_to_seconds(s.render.renderer.beatmap.beat());
                                            s.render.renderer.beatmap.render_spectrogram_ui(
                                                &mut s.ref_mut().render.renderer,
                                                gl,
                                                audio,
                                                sec,
                                                length,
                                            );
                                        }
                                    },
                                )),
                            });

                            ui.add_space(5.);

                            let vol = (self.data.audio_volume * 100.) as u32;
                            let mut volume = vol;
                            ui.add_sized(
                                [150., 20.],
                                egui::Slider::new(&mut volume, 0..=100)
                                    .suffix("%")
                                    .text(self.data.locale.get("audio-volume")),
                            );
                            if volume != vol {
                                let vol = volume as f32 / 100.;
                                self.data.audio_volume = vol;
                                if let Some(audio) = map.audio.as_ref()
                                {
                                    audio.set_volume(vol);
                                }
                            }

                            let spd = (self.state.playback_speed * 100.) as u32;
                            let mut speed = spd;
                            ui.add_sized(
                                [150., 20.],
                                egui::Slider::new(&mut speed, 0..=200)
                                    .suffix("%")
                                    .text(self.data.locale.get("playback-speed")),
                            );
                            if speed != spd {
                                let spd = speed as f32 / 100.;
                                self.state.playback_speed = spd;
                                if let Some(audio) = map.audio.as_ref()
                                {
                                    audio.set_speed(spd);
                                }
                            }

                            let def = self
                                .data
                                .locale
                                .get_with_args("default-value", &[("value".into(), 8.into())].into());

                            ui.add_sized(
                                [150., 20.],
                                egui::Slider::new(&mut self.render.renderer.beatmap.beat_spacing, 1f32..=20f32)
                                    .step_by(0.25)
                                    .text(self.data.locale.get("grid-spacing")),
                            )
                                .on_hover_text(def);
                        }
                    }
                }


            });

        egui::SidePanel::left("left_panel")
            .exact_width(300.)
            .resizable(false)
            .show(ctx, |ui| {
                match self.map_editor.map.as_ref() {
                    None => {
                        // No map selected
                    }
                    Some(map) => match map.controller.as_ref() {
                        None => {
                            // No difficulty selected
                        },
                        Some(_) => {
                            // In editor
                            let mut current = self.map_editor.selected_placement_type;

                            const TOOL_SIZE: f32 = 88.;
                            const TOOL_MARGIN: f32 = 5.;
                            const TOOL_PADDING: f32 = 5.;
                            const TOOL_ROWS: usize = 2;
                            const TOOL_COLS: usize = 3;

                            const T_WIDTH: f32 = TOOL_MARGIN * 2.
                                + TOOL_SIZE * TOOL_COLS as f32
                                + TOOL_PADDING * (TOOL_COLS - 1) as f32;
                            const T_HEIGHT: f32 = TOOL_MARGIN * 2.
                                + TOOL_SIZE * TOOL_ROWS as f32
                                + TOOL_PADDING * (TOOL_ROWS - 1) as f32;

                            let mut hint_rects: Vec<(&'static str, egui::Rect)> = Vec::new();

                            let panel = ui.allocate_ui_with_layout(
                                [T_WIDTH, T_HEIGHT].into(),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    let origin = ui.max_rect().min + egui::vec2(TOOL_MARGIN, TOOL_MARGIN);

                                    for (i, tool) in PlacementObjectType::iter_all().enumerate() {
                                        let col = i % TOOL_COLS;
                                        let row = i / TOOL_COLS;

                                        let img_src = tool.tool_icon();

                                        let pos = origin
                                            + egui::vec2(
                                            col as f32 * (TOOL_SIZE + TOOL_PADDING),
                                            row as f32 * (TOOL_SIZE + TOOL_PADDING),
                                        );
                                        let rect = egui::Rect::from_min_size(pos, egui::Vec2::splat(TOOL_SIZE));

                                        let response = ui
                                            .allocate_rect(rect, egui::Sense::click())
                                            .on_hover_text(tool.display_name());

                                        let bg = if tool == current {
                                            egui::Color32::from_gray(60)
                                        } else if response.hovered() {
                                            egui::Color32::from_gray(50)
                                        } else {
                                            egui::Color32::from_gray(0x20)
                                        };
                                        ui.painter().rect_filled(rect, 4.0, bg);

                                        egui::Image::new(img_src).paint_at(ui, rect);

                                        if response.clicked() {
                                            current = tool;
                                        }

                                        hint_rects.push((tool.key_name(), rect));
                                    }
                                },
                            );

                            for (name, rect) in hint_rects {
                                KeybindHint::new(name, rect, &panel.response)
                                    .gap(4.)
                                    .placement(ExtensionCorner::Interior(Corner::BottomRight))
                                    .show(ui, &self.data.keymaps);
                            }

                            if panel.response.contains_pointer() {
                                self.check_tool_keybinds(&mut current);
                            }

                            self.map_editor.selected_placement_type = current;

                            ui.separator();

                            let width = ui.available_width();
                            let padding = ui.style().spacing.item_spacing.x;

                            let w2 = (width - padding) / 2.;
                            let w3 = (width - padding * 2.) / 3.;
                            let w4 = (width - padding * 3.) / 4.;

                            let rd = RefDuper;
                            let placement_data = unsafe { rd.detach_mut_ref(&mut self.map_editor.placement_data) };

                            match current {
                                PlacementObjectType::ColorNote => {
                                    // Left/Right selector
                                    // Color selector
                                    // -----------------
                                    // rotation selector
                                    note_color_selector(
                                        self, ui,
                                        width, w2, w3,
                                        placement_data.color_note.note_colors(),
                                        false,
                                    );

                                    ui.add_space(10.);
                                    ui.separator();

                                    let is_dot = self.map_editor.placement_data.color_note.cut_direction == CutDirection::Dot;

                                    let disabled_text = self.data.locale.get("tooltip-beatmap_editor-place-cut_offset-disabled").to_string();

                                    if ui.add_sized(
                                        [width, width/2.],
                                        CutDirectionSelector::new(
                                            &mut placement_data.color_note.cut_direction,
                                            &mut placement_data.color_note.use_cut_offset,
                                            SelectorLayout::Horizontal,
                                            &self.data.keymaps,
                                        ).on_disable_hover_text(
                                            disabled_text
                                        ).cut_offset(
                                            &mut placement_data.color_note.cut_offset,
                                        ).image(if is_dot { COLOR_NOTE_DOT_BRIGHT.clone() } else { COLOR_NOTE_BRIGHT.clone() })
                                    ).hovered() {
                                        self.check_cut_dir_bind(&mut placement_data.color_note.cut_direction);
                                    }
                                },
                                PlacementObjectType::BombNote => {
                                    // color selector
                                    object_color_selector(
                                        self, ui,
                                        width, w3,
                                        placement_data.bomb.object_color(),
                                        false,
                                    );
                                },
                                PlacementObjectType::Obstacle => {
                                    // color selector
                                    // --------------
                                    // scale
                                    // rotation
                                    // duration
                                    object_color_selector(
                                        self, ui,
                                        width, w3,
                                        placement_data.obstacle.object_color(),
                                        false,
                                    );
                                },
                                PlacementObjectType::Chain => {
                                    // left/right selector
                                    // color selector
                                    // -------------------
                                    // slice count/squish selector
                                    // rotation selector
                                    note_color_selector(
                                        self, ui,
                                        width, w2, w3,
                                        placement_data.chain.note_colors(),
                                        false,
                                    );
                                },
                                PlacementObjectType::Arc => {
                                    // left/right selector
                                    // color selector
                                    // -------------------
                                    // head rotation selector
                                    // midpoint mode
                                    // tail rotation selector
                                    note_color_selector(
                                        self, ui,
                                        width, w2, w3,
                                        placement_data.arc.note_colors(),
                                        false,
                                    );
                                },
                                PlacementObjectType::Text => {
                                    // color/gradient selector
                                    // -----------------------
                                    // text
                                    // font
                                    // kerning, spacing options
                                    // anchor
                                    // ------------------------
                                    // offset
                                    // scale
                                    // rotation
                                    // duration
                                },
                            }

                            ui.add_space(ui.available_height() - 45.);
                            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                                ui.add_space(5.);
                                egui::ScrollArea::horizontal().show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.set_width(ui.available_width());
                                        let target = &mut self.cam().target;
                                        let spacing = ui.spacing().item_spacing.x;
                                        let width = (ui.available_width() - spacing * 2.0) / 3.0;
                                        let mut vars = HashMap::new();
                                        vars.insert("x".to_string(), target.x);
                                        vars.insert("y".to_string(), target.y);
                                        vars.insert("z".to_string(), target.z);
                                        ui.add_sized(
                                            [width, 20.],
                                            MathDragValue::new(&mut target.x, &mut vars).speed(0.01),
                                        );
                                        ui.add_sized(
                                            [width, 20.],
                                            MathDragValue::new(&mut target.y, &mut vars).speed(0.01),
                                        );
                                        ui.add_sized(
                                            [width, 20.],
                                            MathDragValue::new(&mut target.z, &mut vars).speed(0.01),
                                        );
                                    });
                                    ui.label(self.data.locale.get("camera-pivot"));
                                });
                            });
                        }
                    }
                }

            });

        egui::SidePanel::right("right_panel")
            .exact_width(300.)
            .resizable(false)
            .show(ctx, |_ui| {
                //
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(ctx.theme().default_visuals().panel_fill)
                    .inner_margin(0.),
            )
            .show(ctx, |ui| {
                let rd = RefDuper;
                let self2 = unsafe { rd.detach_mut_ref(self) };
                match self2.map_editor.map.as_mut() {
                    None => {
                        // No map selected
                        self.draw_map_selection_screen(self2, ctx, ui);
                    }
                    Some(map) => match map.controller.as_ref() {
                        None => {
                            ui.allocate_ui_with_layout(
                                ui.available_size(),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    draw_map_info(self, ui, map);
                                    draw_map_diffs(self, ui, map);
                                    ui.separator();
                                    draw_map_diff(self, ui, map);
                                },
                            );
                        }
                        Some((controller, _)) => {
                            self.draw_beatmap_scene(ctx, ui, gl, map, controller);
                        }
                    },
                }
            });
    }

    pub fn draw_map_selection_screen(
        &mut self,
        self2: &mut Self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        ui.allocate_ui_with_layout(
            [ui.available_width(), 250.].into(),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add_space(25.);
                ui.allocate_ui_with_layout(
                    [450., 200.].into(),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        if ui.add_sized(
                            [200., 200.],
                            ImageCard::new(OPEN_MAP_ICON.clone(), [200., 200.]).show_border(false)
                        ).on_hover_text(self.data.locale.get("open-beatmap-project")).clicked() {
                            self.await_beatmap_open();
                        }
                        ui.add_space(50.);
                        if ui.add_sized(
                            [200., 200.],
                            ImageCard::new(CREATE_MAP_ICON.clone(), [200., 200.]).show_border(false)
                        ).on_hover_text(self.data.locale.get("create-beatmap-project")).clicked() {
                            // TODO: create beatmap
                        }
                    }
                );
                ui.add_space(25.);
            },
        );

        let mut to_open = None;
        let mut to_remove = None;
        let scroll = ctx.input(|i| i.smooth_scroll_delta);
        egui::ScrollArea::horizontal()
            .max_width(ui.available_width())
            .id_salt("recent beatmap panel")
            .show(ui, |ui| {
                ui.scroll_with_delta((scroll.y * 2., 0.).into());
                ui.allocate_ui_with_layout(
                    [ui.available_width(), 200.].into(),
                    egui::Layout::left_to_right(egui::Align::Min),
                    |ui| {
                        for (i, modified, path, img) in
                            self.data.recents.iter().enumerate().filter_map(|(i, p)| {
                                if let ProjectType::Beatmap { img } = &p.kind {
                                    Some((i, p.modified, &p.path, img))
                                } else {
                                    None
                                }
                            })
                        {
                            let ext = path.with_extension("");
                            let Some(label) = ext.file_name() else {
                                continue;
                            };
                            let label = label.to_string_lossy();
                            let full_path = path.to_string_lossy();
                            ui.allocate_ui_with_layout(
                                [225., 400.].into(),
                                egui::Layout::top_down(egui::Align::Center),
                                |ui| {
                                    let src = if let Some(img) = img {
                                        format!(
                                            "file://{}",
                                            path.join(img).to_string_lossy()
                                        ).into()
                                    } else {
                                        MISSING_EDITOR_ICON.clone()
                                    };
                                    if ui.add(ImageCard::new(src, [200., 200.]).show_border(false).hover_image(EDIT_ICON.clone())).clicked() {
                                        to_open = Some(path);
                                    }
                                    ui.label(egui::RichText::new(label).strong())
                                        .on_hover_text(full_path);
                                    ui.label(modified.to_string());

                                    ui.allocate_ui_with_layout(
                                        [225., ui.available_height().max(1.)].into(),
                                        egui::Layout::bottom_up(egui::Align::Center),
                                        |ui| {
                                            ui.add_space(20.);
                                            if ui
                                                .button(self.data.locale.get("remove-from-list"))
                                                .clicked()
                                            {
                                                to_remove = Some(i);
                                            }
                                        },
                                    );
                                },
                            );
                        }
                    },
                );
            });

        if let Some(path) = to_open {
            let _ = self2.load_beatmap(
                &mut self.audio_system,
                path.clone(),
                &self.state.gl,
                &mut self.render.renderer,
                self.data.audio_volume,
            );
        }
        if let Some(i) = to_remove {
            self.data.recents.remove(i);
        }
    }

    fn draw_beatmap_scene(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        gl: &glow::Context,
        map: &BeatmapProject,
        controller: &BeatmapController<ObjectSource>,
    ) {
        let rect = ui.available_rect_before_wrap();
        self.state.vp_rect = rect;
        let w = rect.width();
        let h = rect.height();

        let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());
        self.handle_3d_input(&resp, ctx, gl);
        let (click, shift) = ui.input(|i| (i.pointer.primary_clicked(), i.modifiers.shift));
        let raw_mouse = ui.input(|i| i.pointer.latest_pos());
        let mouse_pos = raw_mouse.map(|p| Vec2::new(p.x - rect.min.x, p.y - rect.min.y));

        let mut mouse_pos = mouse_pos.map(|mp| (mp.x, h - mp.y));

        if let Some(mp) = raw_mouse
            && !rect.contains(mp)
        {
            mouse_pos = None;
        }

        if let Some(audio) = map.audio.as_ref()
            && audio.is_playing()
        {
            let sec = audio.position_seconds();
            let beat = controller.runtime_data.seconds_to_beat(sec);
            self.render.renderer.beatmap.seek(beat);
        }

        let s = unsafe { UnsafeMutRef::new(self) };

        ui.painter().add(egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(eframe::egui_glow::CallbackFn::new(
                move |_info, painter| {
                    let gl = painter.gl();
                    unsafe {
                        let view = s.ref_mut().cam().view_mat();
                        let proj = s.ref_mut().cam().proj_mat(w, h);

                        match s.state.view_style {
                            editor::ViewStyle::Beatcraft { blackout_sky: true } => {
                                gl.clear_color(0., 0., 0., 1.);
                            }
                            _ => {
                                gl.clear_color(0.07, 0.08, 0.11, 1.);
                                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                            }
                        }

                        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                        gl.enable(glow::DEPTH_TEST);
                        gl.depth_mask(true);

                        draw_map_gl(
                            &s,
                            gl,
                            &view,
                            &proj,
                            (w as i32, h as i32),
                            mouse_pos,
                            click,
                            shift,
                        );

                        if s.state.show_grid && s.state.view_style == ViewStyle::Edit {
                            s.render.renderer.draw_map_grid(gl, &view, &proj);
                        }
                    }
                },
            )),
        });
    }

    pub fn load_beatmap(
        &mut self,
        audio_system: &mut AudioSystem<NativeOpenAL>,
        folder: PathBuf,
        gl: &Context,
        renderer: &mut Renderer,
        volume: f32,
    ) -> Result<(), MapLoadError> {
        let f2 = folder.clone();
        self.map_editor
            .load(audio_system, folder, gl, renderer, volume)?;
        self.data
            .mark_project_modified(&f2, crate::config::ProjectKind::Beatmap);
        Ok(())
    }
}

fn draw_map_info(app: &mut App, ui: &mut egui::Ui, map: &mut BeatmapProject) {
    if let Some(i) = map.info.as_mut() {
        ui.add_space(15.);
        match i {
            InfoFile::V2(v2) => {
                ui.allocate_ui_with_layout(
                    [ui.available_width(), 200.].into(),
                    egui::Layout::left_to_right(egui::Align::Min),
                    |ui| {
                        ui.add_space(15.);
                        let cover = map.folder.join(&v2.cover_image_filename);
                        ui.add_sized(
                            [200., 200.],
                            if cover.exists() {
                                egui::Image::new(format!("file://{}", cover.to_string_lossy()))
                            } else {
                                egui::Image::new(MISSING_EDITOR_ICON.clone())
                            },
                        );
                        ui.allocate_ui_with_layout(
                            [ui.available_width(), 200.].into(),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.label("Info V2");
                                ui.label(&v2.song_name);
                                ui.label(&v2.song_sub_name);
                                ui.label(format!(
                                    "Artist: {}  BPM: {:.2}  Mappers: {}",
                                    v2.song_author_name, v2.bpm, v2.level_author_name
                                ));
                            },
                        );
                    },
                );
            }
            InfoFile::V4(v4) => {
                ui.allocate_ui_with_layout(
                    [ui.available_width(), 200.].into(),
                    egui::Layout::left_to_right(egui::Align::Min),
                    |ui| {
                        ui.add_space(15.);
                        let cover = map.folder.join(&v4.cover_image_filename);
                        ui.add_sized(
                            [200., 200.],
                            if cover.exists() {
                                egui::Image::new(format!("file://{}", cover.to_string_lossy()))
                            } else {
                                egui::Image::new(MISSING_EDITOR_ICON.clone())
                            },
                        );
                        ui.allocate_ui_with_layout(
                            [ui.available_width(), 200.].into(),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.label("Info V2");
                                ui.label(&v4.song.title);
                                ui.label(&v4.song.sub_title);
                                ui.label(format!(
                                    "Artist: {}  BPM: {:.2}",
                                    v4.song.author, v4.audio.bpm,
                                ));
                            },
                        );
                    },
                );
            }
        }
        ui.add_space(15.);
    }
}

fn draw_map_diffs(app: &mut App, ui: &mut egui::Ui, map: &mut BeatmapProject) {
    ui.allocate_ui_with_layout(
        [ui.available_width(), 200.].into(),
        egui::Layout::left_to_right(egui::Align::Min),
        |ui| {
            ui.add_space(15.);
            for (si, set) in map.sets.iter_mut().enumerate() {
                ui.allocate_ui_with_layout(
                    [200., 200.].into(),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.label(set.set.display_name());
                        ui.separator();
                        for (di, diff) in set.diffs.iter_mut().enumerate() {
                            let sel = if let Some((si0, di0)) = map.selected_diff {
                                si0 == si && di0 == di
                            } else {
                                false
                            };
                            if ui
                                .add_sized(
                                    [180., 25.],
                                    egui::Button::new(diff.difficulty.display_name()).selected(sel),
                                )
                                .clicked()
                            {
                                if sel {
                                    map.selected_diff = None;
                                } else {
                                    map.selected_diff = Some((si, di));
                                }
                            }
                        }
                        ui.add_space(ui.available_height());
                    },
                );
            }
        },
    );
}

fn draw_map_diff(app: &mut App, ui: &mut egui::Ui, map: &mut BeatmapProject) {
    if let Some((si, di)) = map.selected_diff
        && let Some(set) = map.sets.get_mut(si)
        && let Some(diff) = set.diffs.get_mut(di)
    {
        ui.allocate_ui_with_layout(
            [ui.available_width(), 200.].into(),
            egui::Layout::left_to_right(egui::Align::Min),
            |ui| {
                ui.add_space(15.);
                ui.allocate_ui_with_layout(
                    [ui.available_width(), 200.].into(),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.label(format!("NJS: {}  OFFSET: {}", diff.njs, diff.njs_offset));
                        if let Some(v4_data) = diff.v4_data.as_mut() {
                            ui.label(format!("MAPPERS: {}", v4_data.mappers.join(", ")));
                            ui.label(format!("LIGHTERS: {}", v4_data.lighters.join(", ")));
                        }
                        if let Some(path) = diff.beatmap_file.as_deref() {
                            ui.label(format!("PATH: {}", path.to_string_lossy()));
                            if ui.button(app.data.locale.get("open")).clicked() {
                                let path = map.folder.join(path);
                                let data = std::fs::read(path).unwrap();
                                match serde_json::from_slice::<BeatmapFile>(&data) {
                                    Ok(diff2) => {
                                        if let Some(audio) = map.audio.as_ref() {
                                            if let Some(sample_count) = audio.sample_count() {
                                                let bpm_regions = match map.audio_info.as_ref() {
                                                    None => Vec::new(),
                                                    Some(info) => {
                                                        let r = info.bpm_regions();
                                                        tracing::debug!(target: DB_DATA, ?r, "Loaded BPM regions:");
                                                        r
                                                    }
                                                };
                                                app.selection = Selection::None;
                                                let editing_data = EditingData::from_beatmap(&diff2);
                                                let mut rng = ThreadRng::default();
                                                let runtime_data = RuntimeData::new(
                                                    diff.njs,
                                                    map.info.as_ref().unwrap().bpm(),
                                                    diff.njs_offset,
                                                    bpm_regions,
                                                    sample_count,
                                                    audio.sample_rate,
                                                );
                                                let controller = editing_data.canonicalize(
                                                    &mut rng,
                                                    runtime_data,
                                                    Default::default(),
                                                ).unwrap();
                                                map.controller = Some((controller, editing_data));
                                            } else {
                                                tracing::warn!(target: DB_AUDIO, "Audio sample_count is None");
                                            }

                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!(target: DB_DATA, "Error loading beatmap file:\n{}", e);
                                    }
                                }

                            }
                        }
                    }
                );
            }
        );
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HitTargetType {
    Object(ObjectType),
    Grid {
        row: isize,
        column: isize,
        index: usize,
    },
}

impl From<ObjectType> for HitTargetType {
    fn from(value: ObjectType) -> Self {
        Self::Object(value)
    }
}

pub struct Hit {
    pub distance: f32,
    pub typ: HitTargetType,
    pub index: usize,
}

fn check_collision(
    mat: Mat4,
    hitbox: HitBox,
    ray_pos: Vec3,
    ray_dir: Vec3,
    typ: impl Into<HitTargetType>,
    index: usize,
) -> Option<Hit> {
    let inv = mat.inverse();

    let local_origin = inv.transform_point3(ray_pos);
    let local_dir = inv.transform_vector3(ray_dir);

    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    macro_rules! check_axis {
        ($axis:tt) => {
            if local_dir.$axis != 0. {
                let mut t1 = (hitbox.min.$axis - local_origin.$axis) / local_dir.$axis;
                let mut t2 = (hitbox.max.$axis - local_origin.$axis) / local_dir.$axis;
                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2)
                }
                t_min = t_min.max(t1);
                t_max = t_max.min(t2);
            } else {
                if local_origin.$axis < hitbox.min.$axis || local_origin.$axis > hitbox.max.$axis {
                    return None;
                }
            }
        };
    }

    check_axis! { x }
    check_axis! { y }
    check_axis! { z }

    if t_max >= t_min && t_max >= 0. {
        Some(Hit {
            distance: t_min.max(0.),
            typ: typ.into(),
            index,
        })
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_map_gl(
    s: &UnsafeMutRef<App>,
    gl: &glow::Context,
    view: &Mat4,
    proj: &Mat4,
    window: (i32, i32),
    mouse: Option<(f32, f32)>,
    click: bool,
    shift: bool,
) {
    let (w, h) = window;
    let vp = proj * view;
    let controller = s
        .ref_mut()
        .map_editor
        .map
        .as_mut()
        .unwrap()
        .controller
        .as_mut()
        .unwrap();

    let rd = RefDuper;
    let (controller, _) = unsafe { rd.detach_mut_ref(controller) };

    let mut note_instances = Vec::new();
    let mut bomb_instances = Vec::new();
    let mut obstacle_instances = Vec::new();
    let mut chain_head_instances = Vec::new();
    let mut chain_tail_instances = Vec::new();

    let mut arrow_instances = Vec::new();
    let mut dot_instances = Vec::new();
    let mut chain_dot_instances = Vec::new();

    let mut note_highlights = Vec::new();
    let mut bomb_highlights = Vec::new();
    let mut obstacle_highlights = Vec::new();
    let mut chain_head_highlights = Vec::new();
    let mut chain_tail_highlights = Vec::new();

    let mut arrow_highlights = Vec::new();
    let mut dot_highlights = Vec::new();
    let mut chain_dot_highlights = Vec::new();

    let beatmap = &s.render.renderer.beatmap;
    let beat = beatmap.beat();

    let mut note_sel_filter = HashSet::new();
    let mut bomb_sel_filter = HashSet::new();
    let mut obstacle_sel_filter = HashSet::new();
    let mut chain_sel_filter = HashSet::new();
    let mut arc_sel_filter = HashSet::new();

    if let Selection::BeatmapObject(objects) = &s.selection {
        for (t, i) in objects {
            match t {
                ObjectType::ColorNote => note_sel_filter.insert(*i),
                ObjectType::BombNote => bomb_sel_filter.insert(*i),
                ObjectType::Obstacle => obstacle_sel_filter.insert(*i),
                ObjectType::ChainHead | ObjectType::ChainLink => chain_sel_filter.insert(*i),
                ObjectType::ArcHead | ObjectType::ArcTail => arc_sel_filter.insert(*i),
            };
        }
    }

    let (orig, dir) = if let Some((mx, my)) = mouse {
        App::unproject(Vec2::new(mx, my), Vec2::new(w as f32, h as f32), &vp)
    } else {
        (Vec3::ZERO, Vec3::ZERO)
    };
    let mut hits = Vec::new();

    const HITBOX: HitBox = HitBox::new(Vec3::new(-0.25, -0.25, 0.), Vec3::new(0.25, 0.25, 0.));
    s.ref_mut().render.renderer.beatmap.hovered_placement_cell = u32::MAX;
    let mut base = Mat4::from_quat(Quat::from_rotation_y(beatmap.placement_r));
    base *= Mat4::from_translation(Vec3::new(0., 0., beatmap.placement_z));

    let mut i = 0;
    for (y, row) in [(0.3, 0), (0.9, 1), (1.5, 2)] {
        for (x, col) in [(0.9, 0), (0.3, 1), (-0.3, 2), (-0.9, 3)] {
            let pos = base * Mat4::from_translation(Vec3::new(x, y, 0.));
            if let Some(hit) = check_collision(
                pos,
                HITBOX,
                orig,
                dir,
                HitTargetType::Grid {
                    row,
                    column: col,
                    index: i,
                },
                i,
            ) {
                hits.push((hit, None));
            }
            i += 1;
        }
    }

    let beat_spacing = match s.state.view_style {
        ViewStyle::Edit => Some(s.render.renderer.beatmap.beat_spacing),
        ViewStyle::Beatcraft { .. } => None,
    };

    for (i, t, object) in controller
        .color_notes
        .iter()
        .enumerate()
        .map(|(i, x)| (i, ObjectType::ColorNote, x as &dyn GameObjectExt))
        .chain(
            controller
                .bomb_notes
                .iter()
                .enumerate()
                .map(|(i, x)| (i, ObjectType::BombNote, x as &dyn GameObjectExt)),
        )
        .chain(
            controller
                .obstacles
                .iter()
                .enumerate()
                .map(|(i, x)| (i, ObjectType::Obstacle, x as &dyn GameObjectExt)),
        )
        .chain(
            controller
                .chain_notes
                .iter()
                .enumerate()
                .map(|(i, x)| (i, ObjectType::ChainHead, x as &dyn GameObjectExt)),
        )
        .chain(
            controller
                .arcs
                .iter()
                .enumerate()
                .map(|(i, x)| (i, ObjectType::ArcHead, x as &dyn GameObjectExt)),
        )
    {
        let wp = Mat4::IDENTITY;

        if let Some(mat) = match s.state.view_style {
            ViewStyle::Edit => object.animate_simple(wp, beat, beatmap.beats_before, beatmap.visible_beat_count, beatmap.beat_spacing),
            ViewStyle::Beatcraft { .. } => {
                object.animate_complex(wp, beat, &controller.runtime_data)
            }
        } {
            let inst = match s.state.view_style {
                ViewStyle::Edit => object.get_editor_instance(
                    Vec4::ZERO,
                    mat,
                    &controller.runtime_data.color_scheme,
                    s.render.renderer.beatmap.beat_spacing,
                ),
                ViewStyle::Beatcraft { .. } => {
                    object.get_instance(Vec4::ZERO, mat, &controller.runtime_data.color_scheme)
                }
            };
            if mouse.is_some()
                && object.upcast_chain_head().is_none()
                && let Some(hit) =
                    check_collision(mat, object.editor_hitbox(beat_spacing), orig, dir, t, i)
            {
                hits.push((hit, Some((t, inst, None))));
            }
            let is_highlighted = match t {
                ObjectType::ColorNote => {
                    note_instances.push(inst.into());
                    if note_sel_filter.contains(&i) {
                        note_highlights.push(inst.into_data().highlight(Vec4::splat(1.)));
                        true
                    } else {
                        false
                    }
                }
                ObjectType::BombNote => {
                    bomb_instances.push(inst.into());
                    if bomb_sel_filter.contains(&i) {
                        bomb_highlights.push(inst.into_data().highlight(Vec4::splat(1.)));
                        true
                    } else {
                        false
                    }
                }
                ObjectType::Obstacle => {
                    obstacle_instances.push(inst.into());
                    if obstacle_sel_filter.contains(&i) {
                        obstacle_highlights.push(inst.into_data().highlight(Vec4::splat(1.)));
                        true
                    } else {
                        false
                    }
                }
                ObjectType::ChainHead => {
                    chain_head_instances.push(inst.into());
                    arrow_instances.push(inst.into());
                    let Some(chain) = object.upcast_chain_head() else {
                        unreachable!("only chain note heads are marked as ChainHead")
                    };
                    let links = chain.get_links();
                    let mut hit0 = None;
                    let mut insts = Vec::with_capacity(links.len() + 1);
                    let sel = chain_sel_filter.contains(&i);
                    if mouse.is_some()
                        && let Some(hit) = check_collision(
                            mat,
                            object.editor_hitbox(beat_spacing),
                            orig,
                            dir,
                            t,
                            i,
                        )
                    {
                        hit0 = Some(hit);
                    }
                    for link in links {
                        if let Some(mat) = match s.state.view_style {
                            ViewStyle::Edit => {
                                link.animate_simple(wp, beat, beatmap.beats_before, beatmap.visible_beat_count, beatmap.beat_spacing)
                            }
                            ViewStyle::Beatcraft { .. } => {
                                link.animate_complex(wp, beat, &controller.runtime_data)
                            }
                        } {
                            if mouse.is_some()
                                && hit0.is_none()
                                && let Some(hit) = check_collision(
                                    mat,
                                    link.editor_hitbox(beat_spacing),
                                    orig,
                                    dir,
                                    t,
                                    i,
                                )
                            {
                                hit0 = Some(hit);
                            }
                            let inst = match s.state.view_style {
                                ViewStyle::Edit => object.get_editor_instance(
                                    Vec4::ZERO,
                                    mat,
                                    &controller.runtime_data.color_scheme,
                                    s.render.renderer.beatmap.beat_spacing,
                                ),
                                ViewStyle::Beatcraft { .. } => object.get_instance(
                                    Vec4::ZERO,
                                    mat,
                                    &controller.runtime_data.color_scheme,
                                ),
                            };
                            chain_tail_instances.push(inst.into());
                            chain_dot_instances.push(inst.into());
                            if sel {
                                chain_tail_highlights
                                    .push(inst.into_data().highlight(Vec4::splat(1.)));
                                chain_dot_highlights
                                    .push(inst.into_data().highlight(Vec4::splat(1.)));
                            }
                            insts.push(inst);
                        }
                    }
                    if let Some(hit) = hit0 {
                        hits.push((hit, Some((t, inst, Some(insts)))));
                    }
                    if sel {
                        chain_head_highlights.push(inst.into_data().highlight(Vec4::splat(1.)));
                        arrow_highlights.push(inst.into_data().highlight(Vec4::splat(1.)));
                    }
                    false
                }
                _ => false,
            };
            match object.arrow_type() {
                ArrowType::None => {}
                ArrowType::Arrow => {
                    arrow_instances.push(inst.into());
                    if is_highlighted {
                        arrow_highlights.push(inst.into_data().highlight(Vec4::splat(1.)));
                    }
                }
                ArrowType::Dot => {
                    dot_instances.push(inst.into());
                    if is_highlighted {
                        dot_highlights.push(inst.into_data().highlight(Vec4::splat(1.)));
                    }
                }
                ArrowType::ChainDot => chain_dot_instances.push(inst.into()),
            }
        }
    }

    let m = &s.map_editor.mesh_set.note_mesh;
    let b = &s.map_editor.mesh_set.bomb_mesh;
    let o = &s.map_editor.mesh_set.obstacle_mesh;
    let c = &s.map_editor.mesh_set.chain_head_mesh;
    let cl = &s.map_editor.mesh_set.chain_body_mesh;

    let a = &s.map_editor.mesh_set.arrow_mesh;
    let d = &s.map_editor.mesh_set.dot_mesh;
    let cd = &s.map_editor.mesh_set.chain_dot_mesh;

    let mut calls = vec![
        MeshDrawCall {
            mesh: m,
            instances: note_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: false,
            mirror: false,
            obstacle: false,
            highlight: false,
        },
        MeshDrawCall {
            mesh: c,
            instances: chain_head_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: false,
            mirror: false,
            obstacle: false,
            highlight: false,
        },
        MeshDrawCall {
            mesh: cl,
            instances: chain_tail_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: false,
            mirror: false,
            obstacle: false,
            highlight: false,
        },
        MeshDrawCall {
            mesh: a,
            instances: arrow_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: true,
            mirror: false,
            obstacle: false,
            highlight: false,
        },
        MeshDrawCall {
            mesh: d,
            instances: dot_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: true,
            mirror: false,
            obstacle: false,
            highlight: false,
        },
        MeshDrawCall {
            mesh: cd,
            instances: chain_dot_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: true,
            mirror: false,
            obstacle: false,
            highlight: false,
        },
        MeshDrawCall {
            mesh: b,
            instances: bomb_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: false,
            mirror: false,
            obstacle: false,
            highlight: false,
        },
        MeshDrawCall {
            mesh: o,
            instances: obstacle_instances,
            wireframe: false,
            cull: false,
            bloomfog: false,
            solid: false,
            bloom: false,
            mirror: false,
            obstacle: true,
            highlight: false,
        },
        MeshDrawCall {
            mesh: m,
            instances: note_highlights,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: false,
            bloom: false,
            mirror: false,
            obstacle: false,
            highlight: true,
        },
        MeshDrawCall {
            mesh: c,
            instances: chain_head_highlights,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: false,
            bloom: false,
            mirror: false,
            obstacle: false,
            highlight: true,
        },
        MeshDrawCall {
            mesh: cl,
            instances: chain_tail_highlights,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: false,
            bloom: false,
            mirror: false,
            obstacle: false,
            highlight: true,
        },
        MeshDrawCall {
            mesh: a,
            instances: arrow_highlights,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: false,
            bloom: false,
            mirror: false,
            obstacle: false,
            highlight: true,
        },
        MeshDrawCall {
            mesh: d,
            instances: dot_highlights,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: false,
            bloom: false,
            mirror: false,
            obstacle: false,
            highlight: true,
        },
        MeshDrawCall {
            mesh: cd,
            instances: chain_dot_highlights,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: false,
            bloom: false,
            mirror: false,
            obstacle: false,
            highlight: true,
        },
        MeshDrawCall {
            mesh: b,
            instances: bomb_highlights,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: false,
            bloom: false,
            mirror: false,
            obstacle: false,
            highlight: true,
        },
        MeshDrawCall {
            mesh: o,
            instances: obstacle_highlights,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: false,
            bloom: false,
            mirror: false,
            obstacle: false,
            highlight: true,
        },
    ];

    hits.sort_by(|(hit0, _), (hit1, _)| hit0.distance.partial_cmp(&hit1.distance).unwrap());
    if let Some((
        Hit {
            distance: _,
            typ,
            index,
        },
        object_data,
    )) = hits.first()
    {
        match typ {
            HitTargetType::Object(typ) => {
                if click {
                    if shift && let Selection::BeatmapObject(sel) = &mut s.ref_mut().selection {
                        sel.push((*typ, *index));
                    } else {
                        s.ref_mut().selection = Selection::BeatmapObject(vec![(*typ, *index)]);
                    }
                }
            }
            HitTargetType::Grid { row, column, index } => {
                // TODO: grid placement logic
                s.ref_mut().render.renderer.beatmap.hovered_placement_cell = *index as u32;
            }
        }
        if let Some((closest_ty, closest, links)) = object_data {
            let (m, a, m2) = match (closest_ty, links) {
                (ObjectType::ChainHead, Some(link_insts)) => (
                    c,
                    Some(a),
                    Some(
                        link_insts
                            .iter()
                            .map(|i| Into::<InstanceData>::into(*i))
                            .collect::<Vec<_>>(),
                    ),
                ),
                (ObjectType::ColorNote, None) => (m, Some(a), None),
                (ObjectType::BombNote, None) => (b, None, None),
                (ObjectType::Obstacle, None) => (o, None, None),
                // (ObjectType::ArcHead, None) => todo!(),
                // (ObjectType::ArcTail, None) => todo!(),
                (t, n) => {
                    unreachable!("Hit check encountered unexpected type-link combo: {t:?}, {n:?}")
                }
            };
            const HIGHLIGHT_COLOR: Vec4 = Vec4::new(0.01, 0.8, 0.01, 1.0);
            calls.push(MeshDrawCall {
                mesh: m,
                instances: vec![Into::<InstanceData>::into(*closest).highlight(HIGHLIGHT_COLOR)],
                wireframe: false,
                cull: true,
                bloomfog: false,
                solid: false,
                bloom: false,
                mirror: false,
                obstacle: false,
                highlight: true,
            });
            if let Some(arrow) = a {
                calls.push(MeshDrawCall {
                    mesh: arrow,
                    instances: vec![
                        Into::<InstanceData>::into(*closest).highlight(HIGHLIGHT_COLOR),
                    ],
                    wireframe: false,
                    cull: true,
                    bloomfog: false,
                    solid: false,
                    bloom: false,
                    mirror: false,
                    obstacle: false,
                    highlight: true,
                });
            }
            if let Some(instances) = m2 {
                let instances: Vec<InstanceData> = instances
                    .into_iter()
                    .map(|i| i.highlight(HIGHLIGHT_COLOR))
                    .collect();
                calls.push(MeshDrawCall {
                    mesh: cl,
                    instances: instances.clone(),
                    wireframe: false,
                    cull: true,
                    bloomfog: false,
                    solid: false,
                    bloom: false,
                    mirror: false,
                    obstacle: false,
                    highlight: true,
                });
                calls.push(MeshDrawCall {
                    mesh: cd,
                    instances,
                    wireframe: false,
                    cull: true,
                    bloomfog: false,
                    solid: false,
                    bloom: false,
                    mirror: false,
                    obstacle: false,
                    highlight: true,
                });
            }
        }
    }

    match s.state.view_style {
        ViewStyle::Edit => {
            s.ref_mut()
                .render
                .renderer
                .draw_meshes(gl, view, proj, &calls, None, false, false, window);
        }
        ViewStyle::Beatcraft { blackout_sky } => {
            s.ref_mut().render.renderer.draw_meshes_fancy(
                gl,
                view,
                proj,
                &calls,
                window,
                if s.state.show_grid {
                    GridType::BeatGrid
                } else {
                    GridType::None
                },
                s.render.mirror.as_ref(),
                false,
                s.view.fog_heights.unwrap_or([-50., -30.]),
                true,
                if blackout_sky {
                    (0., 0., 0., 1.)
                } else {
                    (0.07, 0.08, 0.11, 1.)
                },
            );
        }
    }

    match s.state.view_style {
        ViewStyle::Edit => {}
        ViewStyle::Beatcraft { .. } => {}
    }
}
