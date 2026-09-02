use crate::data::mesh::{NormalId, UvId, VertexId};
use crate::editor::{Selection, WorkingRenameKey};
use crate::light_mesh::{self, BloomfogStyle, ComputeNormal, ComputeVertex};
use crate::render::{GridType, InstanceData, LIGHT_COLORS, MeshDrawCall, PointDrawCall};
use crate::widgets::TextInput;
use crate::{App, D_ARROW, MODIFIER_NAMES, R_ARROW, RefDuper, SMALL_X, UnsafeMutRef, data, editor};
use crate::ui_elements::*;
use bs_mapping_data::easing::Easing;
use eframe::glow;
use egui::Ui;
use glam::{Mat4, Vec2, Vec3, Vec4};

pub fn draw_edit_left(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
    let rd = RefDuper;
    let self2 = unsafe { rd.detach_mut_ref(s) };
    let self3 = unsafe { rd.detach_mut_ref(s) };
    let self4 = unsafe { rd.detach_mut_ref(s) };
    if let Some(part) = self2.get_current_part_mut() {
        let rd2 = RefDuper;
        let part2 = unsafe { rd2.detach_mut_ref(part) };
        let w = ui.available_width();
        let w2 = (w - ui.spacing().item_spacing.x) / 2.;
        let w3 = (w - ui.spacing().item_spacing.x * 2.) / 3.;
        //let w4 = w3*2.+ui.spacing().item_spacing.x;

        // Indexed vertices
        let verts = &mut s.state.ui.edit_collpased.i_vertices;
        ui.horizontal(|ui| {
            let icon = if *verts { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                *verts = !*verts;
            }
            ui.label(self3.data.locale.get("indexed-vertices"));
        });

        if !*verts {
            for vert in part.vertices.indexed.iter_mut() {
                vec3_row(
                    ui,
                    vert,
                    w3,
                    || part2.clone(),
                    |t| {
                        self3.add_history(editor::HistoryEntry::MeshPart(
                            light_mesh::LightMeshPartSnapshot {
                                id: self3.get_current_mesh_id().unwrap().to_string(),
                                name: self3.get_current_part_name().unwrap().to_string(),
                                part: Box::new(t),
                            },
                        ))
                    },
                    || self4.rebuild_meshes(gl),
                );
            }
        }

        // Named vertices
        let verts = &mut s.state.ui.edit_collpased.n_vertices;
        ui.horizontal(|ui| {
            let icon = if *verts { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                *verts = !*verts;
            }
            ui.label(self3.data.locale.get("named-vertices"));
        });

        if !*verts {
            for (i, (key, vert)) in part.vertices.named.iter_mut().enumerate() {
                if i != 0 {
                    ui.separator();
                }
                let mut name = key.clone();
                if let WorkingRenameKey::NamedVert(ref name2) = self3.state.ui.working_key
                    && *name2 == name
                {
                    name = self3.state.ui.working_name.take().unwrap_or(name);
                }

                if ui
                    .add_sized([w, 20.], egui::TextEdit::singleline(&mut name))
                    .changed()
                {
                    let _ = self3.rename(editor::Rename::Vertex {
                        part: editor::PartId {
                            view_id: self3.get_current_mesh_id().unwrap().to_string(),
                            name: self3.get_current_part_name().unwrap().to_string(),
                        },
                        swap: editor::DataSwap {
                            from: data::mesh::VertexId::Named(key.clone()),
                            to: data::mesh::VertexId::Named(name),
                        },
                    });
                    self3.state.ui.working_key = WorkingRenameKey::None;
                    return;
                }

                if name != *key {
                    self3.state.ui.working_key = WorkingRenameKey::NamedVert(key.clone());
                    self3.state.ui.working_name = Some(name);
                }

                vec3_row(
                    ui,
                    vert,
                    w3,
                    || part2.clone(),
                    |t| {
                        self3.add_history(editor::HistoryEntry::MeshPart(
                            light_mesh::LightMeshPartSnapshot {
                                id: self3.get_current_mesh_id().unwrap().to_string(),
                                name: self3.get_current_part_name().unwrap().to_string(),
                                part: Box::new(t),
                            },
                        ))
                    },
                    || self4.rebuild_meshes(gl),
                );
            }
        }

        // Compute vertices
        let verts = &mut s.state.ui.edit_collpased.c_vertices;
        ui.horizontal(|ui| {
            let icon = if *verts { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                *verts = !*verts;
            }
            ui.label(self3.data.locale.get("compute-vertices"));
        });

        if !*verts {
            for (i, (key, comp)) in part.vertices.compute.iter_mut().enumerate() {
                if i != 0 {
                    ui.separator();
                }
                let mut name = key.clone();
                if let WorkingRenameKey::CompVert(ref name2) = self3.state.ui.working_key
                    && *name2 == name
                {
                    name = self3.state.ui.working_name.take().unwrap_or(name);
                }

                if ui
                    .add_sized([w, 20.], egui::TextEdit::singleline(&mut name))
                    .changed()
                {
                    let _ = self3.rename(editor::Rename::Vertex {
                        part: editor::PartId {
                            view_id: self3.get_current_mesh_id().unwrap().to_string(),
                            name: self3.get_current_part_name().unwrap().to_string(),
                        },
                        swap: editor::DataSwap {
                            from: data::mesh::VertexId::Named(key.clone()),
                            to: data::mesh::VertexId::Named(name),
                        },
                    });
                    self3.state.ui.working_key = WorkingRenameKey::None;
                    return;
                }

                if name != *key {
                    self3.state.ui.working_key = WorkingRenameKey::CompVert(key.clone());
                    self3.state.ui.working_name = Some(name);
                }

                compute_vertex_row(ui, (w2, w3), comp, key, part2, self3, gl);
            }
        }

        // Indexed uvs
        let uvs = &mut s.state.ui.edit_collpased.i_uvs;
        ui.horizontal(|ui| {
            let icon = if *uvs { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                *uvs = !*uvs;
            }
            ui.label(self3.data.locale.get("indexed-uvs"));
        });

        if !*uvs {
            for (i, uv) in part.uvs.indexed.iter_mut().enumerate() {
                if ui
                    .horizontal(|ui| {
                        ui.label(format!("{i}"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button(SMALL_X).clicked() {
                                self3.add_history(editor::HistoryEntry::MeshPart(
                                    light_mesh::LightMeshPartSnapshot {
                                        id: self3.get_current_mesh_id().unwrap().to_string(),
                                        name: self3.get_current_part_name().unwrap().to_string(),
                                        part: Box::new(part2.clone()),
                                    },
                                ));
                                part2.delete_uvs([UvId::Index(i)]);
                                self3.rebuild_meshes(gl);
                                return true;
                            }
                            false
                        })
                        .inner
                    })
                    .inner
                {
                    return;
                };
                vec2_row(
                    ui,
                    uv,
                    w2,
                    || part2.clone(),
                    |t| {
                        self3.add_history(editor::HistoryEntry::MeshPart(
                            light_mesh::LightMeshPartSnapshot {
                                id: self3.get_current_mesh_id().unwrap().to_string(),
                                name: self3.get_current_part_name().unwrap().to_string(),
                                part: Box::new(t),
                            },
                        ))
                    },
                    || self4.rebuild_meshes(gl),
                );
            }
            ui.separator();
            if ui.button(self3.data.locale.get("add-indexed-uv")).clicked() {
                self3.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        id: self3.get_current_mesh_id().unwrap().to_string(),
                        name: self3.get_current_part_name().unwrap().to_string(),
                        part: Box::new(part.clone()),
                    },
                ));
                part.uvs.indexed.push(Vec2::ZERO);
            }
        }

        // Named uvs
        let uvs = &mut s.state.ui.edit_collpased.n_uvs;
        ui.horizontal(|ui| {
            let icon = if *uvs { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                *uvs = !*uvs;
            }
            ui.label(self3.data.locale.get("named-uvs"));
        });

        if !*uvs {
            for (i, (key, uv)) in part.uvs.named.iter_mut().enumerate() {
                if i != 0 {
                    ui.separator();
                }
                let mut name = key.clone();
                if let WorkingRenameKey::NamedUv(ref name2) = self3.state.ui.working_key
                    && *name2 == name
                {
                    name = self3.state.ui.working_name.take().unwrap_or(name);
                }

                if ui
                    .horizontal(|ui| {
                        if ui
                            .add_sized([w - 24., 20.], egui::TextEdit::singleline(&mut name))
                            .changed()
                        {
                            let _ = self3.rename(editor::Rename::Uv {
                                part: editor::PartId {
                                    view_id: self3.get_current_mesh_id().unwrap().to_string(),
                                    name: self3.get_current_part_name().unwrap().to_string(),
                                },
                                swap: editor::DataSwap {
                                    from: UvId::Named(key.clone()),
                                    to: UvId::Named(name.clone()),
                                },
                            });
                            self3.state.ui.working_key = WorkingRenameKey::None;
                            return true;
                        }
                        if ui.small_button(SMALL_X).clicked() {
                            self3.add_history(editor::HistoryEntry::MeshPart(
                                light_mesh::LightMeshPartSnapshot {
                                    id: self3.get_current_mesh_id().unwrap().to_string(),
                                    name: self3.get_current_part_name().unwrap().to_string(),
                                    part: Box::new(part2.clone()),
                                },
                            ));
                            part2.delete_uvs([UvId::Named(key.clone())]);
                            self3.rebuild_meshes(gl);
                            return true;
                        }
                        false
                    })
                    .inner
                {
                    return;
                }

                if name != *key {
                    self3.state.ui.working_key = WorkingRenameKey::NamedUv(key.clone());
                    self3.state.ui.working_name = Some(name);
                }

                vec2_row(
                    ui,
                    uv,
                    w2,
                    || part2.clone(),
                    |t| {
                        self3.add_history(editor::HistoryEntry::MeshPart(
                            light_mesh::LightMeshPartSnapshot {
                                id: self3.get_current_mesh_id().unwrap().to_string(),
                                name: self3.get_current_part_name().unwrap().to_string(),
                                part: Box::new(t),
                            },
                        ))
                    },
                    || self4.rebuild_meshes(gl),
                );
            }
            ui.separator();
            let mut new_uv = None;
            ui.add_sized(
                [w, 20.],
                TextInput::new(self3.data.locale.get("add-named-uv"), &mut new_uv),
            );
            if let Some(name) = new_uv {
                self3.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        id: self3.get_current_mesh_id().unwrap().to_string(),
                        name: self3.get_current_part_name().unwrap().to_string(),
                        part: Box::new(part.clone()),
                    },
                ));
                part.uvs.named.insert(name, Vec2::ZERO);
            }
        }

        // Indexed normals
        let norms = &mut s.state.ui.edit_collpased.i_normals;
        ui.horizontal(|ui| {
            let icon = if *norms { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                *norms = !*norms;
            }
            ui.label(self3.data.locale.get("indexed-normals"));
        });

        if !*norms {
            for (i, norm) in part.normals.indexed.iter_mut().enumerate() {
                if ui
                    .horizontal(|ui| {
                        ui.label(format!("{i}"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button(SMALL_X).clicked() {
                                self3.add_history(editor::HistoryEntry::MeshPart(
                                    light_mesh::LightMeshPartSnapshot {
                                        id: self3.get_current_mesh_id().unwrap().to_string(),
                                        name: self3.get_current_part_name().unwrap().to_string(),
                                        part: Box::new(part2.clone()),
                                    },
                                ));
                                part2.delete_normals([NormalId::Index(i)]);
                                self3.rebuild_meshes(gl);
                                return true;
                            }
                            false
                        })
                        .inner
                    })
                    .inner
                {
                    return;
                };
                vec3_row(
                    ui,
                    norm,
                    w3,
                    || part2.clone(),
                    |t| {
                        self3.add_history(editor::HistoryEntry::MeshPart(
                            light_mesh::LightMeshPartSnapshot {
                                id: self3.get_current_mesh_id().unwrap().to_string(),
                                name: self3.get_current_part_name().unwrap().to_string(),
                                part: Box::new(t),
                            },
                        ))
                    },
                    || self4.rebuild_meshes(gl),
                );
            }
            ui.separator();
            if ui
                .button(self3.data.locale.get("add-indexed-normal"))
                .clicked()
            {
                self3.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        id: self3.get_current_mesh_id().unwrap().to_string(),
                        name: self3.get_current_part_name().unwrap().to_string(),
                        part: Box::new(part.clone()),
                    },
                ));
                part.normals.indexed.push(Vec3::Y);
            }
        }

        // Named normals
        let norms = &mut s.state.ui.edit_collpased.n_normals;
        ui.horizontal(|ui| {
            let icon = if *norms { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                *norms = !*norms;
            }
            ui.label(self3.data.locale.get("named-normals"));
        });

        if !*norms {
            for (i, (key, norm)) in part.normals.named.iter_mut().enumerate() {
                if i != 0 {
                    ui.separator();
                }
                let mut name = key.clone();
                if let WorkingRenameKey::NamedNorm(ref name2) = self3.state.ui.working_key
                    && *name2 == name
                {
                    name = self3.state.ui.working_name.take().unwrap_or(name);
                }

                if ui
                    .horizontal(|ui| {
                        if ui
                            .add_sized([w - 24., 20.], egui::TextEdit::singleline(&mut name))
                            .changed()
                        {
                            let _ = self3.rename(editor::Rename::Normal {
                                part: editor::PartId {
                                    view_id: self3.get_current_mesh_id().unwrap().to_string(),
                                    name: self3.get_current_part_name().unwrap().to_string(),
                                },
                                swap: editor::DataSwap {
                                    from: data::mesh::NormalId::Named(key.clone()),
                                    to: data::mesh::NormalId::Named(name.clone()),
                                },
                            });
                            self3.state.ui.working_key = WorkingRenameKey::None;
                            return true;
                        }
                        if ui.small_button(SMALL_X).clicked() {
                            self3.add_history(editor::HistoryEntry::MeshPart(
                                light_mesh::LightMeshPartSnapshot {
                                    id: self3.get_current_mesh_id().unwrap().to_string(),
                                    name: self3.get_current_part_name().unwrap().to_string(),
                                    part: Box::new(part2.clone()),
                                },
                            ));
                            part2.delete_normals([NormalId::Named(key.clone())]);
                            self3.rebuild_meshes(gl);
                        }
                        false
                    })
                    .inner
                {
                    return;
                };

                if name != *key {
                    self3.state.ui.working_key = WorkingRenameKey::NamedNorm(key.clone());
                    self3.state.ui.working_name = Some(name);
                }

                vec3_row(
                    ui,
                    norm,
                    w3,
                    || part2.clone(),
                    |t| {
                        self3.add_history(editor::HistoryEntry::MeshPart(
                            light_mesh::LightMeshPartSnapshot {
                                id: self3.get_current_mesh_id().unwrap().to_string(),
                                name: self3.get_current_part_name().unwrap().to_string(),
                                part: Box::new(t),
                            },
                        ))
                    },
                    || self4.rebuild_meshes(gl),
                );
            }
            let mut new_norm = None;
            ui.add_sized(
                [w, 20.],
                TextInput::new(self3.data.locale.get("add-named-normal"), &mut new_norm),
            );
            if let Some(name) = new_norm {
                self3.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        id: self3.get_current_mesh_id().unwrap().to_string(),
                        name: self3.get_current_part_name().unwrap().to_string(),
                        part: Box::new(part.clone()),
                    },
                ));
                part.normals.named.insert(name, Vec3::Y);
            }
        }

        // Compute normals
        let norms = &mut s.state.ui.edit_collpased.c_normals;
        ui.horizontal(|ui| {
            let icon = if *norms { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                *norms = !*norms;
            }
            ui.label(self3.data.locale.get("compute-normals"));
        });

        if !*norms {
            for (i, (key, comp)) in part.normals.compute.iter_mut().enumerate() {
                if i != 0 {
                    ui.separator();
                }
                let mut name = key.clone();
                if let WorkingRenameKey::CompNorm(ref name2) = self3.state.ui.working_key
                    && *name2 == name
                {
                    name = self3.state.ui.working_name.take().unwrap_or(name);
                }

                if ui
                    .horizontal(|ui| {
                        if ui
                            .add_sized([w - 24., 20.], egui::TextEdit::singleline(&mut name))
                            .changed()
                        {
                            let _ = self3.rename(editor::Rename::Normal {
                                part: editor::PartId {
                                    view_id: self3.get_current_mesh_id().unwrap().to_string(),
                                    name: self3.get_current_part_name().unwrap().to_string(),
                                },
                                swap: editor::DataSwap {
                                    from: NormalId::Named(key.clone()),
                                    to: NormalId::Named(name.clone()),
                                },
                            });
                            self3.state.ui.working_key = WorkingRenameKey::None;
                            return true;
                        }
                        if ui.small_button(SMALL_X).clicked() {
                            self3.add_history(editor::HistoryEntry::MeshPart(
                                light_mesh::LightMeshPartSnapshot {
                                    id: self3.get_current_mesh_id().unwrap().to_string(),
                                    name: self3.get_current_part_name().unwrap().to_string(),
                                    part: Box::new(part2.clone()),
                                },
                            ));
                            part2.delete_normals([NormalId::Named(key.clone())]);
                            self3.rebuild_meshes(gl);
                        }
                        false
                    })
                    .inner
                {
                    return;
                }

                if name != *key {
                    self3.state.ui.working_key = WorkingRenameKey::CompNorm(key.clone());
                    self3.state.ui.working_name = Some(name);
                }

                ui.horizontal(|ui| {
                    for idx in 0..3 {
                        let id = &mut comp.points[idx];
                        let check = id.clone();
                        let disp = match id {
                            VertexId::Named(n) => n.to_string(),
                            VertexId::Index(i) => format!("{i}"),
                        };

                        egui::ComboBox::from_id_salt(format!("{key}-{idx}"))
                            .selected_text(disp)
                            .width(w3)
                            .show_ui(ui, |ui| {
                                for name in part2.get_valid_vertex_ids() {
                                    let disp = match &name {
                                        VertexId::Named(n) => n.to_string(),
                                        VertexId::Index(i) => format!("{i}"),
                                    };
                                    ui.selectable_value(id, name, disp);
                                }
                            });

                        if *id != check {
                            self3.add_history(editor::HistoryEntry::MeshPart(
                                light_mesh::LightMeshPartSnapshot {
                                    id: self3.get_current_mesh_id().unwrap().to_string(),
                                    name: self3.get_current_part_name().unwrap().to_string(),
                                    part: Box::new(part2.clone()),
                                },
                            ));
                            self3.rebuild_meshes(gl);
                        }
                    }
                });
            }
        }
    }
}

pub fn draw_edit_right(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
    let rd = RefDuper;
    let s2 = unsafe { rd.detach_mut_ref(s) };
    let s3 = unsafe { rd.detach_mut_ref(s) };
    let self3 = unsafe { rd.detach_mut_ref(s) };
    let self4 = unsafe { rd.detach_mut_ref(s) };

    let w = ui.available_width();

    let label = s.data.locale.get_with_args(
        "cycle-part-keys",
        &[
            (
                "left".into(),
                s.data
                    .keymaps
                    .toggle_mesh_part_back
                    .format(&MODIFIER_NAMES, cfg!(target_os = "macos"))
                    .into(),
            ),
            (
                "right".into(),
                s.data
                    .keymaps
                    .toggle_mesh_part_forward
                    .format(&MODIFIER_NAMES, cfg!(target_os = "macos"))
                    .into(),
            ),
        ]
        .into(),
    );
    ui.label(label);

    if let Some(current) = s2.get_current_part_name() {
        let mut rename = None;
        let ren = s
            .data
            .locale
            .get_with_args("rename-asset", &[("name".into(), current.into())].into());
        ui.add_sized([w, 20.], TextInput::new(&ren, &mut rename));
        if let Some(rename) = rename {
            let _ = s.rename(editor::Rename::Part {
                view_id: s.get_current_mesh_id().unwrap().to_string(),
                swap: editor::DataSwap {
                    from: current.to_string(),
                    to: rename,
                },
            });
        }
    }

    if let Selection::Vertices(verts) = &mut s2.selection
        && let Some(vm) = s3.get_current_view_mesh()
        && let Some(part) = s.get_current_part_mut()
    {
        let rd2 = RefDuper;
        let part2 = unsafe { rd2.detach_mut_ref(part) };
        let verts2: Vec<&VertexId> = verts.iter().collect();
        let mut values: Vec<&mut Vec3> = part.filter_non_compute_vertices(&verts2).collect();
        let w2 = (w - ui.spacing().item_spacing.x) / 2.;
        let w3 = (w - ui.spacing().item_spacing.x * 2.) / 3.;

        ui.label(self3.data.locale.get("multi-vertex"));
        multi_vec3_row(
            ui,
            &mut values,
            w3,
            || part2.clone(),
            |t| {
                self3.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        id: self3.get_current_mesh_id().unwrap().to_string(),
                        name: self3.get_current_part_name().unwrap().to_string(),
                        part: Box::new(t),
                    },
                ))
            },
            || self4.rebuild_meshes(gl),
        );

        enum VertVal<'a> {
            V3(&'a mut Vec3),
            C3(&'a mut ComputeVertex),
        }

        if verts.len() == 1
            && let [vert] = verts2.as_slice()
        {
            match match *vert {
                VertexId::Index(i) => {
                    ui.label(format!("{i}"));
                    VertVal::V3(part.vertices.indexed.get_mut(*i).unwrap())
                }
                VertexId::Named(n) => {
                    ui.label(n);
                    part.vertices
                        .named
                        .get_mut(n)
                        .map(VertVal::V3)
                        .unwrap_or_else(|| VertVal::C3(part.vertices.compute.get_mut(n).unwrap()))
                }
            } {
                VertVal::V3(v3) => {
                    ui.label(self3.data.locale.get("vertex-position"));
                    vec3_row(
                        ui,
                        v3,
                        w3,
                        || part2.clone(),
                        |t| {
                            self3.add_history(editor::HistoryEntry::MeshPart(
                                light_mesh::LightMeshPartSnapshot {
                                    id: self3.get_current_mesh_id().unwrap().to_string(),
                                    name: self3.get_current_part_name().unwrap().to_string(),
                                    part: Box::new(t),
                                },
                            ))
                        },
                        || self4.rebuild_meshes(gl),
                    );
                }
                VertVal::C3(c3) => {
                    ui.label(self3.data.locale.get("compute-position"));
                    let VertexId::Named(key) = vert else {
                        unreachable!()
                    };
                    compute_vertex_row(ui, (w2, w3), c3, key, part2, self3, gl);
                }
            }
        }

        if verts.len() == 2
            && let [v1, v2] = verts2.as_slice()
        {
            let mut comp_name = None;
            ui.add_sized(
                [w, 20.],
                TextInput::new(self3.data.locale.get("add-compute-vertex"), &mut comp_name),
            );
            if let Some(name) = comp_name {
                self3.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        id: self3.get_current_mesh_id().unwrap().to_string(),
                        name: self3.get_current_part_name().unwrap().to_string(),
                        part: Box::new(part2.clone()),
                    },
                ));
                let vert = ComputeVertex {
                    points: [(*v1).clone(), (*v2).clone()],
                    function: Easing::easeLinear,
                    delta: Some(0.5),
                    x: None,
                    y: None,
                    z: None,
                };
                let _ = part2.vertices.compute.insert(name, vert);
                self3.rebuild_meshes(gl);
            }
        }

        let part3 = unsafe { rd2.detach_mut_ref(part2) };
        let part4 = unsafe { rd2.detach_mut_ref(part2) };
        let mut tris: Vec<_> = part3
            .filter_triangles(&verts2)
            .map(|tri| {
                let [a, b, c] = &mut tri.vertices;
                (
                    [
                        (&mut a.normal, &mut a.uv),
                        (&mut b.normal, &mut b.uv),
                        (&mut c.normal, &mut c.uv),
                    ],
                    &mut tri.material,
                )
            })
            .collect();

        if verts.len() == 3
            && let [v1, v2, v3] = verts2.as_slice()
        {
            let mut v1 = *v1;
            let mut v2 = *v2;
            let mut v3 = *v3;

            let hint = if tris.len() == 1
                && let Some(tri) = part4.filter_triangles(&[v1, v2, v3]).next()
            {
                let [a, b, c] = &tri.vertices;
                v1 = &a.vertex;
                v2 = &b.vertex;
                v3 = &c.vertex;
                "add-compute-normal-wound"
            } else {
                "add-compute-normal"
            };
            let hint = self3.data.locale.get(hint);

            let mut comp_name = None;
            ui.add_sized([w, 20.], TextInput::new(hint, &mut comp_name));
            if let Some(name) = comp_name {
                self3.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        id: self3.get_current_mesh_id().unwrap().to_string(),
                        name: self3.get_current_part_name().unwrap().to_string(),
                        part: Box::new(part2.clone()),
                    },
                ));
                let norm = ComputeNormal {
                    points: [(*v1).clone(), (*v2).clone(), (*v3).clone()],
                };
                let _ = part2.normals.compute.insert(name, norm);
                self3.rebuild_meshes(gl);
            }
        }

        if !tris.is_empty() {
            ui.label(self3.data.locale.get("multi-triangle-data"));

            ui.label(self3.data.locale.get("normals"));
            ui.horizontal(|ui| {
                for (idx, v) in [(0, "a"), (1, "b"), (2, "c")] {
                    let mut normal = NormalId::Named(String::new());
                    egui::ComboBox::from_id_salt(format!("multi-normal-{v}"))
                        .selected_text(v)
                        .width(w3)
                        .show_ui(ui, |ui| {
                            for name in part.get_valid_normal_ids() {
                                let disp = match &name {
                                    NormalId::Named(n) => n.to_string(),
                                    NormalId::Index(i) => format!("{i}"),
                                };
                                ui.selectable_value(&mut normal, name.clone(), disp);
                            }
                        });
                    if normal != NormalId::Named(String::new()) {
                        self3.add_history(editor::HistoryEntry::MeshPart(
                            light_mesh::LightMeshPartSnapshot {
                                id: self3.get_current_mesh_id().unwrap().to_string(),
                                name: self3.get_current_part_name().unwrap().to_string(),
                                part: Box::new(part.clone()),
                            },
                        ));
                        for tri in tris.iter_mut() {
                            *tri.0[idx].0 = normal.clone();
                        }
                        self3.rebuild_meshes(gl);
                    }
                }
            });

            ui.label(self3.data.locale.get("uvs"));
            ui.horizontal(|ui| {
                for (idx, v) in [(0, "a"), (1, "b"), (2, "c")] {
                    let mut uv = UvId::Named(String::new());
                    egui::ComboBox::from_id_salt(format!("multi-uv-{v}"))
                        .selected_text(v)
                        .width(w3)
                        .show_ui(ui, |ui| {
                            for name in part.get_valid_uv_ids() {
                                let disp = match &name {
                                    UvId::Named(n) => n.to_string(),
                                    UvId::Index(i) => format!("{i}"),
                                };
                                ui.selectable_value(&mut uv, name.clone(), disp);
                            }
                        });
                    if uv != UvId::Named(String::new()) {
                        self3.add_history(editor::HistoryEntry::MeshPart(
                            light_mesh::LightMeshPartSnapshot {
                                id: self3.get_current_mesh_id().unwrap().to_string(),
                                name: self3.get_current_part_name().unwrap().to_string(),
                                part: Box::new(part.clone()),
                            },
                        ));
                        for tri in tris.iter_mut() {
                            *tri.0[idx].1 = uv.clone();
                        }
                        self3.rebuild_meshes(gl);
                    }
                }
            });
            ui.label(self3.data.locale.get("material"));
            ui.horizontal(|ui| {
                let mut mat = String::new();
                egui::ComboBox::from_id_salt("multi-material")
                    .width(w)
                    .show_ui(ui, |ui| {
                        for name in vm.data.data.keys() {
                            ui.selectable_value(&mut mat, name.to_string(), name);
                        }
                    });
                if !mat.is_empty() {
                    self3.add_history(editor::HistoryEntry::MeshPart(
                        light_mesh::LightMeshPartSnapshot {
                            id: self3.get_current_mesh_id().unwrap().to_string(),
                            name: self3.get_current_part_name().unwrap().to_string(),
                            part: Box::new(part.clone()),
                        },
                    ));
                    for tri in tris.iter_mut() {
                        if mat == "default" {
                            *tri.1 = None;
                        } else {
                            *tri.1 = Some(mat.clone())
                        }
                    }
                    self3.rebuild_meshes(gl);
                }
            });
        }

        if !verts.is_empty()
            && ui
                .add_sized(
                    [w, 20.],
                    egui::Button::new(self3.data.locale.get("dedupe-vertices")),
                )
                .clicked()
        {
            self3.add_history(editor::HistoryEntry::MeshPart(
                light_mesh::LightMeshPartSnapshot {
                    id: self3.get_current_mesh_id().unwrap().to_string(),
                    name: self3.get_current_part_name().unwrap().to_string(),
                    part: Box::new(part.clone()),
                },
            ));
            part.dedupe_data();
            self3.rebuild_meshes(gl);
        }
    }
}

pub fn draw_edit_gl(
    s: &UnsafeMutRef<App>,
    gl: &glow::Context,
    view: &Mat4,
    proj: &Mat4,
    window: (i32, i32),
) {
    let vp = proj * view;
    if let Some((_, name, _part)) = s.get_current_part()
        && let Some(sel) = s.editor.mesh.as_deref()
        && let Some(Some(vm)) = s.view.meshes.get(sel)
        && let Some(mesh) = vm.gpu_bufs.0.get(name)
    {
        let calls = vec![MeshDrawCall {
            mesh,
            instances: vec![InstanceData::new(Vec4::ZERO, Mat4::IDENTITY, LIGHT_COLORS)],
            wireframe: s.state.wireframe,
            cull: vm.data.cull,
            bloomfog: matches!(
                vm.data.bloomfog_style,
                BloomfogStyle::BloomOnly | BloomfogStyle::Everything
            ),
            bloom: vm.data.do_bloom,
            solid: vm.data.do_solid,
            mirror: vm.data.do_mirroring,
            obstacle: false,
            highlight: false,
        }];

        match s.state.view_style {
            editor::ViewStyle::Edit => {
                s.ref_mut().render.renderer.draw_meshes(
                    gl,
                    view,
                    proj,
                    &calls,
                    s.render.mirror.as_ref(),
                    s.state.wireframe,
                    false,
                    window,
                );
            }
            editor::ViewStyle::Beatcraft { blackout_sky } => {
                s.ref_mut().render.renderer.draw_meshes_fancy(
                    gl,
                    view,
                    proj,
                    &calls,
                    window,
                    if s.state.show_grid {
                        GridType::WorldGrid
                    } else {
                        GridType::None
                    },
                    s.render.mirror.as_ref(),
                    s.state.wireframe,
                    s.view.fog_heights.unwrap_or([-50., -30.]),
                    false,
                    if blackout_sky {
                        (0., 0., 0., 1.)
                    } else {
                        (0.07, 0.08, 0.11, 1.)
                    },
                );
            }
        }

        let mut calls = Vec::new();
        if s.state.show_verts {
            calls.push(PointDrawCall {
                mesh,
                instances: vec![InstanceData::new(
                    Vec4::ZERO,
                    Mat4::IDENTITY,
                    [Vec4::new(0., 1., 1., 1.); 8],
                )],
                size: 2.5,
            });
        }

        if let Some(selected) = s.render.sel_points.as_ref() {
            calls.push(PointDrawCall {
                mesh: selected,
                instances: vec![InstanceData::new(
                    Vec4::ZERO,
                    Mat4::IDENTITY,
                    [Vec4::new(1., 1., 0., 1.); 8],
                )],
                size: 4.,
            });
        }

        if !calls.is_empty() {
            s.render.renderer.draw_points_batch(gl, &vp, &calls);
        }
    }
}
