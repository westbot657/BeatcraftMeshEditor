use std::collections::HashMap;

use bs_mapping_data::{Color, CutDirection, MapVersion};
use bs_mapping_data::v2::{ArcV2, BeatmapFileV2, BombNoteV2, ColorNoteV2, ObstacleV2, ObstacleV2Type, V2Note};

use crate::beatmap::object::NoteColor;
use crate::data::map_editing::{DataElement, EditingData, Value};

use super::{ExportResult, Exportable};


impl Exportable for BeatmapFileV2 {
    fn export(data: &EditingData, mods: &[super::Mod], mut values: HashMap<String, Value>) -> super::ExportResult<Self> {
        let span = tracing::debug_span!("export-map-v2");
        let _guard = span.enter();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let mut notes = Vec::new();
        let mut obstacles = Vec::new();
        let mut arcs = Vec::new();

        let mut events = Vec::new();

        for (k, v) in data.values.iter() {
            values.insert(k.clone(), v.clone());
        }

        for element in data.elements.iter() {
            match (|| {
                match element {
                    DataElement::Note(note_data) => {
                        let span = tracing::debug_span!("note");
                        let _guard = span.enter();
                        let time = note_data.beat.resolve(&values, Default::default())?;
                        // TODO: time granularity warning
                        let line_index = note_data.x.resolve(&values, Default::default())?;
                        let line_layer = note_data.y.resolve(&values, Default::default())?;
                        // TODO: precision check
                        // TODO: bounds check
                        let color = note_data.note_type.resolve(&values, Default::default())?;
                        let (typ, color) = match color {
                            NoteColor::Red => (Color::Red, None),
                            NoteColor::Blue => (Color::Blue, None),
                            NoteColor::CustomRed(vec4) => (Color::Red, Some(vec4)),
                            NoteColor::CustomBlue(vec4) => (Color::Blue, Some(vec4)),
                        };
                        // TODO: chroma check
                        let cut_direction = note_data.cut_direction.resolve(&values, Default::default())?;
                        // TODO: angle offset check
                        notes.push(V2Note::Note(
                            ColorNoteV2 {
                                time,
                                line_index,
                                line_layer,
                                typ,
                                cut_direction,
                                custom_data: None, // TODO
                            }
                        ))
                    },
                    DataElement::Bomb(bomb_data) => {
                        let span = tracing::debug_span!("bomb");
                        let _guard = span.enter();
                        let beat = bomb_data.beat.resolve(&values, Default::default())?;
                        // TODO: time granularity warning
                        let line_index = bomb_data.x.resolve(&values, Default::default())?;
                        let line_layer = bomb_data.y.resolve(&values, Default::default())?;
                        // TODO: precision check
                        // TODO: bounds check
                        notes.push(V2Note::Bomb(
                            BombNoteV2 {
                                beat,
                                line_index,
                                line_layer,
                                _type: bs_mapping_data::Sentinel,
                                cut_direction: CutDirection::Up,
                                custom_data: None, // TODO
                            }
                        ));
                    },
                    DataElement::Obstacle(obstacle_data) => {
                        let span = tracing::debug_span!("obstacle");
                        let _guard = span.enter();
                        let beat = obstacle_data.beat.resolve(&values, Default::default())?;
                        // TODO: time granularity warning
                        let duration = obstacle_data.duration.resolve(&values, Default::default())?;
                        if duration < 0. {
                            // TODO: error
                        }
                        let line_index = obstacle_data.x.resolve(&values, Default::default())?;
                        let line_layer = obstacle_data.y.resolve(&values, Default::default())?;
                        // TODO: precision check
                        // TODO: bounds check
                        let width = obstacle_data.width.resolve(&values, Default::default())?;
                        let height = obstacle_data.height.resolve(&values, Default::default())?;
                        // TODO: size check
                        obstacles.push(
                            ObstacleV2 {
                                typ: ObstacleV2Type::Free,
                                beat,
                                duration,
                                line_index,
                                line_layer,
                                width,
                                height,
                                custom_data: None,
                            }
                        );
                    },
                    DataElement::Chain(chain_data) => {
                        let span = tracing::debug_span!("chain");
                        let _guard = span.enter();
                        // TODO: warning + export normal note
                    },
                    DataElement::Arc(arc_data) => {
                        let span = tracing::debug_span!("arc");
                        let _guard = span.enter();
                        let color = arc_data.note_type.resolve(&values, Default::default())?;
                        let (color, custom_color) = match color {
                            NoteColor::Red => (Color::Red, None),
                            NoteColor::Blue => (Color::Blue, None),
                            NoteColor::CustomRed(vec4) => (Color::Red, Some(vec4)),
                            NoteColor::CustomBlue(vec4) => (Color::Blue, Some(vec4)),
                        };
                        let head_beat = arc_data.beat.resolve(&values, Default::default())?;
                        let head_line_index = arc_data.x.resolve(&values, Default::default())?;
                        let head_line_layer = arc_data.y.resolve(&values, Default::default())?;
                        let head_cut_direction = arc_data.cut_direction.resolve(&values, Default::default())?;
                        let head_ctrl_magnitude = arc_data.head_ctrl_magnitude.resolve(&values, Default::default())?;
                        let tail_beat = arc_data.tail_beat.resolve(&values, Default::default())?;
                        let tail_line_index = arc_data.tx.resolve(&values, Default::default())?;
                        let tail_line_layer = arc_data.ty.resolve(&values, Default::default())?;
                        let tail_cut_direction = arc_data.tail_cut_direction.resolve(&values, Default::default())?;
                        let tail_ctrl_magnitude = arc_data.tail_ctrl_magnitude.resolve(&values, Default::default())?;
                        let mid_anchor_mode = arc_data.mid_anchor_mode.resolve(&values, Default::default())?;

                        arcs.push(
                            ArcV2 {
                                color,
                                head_beat,
                                head_line_index,
                                head_line_layer,
                                head_cut_direction,
                                head_ctrl_magnitude,
                                tail_beat,
                                tail_line_index,
                                tail_line_layer,
                                tail_cut_direction,
                                tail_ctrl_magnitude,
                                mid_anchor_mode,
                                custom_data: None, // TODO
                            }
                        );
                    },
                    DataElement::Template(template_placement) => {
                        let span = tracing::debug_span!("template");
                        let _guard = span.enter();

                    },
                    DataElement::ObstacleText(obstacle_text_data) => {
                        let span = tracing::debug_span!("obstacle-text");
                        let _guard = span.enter();

                    },
                };
                Ok(())
            })() {
                Ok(_) => {},
                Err(e) => {
                    errors.push(e);
                }
            }
        }


        if errors.is_empty() {
            ExportResult::Ok {
                output: Self {
                    stats: None,
                    bpm_changes: None,
                    version: MapVersion::V2_6_0,
                    notes,
                    obstacles,
                    arcs,
                    events,
                    custom_data: None,
                    bookmarks: None,
                    waypoints: None,
                    special_events: None,
                },
                warnings
            }
        } else {
            ExportResult::Err {
                errors,
                warnings
            }
        }
    }
}


