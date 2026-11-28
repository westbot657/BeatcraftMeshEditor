use std::collections::HashMap;
use std::sync::Arc;
use glam::{Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use beatcraft_audiolib::{Audio, NativeOpenAL};
use crate::DB_DATA;
use beatmap_core::beatmap::{BombNote, ChainNote, ChainNoteLinkData, ColorNote, CutDirectionExt, NoteColor, ObjectColor, Obstacle, RawNoteColor};
use beatmap_core::{BeatmapController, RuntimeData, TimeUnit};
use beatmap_core::tracks_rs::animation::events::EventData;
use bs_mapping_data::v2::{self, ObstacleV2Type, V2Note};
use bs_mapping_data::{BeatmapFile, BpmRegion, CutDirection};
use object_data::{ArcData, BombData, ChainData, NoteData, ObjectSource, ObstacleData, ObstacleTextData, Template, TemplatePlacement};
use resolver::{ResolveError, Value};
use crate::beatmap::{BeatmapProjectDiff};

pub mod values;
pub mod resolver;
pub mod object_data;

#[derive(Serialize, Deserialize)]
pub enum DataElement {
    Note(NoteData),
    Bomb(BombData),
    Obstacle(ObstacleData),
    Chain(ChainData),
    Arc(ArcData),
    Template(TemplatePlacement),
    ObstacleText(ObstacleTextData),
}

/// Data specific to a single beatmap
// #[derive(Default, Serialize, Deserialize)]
pub struct EditingData {
    pub elements: Vec<DataElement>,
    pub values: HashMap<String, Value>,
    pub templates: HashMap<String, Template>,
    pub point_definitions: Option<ahash::HashMap<String, serde_json::Value>>,
    pub custom_events: Option<Vec<serde_json::Value>>,
    pub runtime_data: RuntimeData,
}

/// Global, cross-beatmap data
#[derive(Default, Serialize, Deserialize)]
pub struct GlobalEditingData {
    pub values: HashMap<String, Value>,
    pub templates: HashMap<String, Template>,
}

#[derive(thiserror::Error, Debug)]
pub enum CanonicalizationError {
    #[error("{0}")]
    ResolveError(#[from] ResolveError),
    #[error("Template not found: {0}")]
    InvalidTemplate(String),
}

impl EditingData {

    pub fn from_beatmap(
        file: &BeatmapFile,
        diff: &BeatmapProjectDiff,
        bpm: f32,
        bpm_regions: Vec<BpmRegion>,
        sample_count: usize,
        audio: &Arc<Audio<NativeOpenAL>>
    ) -> Self {
        let runtime_data = RuntimeData::new(
            diff.njs,
            bpm,
            diff.njs_offset,
            bpm_regions,
            sample_count,
            audio.sample_rate,
        );

        let mut elements = Vec::new();
        let mut point_definitions = None;
        let mut custom_events = None;

        match file {
            BeatmapFile::V2(v2) => {
                if let Some(custom) = &v2.custom_data {
                    if let Some(points) = custom.point_definitions.as_ref() {
                        let mut defs = ahash::HashMap::default();
                        for point_def in points.iter() {
                            defs.insert(
                                point_def.name.clone(),
                                point_def.points.clone(),
                            );
                        }
                        point_definitions = Some(defs);
                    }
                    if let Some(events) = custom.custom_events.as_ref() {
                        custom_events = Some(events.clone());
                    }
                }
                let mut rotations = Vec::new();
                for event in v2.events.iter() {
                    if let v2::V2Event::SpawnRotation(rot) = event {
                        rotations.push(*rot);
                    }
                }
                rotations.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap());
                elements.reserve(v2.notes.len() + v2.obstacles.len() + v2.arcs.len());
                for note in v2.notes.iter() {
                    match note {
                        V2Note::Note(note) => {
                            let note_type = note.typ.into();
                            let beat = note.time;
                            let mut rotation_lane_deg = 0i32;
                            'rot: for rot in rotations.iter() {
                                if (rot.beat < beat)
                                    || (rot.beat == beat && rot.execution_time.is_early())
                                {
                                    rotation_lane_deg += rot.rotation_angle.get_degrees();
                                } else {
                                    break 'rot;
                                }
                            }
                            elements.push(DataElement::Note(NoteData {
                                beat: beat.into(),
                                cut_direction: note.cut_direction.into(),
                                x: note.line_index.into(),
                                y: note.line_layer.into(),
                                lane_rotation_deg: rotation_lane_deg.into(),
                                note_type,
                                angle_offset_deg: 0f32.into(),
                                custom_data: (&note.custom_data).into()
                            }));
                        }
                        V2Note::Bomb(bomb) => {
                            let beat = bomb.beat;
                            let mut rotation_lane_deg = 0i32;
                            'rot: for rot in rotations.iter() {
                                if (rot.beat < beat)
                                    || (rot.beat == beat && rot.execution_time.is_early())
                                {
                                    rotation_lane_deg += rot.rotation_angle.get_degrees();
                                } else {
                                    break 'rot;
                                }
                            }
                            elements.push(DataElement::Bomb(BombData {
                                beat: beat.into(),
                                x: bomb.line_index.into(),
                                y: bomb.line_layer.into(),
                                lane_rotation_deg: rotation_lane_deg.into(),
                                custom_data: (&bomb.custom_data).into()
                            }));
                        }
                    }
                }
                for obst in v2.obstacles.iter() {
                    let (x, y, width, height) = match obst.typ {
                        ObstacleV2Type::FullHeight => {
                            (obst.line_index, obst.line_layer, obst.width, 5.)
                        }
                        ObstacleV2Type::Crouch => {
                            (obst.line_index, obst.line_layer + 2., obst.width, 3.)
                        }
                        ObstacleV2Type::Free => {
                            (obst.line_index, obst.line_layer, obst.width, obst.height)
                        }
                    };
                    let beat = obst.beat;
                    let mut lane_rotation_deg = 0;
                    'rot: for rot in rotations.iter() {
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early())
                        {
                            lane_rotation_deg += rot.rotation_angle.get_degrees();
                        } else {
                            break 'rot;
                        }
                    }
                    elements.push(DataElement::Obstacle(ObstacleData {
                        beat: beat.into(),
                        x: x.into(),
                        y: y.into(),
                        duration: obst.duration.into(),
                        lane_rotation_deg: lane_rotation_deg.into(),
                        width: width.into(),
                        height: height.into(),
                        custom_data: (&obst.custom_data).into(),
                    }));
                }
            }
            BeatmapFile::V3(v3) => {
                if let Some(custom) = &v3.custom_data {
                    if let Some(defs) = custom.point_definitions.as_ref() {
                        let mut out = ahash::HashMap::default();
                        for (k, v) in defs {
                            out.insert(
                                k.clone(),
                                v.clone(),
                            );
                        }
                        point_definitions = Some(out);
                    }
                }
                let mut rotations = Vec::new();
                for event in v3.rotation_events.iter() {
                    rotations.push(*event);
                }
                rotations.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap());
                let mut chain_masks = Vec::with_capacity(v3.chains.len());
                for chain in v3.chains.iter() {
                    let note_type = chain.color.into();
                    let beat = chain.head_beat;
                    let t_beat = chain.tail_beat;
                    let mut lane_rotation_deg = 0.;
                    let mut tail_lane_rotation = 0.;
                    let mb = beat.max(t_beat);
                    for rot in rotations.iter() {
                        if rot.beat > mb {
                            break;
                        }
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early())
                        {
                            lane_rotation_deg += rot.rotation;
                        }
                        if (rot.beat < t_beat)
                            || (rot.beat == t_beat && rot.execution_time.is_early())
                        {
                            tail_lane_rotation += rot.rotation;
                        }
                    }
                    chain_masks.push((
                        chain.color,
                        chain.head_beat,
                        chain.head_line_index,
                        chain.head_line_layer,
                        chain.head_cut_direction,
                    ));
                    elements.push(DataElement::Chain(ChainData {
                        beat: beat.into(),
                        cut_direction: chain.head_cut_direction.into(),
                        x: chain.head_line_index.into(),
                        y: chain.head_line_layer.into(),
                        head_lane_rotation_deg: lane_rotation_deg.into(),
                        tail_beat: chain.tail_beat.into(),
                        tx: chain.tail_line_index.into(),
                        ty: chain.tail_line_layer.into(),
                        tail_lane_rotation_deg: tail_lane_rotation.into(),
                        squish_factor: chain.squish_factor.into(),
                        link_count: chain.slice_count.into(),
                        note_type,
                        custom_data: (&chain.custom_data).into(),
                    }));
                }
                for note in v3.color_notes.iter() {
                    if chain_masks.iter().any(|(color, beat, x, y, c)| {
                        *color == note.color
                            && *beat == note.beat
                            && *x == note.line_index
                            && *y == note.line_layer
                            && *c == note.cut_direction
                    }) {
                        continue;
                    }
                    let note_type = note.color.into();
                    let beat = note.beat;
                    let mut lane_rotation_deg = 0.;
                    for rot in rotations.iter() {
                        if rot.beat > beat {
                            break;
                        }
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early())
                        {
                            lane_rotation_deg += rot.rotation;
                        }
                    }
                    elements.push(DataElement::Note(NoteData {
                        beat: beat.into(),
                        cut_direction: note.cut_direction.into(),
                        x: note.line_index.into(),
                        y: note.line_layer.into(),
                        lane_rotation_deg: lane_rotation_deg.into(),
                        note_type,
                        angle_offset_deg: note.angle_offset.into(),
                        custom_data: (&note.custom_data).into()
                    }));
                }
                for bomb in v3.bomb_notes.iter() {
                    let beat = bomb.beat;
                    let mut lane_rotation_deg = 0.;
                    for rot in rotations.iter() {
                        if rot.beat > beat {
                            break;
                        }
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early())
                        {
                            lane_rotation_deg += rot.rotation;
                        }
                    }
                    elements.push(DataElement::Bomb(BombData {
                        beat: beat.into(),
                        x: bomb.line_index.into(),
                        y: bomb.line_layer.into(),
                        lane_rotation_deg: lane_rotation_deg.into(),
                        custom_data: (&bomb.custom_data).into(),
                    }));
                }
                for obst in v3.obstacles.iter() {
                    let beat = obst.beat;
                    let mut lane_rotation_deg = 0.;
                    for rot in rotations.iter() {
                        if rot.beat > beat {
                            break;
                        }
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early())
                        {
                            lane_rotation_deg += rot.rotation;
                        }
                    }
                    elements.push(DataElement::Obstacle(ObstacleData {
                        beat: beat.into(),
                        x: obst.line_index.into(),
                        y: obst.line_layer.into(),
                        duration: obst.duration.into(),
                        lane_rotation_deg: lane_rotation_deg.into(),
                        width: obst.width.into(),
                        height: obst.height.into(),
                        custom_data: (&obst.custom_data).into(),
                    }));
                }
            }
            BeatmapFile::V4(v4) => {
                let mut chain_masks = Vec::with_capacity(v4.chains.len());
                for chain in v4.chains.iter() {
                    let Some(head_data) = v4
                        .color_notes_data
                        .get(chain.head_note_metadata_index as usize)
                    else {
                        tracing::warn!(target: DB_DATA, "chain note references invalid color note data indes: {}", chain.head_note_metadata_index);
                        continue;
                    };
                    let Some(data) = v4.chains_data.get(chain.metadata_index as usize) else {
                        tracing::warn!(target: DB_DATA, "chain note references invalid chain note data index: {}", chain.metadata_index);
                        continue;
                    };
                    let note_type = head_data.color.into();
                    chain_masks.push((
                        head_data.color,
                        chain.head_beat,
                        head_data.line_index,
                        head_data.line_layer,
                        head_data.cut_direction,
                    ));
                    elements.push(DataElement::Chain(ChainData {
                        beat: chain.head_beat.into(),
                        cut_direction: head_data.cut_direction.into(),
                        x: head_data.line_index.into(),
                        y: head_data.line_layer.into(),
                        head_lane_rotation_deg: chain.head_rotation_lane.into(),
                        tail_beat: chain.tail_beat.into(),
                        tx: data.tail_line_index.into(),
                        ty: data.tail_line_layer.into(),
                        tail_lane_rotation_deg: chain.tail_rotation_lane.into(),
                        squish_factor: data.squish_factor.into(),
                        link_count: data.slice_count.into(),
                        note_type,
                        custom_data: Default::default(),
                    }));
                }
                for note in v4.color_notes.iter() {
                    let Some(data) = v4.color_notes_data.get(note.metadata_index as usize) else {
                        tracing::warn!(target: DB_DATA, "color note references invalid note data index: {}", note.metadata_index);
                        continue;
                    };

                    if chain_masks.iter().any(|(color, beat, x, y, c)| {
                        *color == data.color
                            && *beat == note.beat
                            && *x == data.line_index
                            && *y == data.line_layer
                            && *c == data.cut_direction
                    }) {
                        continue;
                    }
                    elements.push(DataElement::Note(NoteData {
                        beat: note.beat.into(),
                        cut_direction: data.cut_direction.into(),
                        x: data.line_index.into(),
                        y: data.line_layer.into(),
                        lane_rotation_deg: note.rotation_lane.into(),
                        note_type: data.color.into(),
                        angle_offset_deg: data.angle_offset.into(),
                        custom_data: Default::default(),
                    }));
                }
                for bomb in v4.bomb_notes.iter() {
                    let Some(data) = v4.bomb_notes_data.get(bomb.metadata_index as usize) else {
                        tracing::warn!(target: DB_DATA, "bomb note references invalid bomb data index: {}", bomb.metadata_index);
                        continue;
                    };
                    elements.push(DataElement::Bomb(BombData {
                        beat: bomb.beat.into(),
                        x: data.line_index.into(),
                        y: data.line_layer.into(),
                        lane_rotation_deg: bomb.rotation_lane.into(),
                        custom_data: Default::default(),
                    }));
                }
                for obst in v4.obstacles.iter() {
                    let Some(data) = v4.obstacles_data.get(obst.metadata_index as usize) else {
                        tracing::warn!(target: DB_DATA, "obstacle references invalid obstcale data index: {}", obst.metadata_index);
                        continue;
                    };
                    elements.push(DataElement::Obstacle(ObstacleData {
                        beat: obst.beat.into(),
                        x: data.line_index.into(),
                        y: data.line_layer.into(),
                        duration: data.duration.into(),
                        lane_rotation_deg: obst.rotation_lane.into(),
                        width: data.width.into(),
                        height: data.height.into(),
                        custom_data: Default::default(),
                    }));
                }
            }
        }

        // don't sort elements cuz it's lowkey pointless
        // since it can be sorted when exported or canonicalized


        Self {
            elements,
            values: Default::default(),
            templates: Default::default(),
            point_definitions,
            custom_events,
            runtime_data,
        }
    }

    fn random_quat(rng: &mut rand::rngs::ThreadRng) -> Quat {
        use rand::RngExt;
        Quat::from_euler(
            glam::EulerRot::YXZ,
            rng.random::<f32>() * 1.2 - 0.6,
            rng.random::<f32>() * 1.2 - 0.6,
            rng.random::<f32>() * 1.2 - 0.6,
        )
    }

    pub fn canonicalize(
        &self,
        rng: &mut rand::rngs::ThreadRng,
        mut values: HashMap<String, Value>,
    ) -> Result<BeatmapController<ObjectSource>, CanonicalizationError> {
        let mut runtime_data = self.runtime_data.clone();
        let mut color_notes = Vec::new();
        let mut bomb_notes = Vec::new();
        let mut obstacles = Vec::new();
        let mut chain_notes = Vec::new();
        let mut arcs: Vec<beatmap_core::beatmap::Arc<ObjectSource>> = Vec::new();

        for (k, v) in self.values.iter() {
            values.insert(k.clone(), v.clone());
        }

        
        let point_defs;
        let point_definitions = match &self.point_definitions {
            Some(p) => p,
            None => {
                point_defs = Default::default();
                &point_defs
            }
        };

        for (i, element) in self.elements.iter().enumerate() {
            match element {
                DataElement::Note(note_data) => {
                    let mut color = note_data
                        .note_type
                        .resolve(&values, Default::default())?;
                    if let Some(rgba) = note_data.custom_data.color.resolve(&values, Default::default())? {
                        color = match color {
                            NoteColor::Red | NoteColor::CustomRed(_) => NoteColor::CustomRed(rgba),
                            NoteColor::Blue | NoteColor::CustomBlue(_) => {
                                NoteColor::CustomBlue(rgba)
                            }
                        }
                    }
                    let index = color_notes.len() as u32;
                    color_notes.push(ColorNote {
                        spawn_orientation: Self::random_quat(rng),
                        beat: note_data.beat.resolve(&values, Default::default())?,
                        color,
                        cut_direction: note_data
                            .cut_direction
                            .resolve(&values, Default::default())?,
                        angle_offset_deg: note_data
                            .angle_offset_deg
                            .resolve(&values, Default::default())?,
                        grid_pos: Vec2::new(
                            note_data.x.resolve(&values, Default::default())?,
                            note_data.y.resolve(&values, Default::default())?,
                        ),
                        lane_rotation_deg: note_data
                            .lane_rotation_deg
                            .resolve(&values, Default::default())?,
                        dissolve: 0.,
                        index,
                        source: ObjectSource::Element { index: i },
                        custom_data: note_data.custom_data.resolve(&values, &mut runtime_data, point_definitions)?,
                    })
                }
                DataElement::Bomb(bomb_data) => {
                    let color = if let Some(rgba) =
                        bomb_data.custom_data.color.resolve(&values, Default::default())?
                    {
                        ObjectColor::Custom(rgba)
                    } else {
                        ObjectColor::default()
                    };
                    let index = bomb_notes.len() as u32;
                    bomb_notes.push(BombNote {
                        beat: bomb_data.beat.resolve(&values, Default::default())?,
                        color,
                        grid_pos: Vec2::new(
                            bomb_data.x.resolve(&values, Default::default())?,
                            bomb_data.y.resolve(&values, Default::default())?,
                        ),
                        lane_rotation_deg: bomb_data
                            .lane_rotation_deg
                            .resolve(&values, Default::default())?,
                        dissolve: 0.,
                        index,
                        source: ObjectSource::Element { index: i },
                        custom_data: bomb_data.custom_data.resolve(&values, &mut runtime_data, point_definitions)?,
                    })
                }
                DataElement::Obstacle(obstacle_data) => {
                    let color = if let Some(rgba) = obstacle_data
                        .custom_data.color
                        .resolve(&values, Default::default())?
                    {
                        ObjectColor::Custom(rgba)
                    } else {
                        ObjectColor::default()
                    };
                    let beat = obstacle_data
                        .beat
                        .resolve(&values, Default::default())?;
                    let njs = obstacle_data
                        .custom_data.custom_njs
                        .resolve(&values, Default::default())?.unwrap_or(runtime_data.njs);
                    let duration = obstacle_data
                        .duration
                        .resolve(&values, Default::default())?;
                    let width = obstacle_data
                        .width
                        .resolve(&values, Default::default())?;
                    let height = obstacle_data
                        .height
                        .resolve(&values, Default::default())?;
                    // let length = runtime_data.njs * (60. / runtime_data.bpm(TimeUnit::Beat(beat)));
                    let size = obstacle_data
                        .custom_data.size
                        .resolve(&values, Default::default())?;
                    let noodle_logic = size.z().is_some();
                    let size = Vec3::new(
                        size.x().unwrap_or(width),
                        size.y().unwrap_or(height),
                        size.z().unwrap_or(duration) * njs * (60. / runtime_data.bpm(TimeUnit::Beat(beat))),
                    );
                    let index = obstacles.len() as u32;
                    obstacles.push(Obstacle {
                        beat,
                        color,
                        grid_pos: Vec2::new(
                            obstacle_data.x.resolve(&values, Default::default())?,
                            obstacle_data.y.resolve(&values, Default::default())?,
                        ),
                        duration,
                        size,
                        lane_rotation_deg: obstacle_data
                            .lane_rotation_deg
                            .resolve(&values, Default::default())?,
                        dissolve: 0.,
                        index,
                        noodle_logic,
                        source: ObjectSource::Element { index: i },
                        custom_data: obstacle_data.custom_data.resolve(&values, &mut runtime_data, point_definitions)?,
                    })
                }
                DataElement::Chain(chain_data) => {
                    let mut color = chain_data
                        .note_type
                        .resolve(&values, Default::default())?;
                    if let Some(rgba) =
                        chain_data.custom_data.color.resolve(&values, Default::default())?
                    {
                        color = match color {
                            NoteColor::Red | NoteColor::CustomRed(_) => NoteColor::CustomRed(rgba),
                            NoteColor::Blue | NoteColor::CustomBlue(_) => {
                                NoteColor::CustomBlue(rgba)
                            }
                        }
                    }
                    let index = chain_notes.len() as u32;
                    let spawn_orientation = Self::random_quat(rng);
                    let mut links = Vec::new();
                    let slice_count = chain_data
                        .link_count
                        .resolve(&values, Default::default())?;
                    for i in 0..slice_count {
                        links.push(ChainNoteLinkData {
                            spawn_orientation: Self::random_quat(rng),
                            index: i as u32,
                        });
                    }
                    chain_notes.push(ChainNote {
                        spawn_orientation,
                        head_beat: chain_data.beat.resolve(&values, Default::default())?,
                        tail_beat: chain_data
                            .tail_beat
                            .resolve(&values, Default::default())?,
                        head_lane_rotation_deg: chain_data
                            .head_lane_rotation_deg
                            .resolve(&values, Default::default())?,
                        tail_lane_rotation_deg: chain_data
                            .tail_lane_rotation_deg
                            .resolve(&values, Default::default())?,
                        cut_direction: chain_data
                            .cut_direction
                            .resolve(&values, Default::default())?,
                        color,
                        head_grid_pos: Vec2::new(
                            chain_data.x.resolve(&values, Default::default())?,
                            chain_data.y.resolve(&values, Default::default())?,
                        ),
                        tail_grid_pos: Vec2::new(
                            chain_data.tx.resolve(&values, Default::default())?,
                            chain_data.ty.resolve(&values, Default::default())?,
                        ),
                        squish_factor: chain_data
                            .squish_factor
                            .resolve(&values, Default::default())?,
                        links,
                        dissolve: 0.,
                        index,
                        source: ObjectSource::Element { index: i },
                        custom_data: chain_data.custom_data.resolve(&values, &mut runtime_data, point_definitions)?,
                    })
                }
                DataElement::Arc(arc_data) => todo!(),
                DataElement::Template(template_placement) => todo!(),
                DataElement::ObstacleText(obstacle_text_data) => todo!(),
            }
        }

        color_notes.sort_by(|a, b| a.beat.total_cmp(&b.beat));
        bomb_notes.sort_by(|a, b| a.beat.total_cmp(&b.beat));
        obstacles.sort_by(|a, b| a.beat.total_cmp(&b.beat));
        chain_notes.sort_by(|a, b| a.head_beat.total_cmp(&b.head_beat));
        arcs.sort_by(|a, b| a.head_beat.total_cmp(&b.head_beat));

        Self::check_window_snaps(&mut color_notes);

        if let Some(events) = self.custom_events.as_ref() {
            for event in events.iter() {
                if let Ok(event) = EventData::deserialize(
                    event,
                    runtime_data.bpm,
                    &mut runtime_data.tracks_holder,
                    &mut runtime_data.base_provider_context,
                    point_definitions
                ) {
                    for event in event {
                        runtime_data.timeline_coroutine_manager.add_event(runtime_data.bpm, event);
                    }
                }
            }
        }

        Ok(BeatmapController {
            runtime_data,
            color_notes,
            bomb_notes,
            obstacles,
            chain_notes,
            arcs,
        })
    }

    fn check_window_snaps(color_notes: &mut [ColorNote<ObjectSource>]) {
        let mut note_types: HashMap<RawNoteColor, Vec<&mut ColorNote<ObjectSource>>> = HashMap::new();
        for n in color_notes.iter_mut() {
            let entry = note_types.entry(n.color.raw()).or_default();
            entry.push(n)
        }
        for (_, notes) in note_types.into_iter() {
            let mut time_groups: HashMap<u32, Vec<&mut ColorNote<ObjectSource>>> = HashMap::new();
            for n in notes.into_iter() {
                let entry = time_groups.entry(n.beat.to_bits()).or_default();
                entry.push(n);
            }

            for (_, mut notes) in time_groups.into_iter() {
                if notes.len() != 2 {
                    continue;
                }
                let b = notes.pop().unwrap();
                let a = notes.pop().unwrap();

                Self::check_window_snap(a, b);
                Self::check_window_snap(b, a);
            }
        }
    }

    fn check_window_snap(a: &mut ColorNote<ObjectSource>, b: &mut ColorNote<ObjectSource>) {
        fn normalize_angle(a: f32) -> f32 {
            (a + 360.) % 360.
        }
        fn degrees_between(a: f32, b: f32) -> f32 {
            let diff = (normalize_angle(a) - normalize_angle(b)).abs();
            f32::min(diff, 360. - diff)
        }

        let same_cuts = a.cut_direction == b.cut_direction;
        let this_is_dot = a.cut_direction == CutDirection::Dot;
        let other_is_dot = b.cut_direction == CutDirection::Dot;

        if !same_cuts && !this_is_dot && !other_is_dot {
            return;
        }

        let ap = a.grid_pos;
        let bp = b.grid_pos;
        let cross = bp - ap;
        let angle = cross.to_angle().to_degrees();

        if this_is_dot && other_is_dot {
            a.angle_offset_deg = angle;
            b.angle_offset_deg = angle;
            return;
        }

        let degrees = if this_is_dot {
            b.cut_direction.angle_degrees()
        } else {
            a.cut_direction.angle_degrees()
        };
        let between = degrees_between(degrees, angle);

        if between <= 40. {
            a.angle_offset_deg = angle - a.cut_direction.angle_degrees();
            b.angle_offset_deg = angle - b.cut_direction.angle_degrees();
        }
    }
    
}
