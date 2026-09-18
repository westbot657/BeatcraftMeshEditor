use std::collections::HashMap;
use std::f32;

use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use rand::{RngExt, rngs::ThreadRng};

use crate::data::map_editing::ObjectSource;
use crate::render::GameObjectInstanceData;

use super::BeatmapProjectDiff;
use bs_mapping_data::{
    BeatmapDataError, BeatmapFile, BpmRegion,
    CutDirection, InfoFile, v2,
};

use beatmap_core::*;

pub trait GameObjectExt: GameObject<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
    ) -> GameObjectInstanceData;
    fn get_editor_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
        _beat_distance: f32,
    ) -> GameObjectInstanceData {
        self.get_instance(clipping_plane, model, cs)
    }
}

impl GameObjectExt for ColorNote<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::color_note(
            clipping_plane,
            model,
            self.color.color(cs),
            self.dissolve,
            self.index,
            Vec4::ZERO,
        )
    }
}

impl GameObjectExt for BombNote<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::bomb_note(
            clipping_plane,
            model,
            self.color.color(cs),
            self.dissolve,
            self.index,
            Vec4::ZERO,
        )
    }
}

impl GameObjectExt for Obstacle<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::obstacle(
            clipping_plane,
            model * Mat4::from_translation(Vec3::new(0., 0., -0.25)),
            self.color.color(cs),
            self.dissolve,
            self.index,
            self.size,
        )
    }
    fn get_editor_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
        beat_distance: f32,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::obstacle(
            clipping_plane,
            model,
            self.color.color(cs),
            self.dissolve,
            self.index,
            Vec3::new(self.size.x, self.size.y, self.duration * beat_distance),
        )
    }
}

impl GameObjectExt for ChainNote<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::color_note(
            clipping_plane,
            model,
            self.color.color(cs),
            self.dissolve,
            self.index,
            Vec4::ZERO,
        )
    }
}

impl GameObjectExt for ChainNoteLink<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::color_note(
            clipping_plane,
            model,
            self.color.color(cs),
            self.dissolve,
            self.index,
            Vec4::ZERO,
        )
    }
}

impl GameObjectExt for Arc<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::arc(clipping_plane, model, self.color.color(cs))
    }
}

pub trait BeatmapControllerExt: Sized {
    fn new(
        info: &InfoFile,
        diff_data: &BeatmapProjectDiff,
        diff: &BeatmapFile,
        bpm_regions: Vec<BpmRegion>,
        sample_count: usize,
        sample_rate: u32,
    ) -> Result<Self, BeatmapDataError>;
}

impl BeatmapControllerExt for BeatmapController<ObjectSource> {
    fn new(
        info: &InfoFile,
        diff_data: &BeatmapProjectDiff,
        diff: &BeatmapFile,
        bpm_regions: Vec<BpmRegion>,
        sample_count: usize,
        sample_rate: u32,
    ) -> Result<Self, BeatmapDataError> {
        diff.to_controller(info, diff_data, bpm_regions, sample_count, sample_rate)
    }
}

trait BeatmapFileExt {
    #[deprecated = "switch to editor system"]
    fn to_controller(
        &self,
        info: &InfoFile,
        diff_data: &BeatmapProjectDiff,
        bpm_regions: Vec<BpmRegion>,
        sample_count: usize,
        sample_rate: u32,
    ) -> Result<BeatmapController<ObjectSource>, BeatmapDataError>;
    fn check_window_snaps(color_notes: &mut [ColorNote<ObjectSource>]);
    fn check_window_snap(a: &mut ColorNote<ObjectSource>, b: &mut ColorNote<ObjectSource>);
}

pub fn get_random_spawn_quat(rng: &mut ThreadRng) -> Quat {
    let get_c = |rng: &mut ThreadRng| -> f32 { rng.random::<f32>() * 0.6f32 * 2. - 0.6 };
    Quat::from_euler(glam::EulerRot::ZYX, get_c(rng), get_c(rng), get_c(rng))
}

impl BeatmapFileExt for BeatmapFile {
    fn to_controller(
        &self,
        info: &InfoFile,
        diff_data: &BeatmapProjectDiff,
        bpm_regions: Vec<BpmRegion>,
        sample_count: usize,
        sample_rate: u32,
    ) -> Result<BeatmapController<ObjectSource>, BeatmapDataError> {
        let runtime_data = RuntimeData::new(
            diff_data.njs,
            info.bpm(),
            diff_data.njs_offset,
            bpm_regions,
            sample_count,
            sample_rate,
        );

        let mut rng = rand::rng();
        let mut color_notes = Vec::new();
        let mut bomb_notes = Vec::new();
        let mut obstacles = Vec::new();
        let mut chain_notes = Vec::new();
        let arcs = Vec::new();

        match self {
            Self::V2(v2) => {
                let mut rotations = Vec::new();
                for event in v2.events.iter() {
                    if let v2::V2Event::SpawnRotation(rot) = event {
                        rotations.push(*rot);
                    }
                }
                rotations.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap());
                for note in v2.notes.iter() {
                    match note {
                        v2::V2Note::Note(color_note) => {
                            let index = color_notes.len() as u32;
                            let color: NoteColor = color_note.typ.into();
                            let beat = color_note.time;
                            let mut lane_rotation_deg = 0i32;
                            'rot: for rot in rotations.iter() {
                                if (rot.beat < beat)
                                    || (rot.beat == beat && rot.execution_time.is_early())
                                {
                                    lane_rotation_deg += rot.rotation_angle.get_degrees();
                                } else {
                                    break 'rot;
                                }
                            }

                            color_notes.push(ColorNote {
                                spawn_orientation: get_random_spawn_quat(&mut rng),
                                beat,
                                color,
                                cut_direction: color_note.cut_direction,
                                angle_offset_deg: 0.,
                                grid_pos: Vec2::new(color_note.line_index, color_note.line_layer),
                                lane_rotation_deg: lane_rotation_deg as f32,
                                dissolve: 0.,
                                index,
                                source: ObjectSource::Json { index },
                            })
                        }
                        v2::V2Note::Bomb(bomb_note) => {
                            let index = bomb_notes.len() as u32;
                            let beat = bomb_note.beat;
                            let mut lane_rotation_deg = 0;
                            'rot: for rot in rotations.iter() {
                                if (rot.beat < beat)
                                    || (rot.beat == beat && rot.execution_time.is_early())
                                {
                                    lane_rotation_deg += rot.rotation_angle.get_degrees();
                                } else {
                                    break 'rot;
                                }
                            }
                            bomb_notes.push(BombNote {
                                beat,
                                color: ObjectColor::default(),
                                grid_pos: Vec2::new(bomb_note.line_index, bomb_note.line_layer),
                                lane_rotation_deg: lane_rotation_deg as f32,
                                dissolve: 0.,
                                index,
                                source: ObjectSource::Json { index },
                            })
                        }
                    }
                }
                for obst in v2.obstacles.iter() {
                    let index = obstacles.len() as u32;
                    let local_bpm = runtime_data.bpm(TimeUnit::Beat(obst.beat));
                    let dist_beats_to_meters = runtime_data.njs * (60. / local_bpm);
                    let (grid_pos, size) = match obst.typ {
                        bs_mapping_data::v2::ObstacleV2Type::FullHeight => (
                            Vec2::new(obst.line_index, obst.line_layer),
                            Vec3::new(obst.width, 5., obst.duration * dist_beats_to_meters),
                        ),
                        bs_mapping_data::v2::ObstacleV2Type::Crouch => (
                            Vec2::new(obst.line_index, obst.line_layer + 2.),
                            Vec3::new(obst.width, 3., obst.duration * dist_beats_to_meters),
                        ),
                        bs_mapping_data::v2::ObstacleV2Type::Free => (
                            Vec2::new(obst.line_index, obst.line_layer),
                            Vec3::new(
                                obst.width,
                                obst.height,
                                obst.duration * dist_beats_to_meters,
                            ),
                        ),
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
                    obstacles.push(Obstacle {
                        beat,
                        color: ObjectColor::default(),
                        grid_pos,
                        lane_rotation_deg: lane_rotation_deg as f32,
                        duration: obst.duration,
                        size,
                        dissolve: 0.,
                        index,
                        noodle_logic: false,
                        source: ObjectSource::Json { index },
                    })
                }
            }
            Self::V3(v3) => {
                let mut rotations = Vec::new();
                for event in v3.rotation_events.iter() {
                    rotations.push(*event);
                }
                rotations.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap());

                for chain in v3.chains.iter() {
                    let index = chain_notes.len() as u32;
                    let mut links = Vec::with_capacity(chain.slice_count as usize);
                    let beat = chain.head_beat;
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

                    for i in 0..chain.slice_count {
                        links.push(ChainNoteLinkData {
                            spawn_orientation: get_random_spawn_quat(&mut rng),
                            index: i as u32,
                        });
                    }
                    chain_notes.push(ChainNote {
                        spawn_orientation: get_random_spawn_quat(&mut rng),
                        head_beat: beat,
                        tail_beat: chain.tail_beat,
                        head_lane_rotation_deg: lane_rotation_deg,
                        tail_lane_rotation_deg: lane_rotation_deg,
                        cut_direction: chain.head_cut_direction,
                        color: chain.color.into(),
                        head_grid_pos: Vec2::new(chain.head_line_index, chain.head_line_layer),
                        tail_grid_pos: Vec2::new(chain.tail_line_index, chain.tail_line_layer),
                        squish_factor: chain.squish_factor,
                        links,
                        dissolve: 0.,
                        index,
                        source: ObjectSource::Json { index },
                    });
                }
                for note in v3.color_notes.iter() {
                    let index = color_notes.len() as u32;
                    let color: NoteColor = note.color.into();
                    let grid_pos = Vec2::new(note.line_index, note.line_layer);

                    if chain_notes.iter().any(|c| {
                        c.color == color && c.head_grid_pos == grid_pos && c.head_beat == note.beat
                    }) {
                        continue;
                    }

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

                    color_notes.push(ColorNote {
                        spawn_orientation: get_random_spawn_quat(&mut rng),
                        beat,
                        color,
                        cut_direction: note.cut_direction,
                        angle_offset_deg: 0.,
                        grid_pos,
                        lane_rotation_deg,
                        dissolve: 0.,
                        index,
                        source: ObjectSource::Json { index },
                    });
                }
                for bomb in v3.bomb_notes.iter() {
                    let index = bomb_notes.len() as u32;
                    let mut lane_rotation_deg = 0.;
                    let beat = bomb.beat;
                    for rot in rotations.iter() {
                        if rot.beat > beat {
                            break;
                        }
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early())
                        {
                            lane_rotation_deg += rot.rotation;
                        }
                    }
                    bomb_notes.push(BombNote {
                        beat,
                        color: ObjectColor::default(),
                        grid_pos: Vec2::new(bomb.line_index, bomb.line_layer),
                        lane_rotation_deg,
                        dissolve: 0.,
                        index,
                        source: ObjectSource::Json { index },
                    });
                }
                for obst in v3.obstacles.iter() {
                    let index = obstacles.len() as u32;
                    let mut lane_rotation_deg = 0.;
                    let beat = obst.beat;
                    for rot in rotations.iter() {
                        if rot.beat > beat {
                            break;
                        }
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early())
                        {
                            lane_rotation_deg += rot.rotation;
                        }
                    }
                    let local_bpm = runtime_data.bpm(TimeUnit::Beat(beat));
                    let dist_beats_to_meters = runtime_data.njs * (60. / local_bpm);
                    obstacles.push(Obstacle {
                        beat,
                        color: ObjectColor::default(),
                        grid_pos: Vec2::new(obst.line_index, obst.line_layer),
                        size: Vec3::new(
                            obst.width,
                            obst.height,
                            obst.duration * dist_beats_to_meters,
                        ),
                        duration: obst.duration,
                        lane_rotation_deg,
                        dissolve: 0.,
                        index,
                        noodle_logic: false,
                        source: ObjectSource::Json { index },
                    });
                }
            }
            Self::V4(v4) => {
                for chain in v4.chains.iter() {
                    let index = chain_notes.len() as u32;
                    let Some(head_data) = v4
                        .color_notes_data
                        .get(chain.head_note_metadata_index as usize)
                    else {
                        continue;
                    };
                    let Some(data) = v4.chains_data.get(chain.metadata_index as usize) else {
                        continue;
                    };
                    let mut links = Vec::new();
                    for i in 0..data.slice_count {
                        links.push(ChainNoteLinkData {
                            spawn_orientation: get_random_spawn_quat(&mut rng),
                            index: i as u32,
                        });
                    }

                    chain_notes.push(ChainNote {
                        spawn_orientation: get_random_spawn_quat(&mut rng),
                        head_beat: chain.head_beat,
                        tail_beat: chain.tail_beat,
                        head_lane_rotation_deg: chain.head_rotation_lane as f32,
                        tail_lane_rotation_deg: chain.tail_rotation_lane as f32,
                        cut_direction: head_data.cut_direction,
                        color: head_data.color.into(),
                        head_grid_pos: Vec2::new(head_data.line_index, head_data.line_layer),
                        tail_grid_pos: Vec2::new(data.tail_line_index, data.tail_line_layer),
                        squish_factor: data.squish_factor,
                        links,
                        dissolve: 0.,
                        index,
                        source: ObjectSource::Json { index },
                    });
                }
                for note in v4.color_notes.iter() {
                    let index = color_notes.len() as u32;
                    let Some(data) = v4.color_notes_data.get(note.metadata_index as usize) else {
                        continue;
                    };
                    let grid_pos = Vec2::new(data.line_index, data.line_layer);
                    let color: NoteColor = data.color.into();

                    if chain_notes.iter().any(|c| {
                        c.color == color && c.head_grid_pos == grid_pos && c.head_beat == note.beat
                    }) {
                        continue;
                    }
                    color_notes.push(ColorNote {
                        spawn_orientation: get_random_spawn_quat(&mut rng),
                        beat: note.beat,
                        color,
                        cut_direction: data.cut_direction,
                        angle_offset_deg: data.angle_offset as f32,
                        grid_pos,
                        lane_rotation_deg: note.rotation_lane as f32,
                        dissolve: 0.,
                        index,
                        source: ObjectSource::Json { index },
                    });
                }
                for bomb in v4.bomb_notes.iter() {
                    let index = bomb_notes.len() as u32;
                    let Some(data) = v4.bomb_notes_data.get(bomb.metadata_index as usize) else {
                        continue;
                    };
                    bomb_notes.push(BombNote {
                        beat: bomb.beat,
                        color: ObjectColor::default(),
                        grid_pos: Vec2::new(data.line_index, data.line_layer),
                        lane_rotation_deg: bomb.rotation_lane as f32,
                        dissolve: 0.,
                        index,
                        source: ObjectSource::Json { index },
                    });
                }
                for obst in v4.obstacles.iter() {
                    let index = obstacles.len() as u32;
                    let Some(data) = v4.obstacles_data.get(obst.metadata_index as usize) else {
                        continue;
                    };
                    let local_bpm = runtime_data.bpm(TimeUnit::Beat(obst.beat));
                    let dist_beats_to_meters = runtime_data.njs * (60. / local_bpm);
                    obstacles.push(Obstacle {
                        beat: obst.beat,
                        color: ObjectColor::default(),
                        grid_pos: Vec2::new(data.line_index, data.line_layer),
                        size: Vec3::new(
                            data.width,
                            data.height,
                            data.duration * dist_beats_to_meters,
                        ),
                        duration: data.duration,
                        lane_rotation_deg: obst.rotation_lane as f32,
                        dissolve: 0.,
                        index,
                        noodle_logic: false,
                        source: ObjectSource::Json { index },
                    });
                }
            }
        }

        Self::check_window_snaps(&mut color_notes);

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
        let angle = -cross.to_angle().to_degrees() + 90.;

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
