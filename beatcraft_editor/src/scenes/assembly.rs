
use crate::light_mesh::{self, BloomfogStyle, Part};
use crate::renaming::light_mesh::rehash;
use crate::render::{GridType, HandleDrawCall, InstanceData, MeshDrawCall, PointDrawCall};
use crate::widgets::TextInput;
use crate::{App, D_ARROW, R_ARROW, RefDuper, SMALL_R_ARROW, SMALL_X, UnsafeMutRef, editor};
use crate::data::mesh::{BillboardData, MaterialType, ShaderSettingsData, ShaderStyle};
use crate::editor::{RoutineAction, WorkingRenameKey};
use crate::ui_elements::*;
use eframe::glow;
use egui::Ui;
use glam::{Mat4, Quat, Vec3, Vec4};
use indexmap::IndexMap;
use indexmap::map::MutableKeys;
use std::sync::mpsc;

pub fn draw_assembly_left(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
    let rd = RefDuper;
    let self2 = unsafe { rd.detach_mut_ref(s) };
    let self3 = unsafe { rd.detach_mut_ref(s) };
    let self4 = unsafe { rd.detach_mut_ref(s) };
    if let Some(mesh) = s.get_current_view_mesh_mut() {
        let rd2 = RefDuper;
        let mesh2 = unsafe { rd2.detach_mut_ref(mesh) };
        let path = mesh.path.clone();
        let part_names = mesh.data.part_names.clone();
        let toggles = self2
            .state
            .ui
            .assembly_collapsed
            .entry(path.clone())
            .or_default();
        let w = ui.available_width() - ui.spacing().item_spacing.x;
        let w2 = (w - ui.spacing().item_spacing.x) / 2.0;
        let w3 = (w - ui.spacing().item_spacing.x * 2.0) / 3.0;

        ui.horizontal(|ui| {
            let icon = if toggles.placements { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                toggles.placements = !toggles.placements;
            }
            ui.label(self2.data.locale.get("placements"));
        });

        if !toggles.placements {
            let mut to_remove = None;
            for (pi, placement) in mesh.data.placements.iter_mut().enumerate() {
                let pt_collapsed = toggles
                    .placement_parts
                    .entry(pi)
                    .or_insert(([true, true, true], Default::default()));

                ui.horizontal(|ui| {
                    let icon = if pt_collapsed.0[0] { R_ARROW } else { D_ARROW };
                    if ui.small_button(icon).clicked() {
                        pt_collapsed.0[0] = !pt_collapsed.0[0];
                    }

                    egui::ComboBox::from_id_salt(egui::Id::new("placement_part").with(pi))
                        .selected_text(placement.part.as_str())
                        .width(w - ui.spacing().item_spacing.x * 2. - 24.)
                        .show_ui(ui, |ui| {
                            for name in &part_names {
                                ui.selectable_value(
                                    &mut placement.part,
                                    name.clone(),
                                    name.as_str(),
                                );
                            }
                        });

                    if ui.small_button(SMALL_X).clicked() {
                        to_remove = Some(pi);
                    }
                });

                if !pt_collapsed.0[0] {
                    let rot_mode = &mut pt_collapsed.1;

                    ui.horizontal(|ui| {
                        ui.label(self2.data.locale.get("position"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |_ui| {
                            // if ui.small_button("Paste").clicked() { /* TODO */ }
                            // if ui.small_button("Copy").clicked() { /* TODO */ }
                        });
                    });
                    vec3_row(
                        ui,
                        &mut placement.position,
                        w3,
                        || mesh2.data.placements.clone(),
                        |t| {
                            self3.add_history(editor::HistoryEntry::MeshPlacement(
                                light_mesh::LightMeshPlacementSnapshot {
                                    view_id: self3.get_current_mesh_id().unwrap().to_string(),
                                    placements: t,
                                },
                            ))
                        },
                        || self4.rebuild_meshes(gl),
                    );

                    ui.horizontal(|ui| {
                        ui.label(self2.data.locale.get("rotation"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |_ui| {
                            // if ui.small_button("Paste").clicked() { /* TODO */ }
                            // if ui.small_button("Copy").clicked() { /* TODO */ }
                        });
                    });
                    quat_row(
                        ui,
                        &mut placement.rotation,
                        rot_mode,
                        (w2, w3),
                        || mesh2.data.placements.clone(),
                        |t| {
                            self3.add_history(editor::HistoryEntry::MeshPlacement(
                                light_mesh::LightMeshPlacementSnapshot {
                                    view_id: self3.get_current_mesh_id().unwrap().to_string(),
                                    placements: t,
                                },
                            ))
                        },
                        || self4.rebuild_meshes(gl),
                    );

                    ui.horizontal(|ui| {
                        ui.label(self2.data.locale.get("scale"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |_ui| {
                            // if ui.small_button("Paste").clicked() { /* TODO */ }
                            // if ui.small_button("Copy").clicked() { /* TODO */ }
                            // TODO: re-add lock system when it works
                            // correctly
                            // let lock_icon = if pt_collapsed.0[2] { "[#]" } else { "[ ]" };
                            // if ui.small_button(lock_icon).clicked() {
                            //     pt_collapsed.0[2] = !pt_collapsed.0[2];
                            // }
                        });
                    });
                    vec3_row(
                        ui,
                        &mut placement.scale,
                        w3,
                        || mesh2.data.placements.clone(),
                        |t| {
                            self3.add_history(editor::HistoryEntry::MeshPlacement(
                                light_mesh::LightMeshPlacementSnapshot {
                                    view_id: self3.get_current_mesh_id().unwrap().to_string(),
                                    placements: t,
                                },
                            ))
                        },
                        || self4.rebuild_meshes(gl),
                    );

                    let mut delete_bb = false;
                    match placement.billboard.as_mut() {
                        Some(bb) => {
                            ui.horizontal(|ui| {
                                ui.add_sized(
                                    [w - ui.spacing().item_spacing.x - 24., 20.],
                                    egui::Label::new(self2.data.locale.get("graphics-billboard")),
                                );

                                delete_bb = ui.small_button(SMALL_X).clicked();
                            });
                            ui.label(self2.data.locale.get("billboard-origin"));
                            vec3_row(
                                ui,
                                &mut bb.origin,
                                w3,
                                || mesh2.data.placements.clone(),
                                |t| {
                                    self3.add_history(editor::HistoryEntry::MeshPlacement(
                                        light_mesh::LightMeshPlacementSnapshot {
                                            view_id: self3
                                                .get_current_mesh_id()
                                                .unwrap()
                                                .to_string(),
                                            placements: t,
                                        },
                                    ))
                                },
                                || self4.rebuild_meshes(gl),
                            );
                            ui.label(self2.data.locale.get("billboard-axis"));
                            vec3_row(
                                ui,
                                &mut bb.axis,
                                w3,
                                || mesh2.data.placements.clone(),
                                |t| {
                                    self3.add_history(editor::HistoryEntry::MeshPlacement(
                                        light_mesh::LightMeshPlacementSnapshot {
                                            view_id: self3
                                                .get_current_mesh_id()
                                                .unwrap()
                                                .to_string(),
                                            placements: t,
                                        },
                                    ))
                                },
                                || self4.rebuild_meshes(gl),
                            );
                            ui.label(self2.data.locale.get("billboard-normal"));
                            vec3_row(
                                ui,
                                &mut bb.normal,
                                w3,
                                || mesh2.data.placements.clone(),
                                |t| {
                                    self3.add_history(editor::HistoryEntry::MeshPlacement(
                                        light_mesh::LightMeshPlacementSnapshot {
                                            view_id: self3
                                                .get_current_mesh_id()
                                                .unwrap()
                                                .to_string(),
                                            placements: t,
                                        },
                                    ))
                                },
                                || self4.rebuild_meshes(gl),
                            );
                            if ui
                                .add_sized(
                                    [w, 20.],
                                    egui::Button::new(self2.data.locale.get("camera-lock"))
                                        .selected(bb.camera_lock),
                                )
                                .clicked()
                            {
                                self3.add_history(editor::HistoryEntry::MeshPlacement(
                                    light_mesh::LightMeshPlacementSnapshot {
                                        view_id: self3.get_current_mesh_id().unwrap().to_string(),
                                        placements: mesh2.data.placements.clone(),
                                    },
                                ));
                                bb.camera_lock = !bb.camera_lock;
                                self4.rebuild_meshes(gl);
                            }
                        }
                        None => {
                            if ui
                                .add_sized(
                                    [w, 20.],
                                    egui::Button::new(self2.data.locale.get("set-billboard")),
                                )
                                .clicked()
                            {
                                placement.billboard = Some(BillboardData::default());
                            }
                        }
                    }
                    if delete_bb {
                        placement.billboard = None;
                    }

                    let mut remove_settings = false;
                    match placement.shader_settings.as_mut() {
                        Some(sets) => {
                            ui.horizontal(|ui| {
                                ui.add_sized(
                                    [w - ui.spacing().item_spacing.x - 24., 20.],
                                    egui::Label::new(self2.data.locale.get("shader-settings")),
                                );

                                remove_settings = ui.small_button(SMALL_X).clicked();
                            });

                            let mut current = sets.style;
                            egui::ComboBox::new(
                                format!("shader-style-{}", pi),
                                self2.data.locale.get("visual-style"),
                            )
                            .selected_text(sets.style.translation_text(&mut self2.data.locale))
                            .show_ui(ui, |ui| {
                                for style in ShaderStyle::iter_all() {
                                    ui.selectable_value(
                                        &mut current,
                                        style,
                                        style.translation_text(&mut self2.data.locale),
                                    );
                                }
                            });

                            if current != sets.style {
                                self3.add_history(editor::HistoryEntry::MeshPlacement(
                                    light_mesh::LightMeshPlacementSnapshot {
                                        view_id: self3.get_current_mesh_id().unwrap().to_string(),
                                        placements: mesh2.data.placements.clone(),
                                    },
                                ));
                                sets.style = current;
                                self4.rebuild_meshes(gl);
                            }
                        }
                        None => {
                            if ui
                                .add_sized(
                                    [w, 20.],
                                    egui::Button::new(self2.data.locale.get("set-shader-settings")),
                                )
                                .clicked()
                            {
                                placement.shader_settings = Some(ShaderSettingsData::default());
                            }
                        }
                    }

                    ui.horizontal(|ui| {
                        let icon = if pt_collapsed.0[1] { R_ARROW } else { D_ARROW };
                        if ui.small_button(icon).clicked() {
                            pt_collapsed.0[1] = !pt_collapsed.0[1];
                        }
                        ui.label(self2.data.locale.get("remap-data"));
                    });

                    if !pt_collapsed.0[1] {
                        let mut remap_to_remove = None;
                        for (ri, (from, to)) in placement.remap_data.iter_mut2().enumerate() {
                            ui.horizontal(|ui| {
                                let fw = (w - ui.spacing().item_spacing.x * 3. - 24.) / 2.;
                                ui.add_sized([fw, 20.], egui::TextEdit::singleline(from));
                                ui.label(SMALL_R_ARROW);
                                ui.add_sized([fw, 20.], egui::TextEdit::singleline(to));
                                if ui.small_button(SMALL_X).clicked() {
                                    remap_to_remove = Some(ri);
                                }
                            });
                        }
                        let remap = std::mem::take(&mut placement.remap_data);
                        placement.remap_data = rehash(remap);
                        if let Some(ri) = remap_to_remove {
                            placement.remap_data.shift_remove_index(ri);
                        }
                        if ui
                            .add_sized(
                                [w, 20.],
                                egui::Button::new(self2.data.locale.get("add-data")),
                            )
                            .clicked()
                        {
                            placement.remap_data.insert(String::new(), String::new());
                        }
                    }
                }

                ui.separator();
            }

            if let Some(i) = to_remove {
                mesh.data.placements.remove(i);
                toggles.placement_parts.remove(&i);
                self3.rebuild_meshes(gl);
            }
            if ui
                .add_sized(
                    [w, 20.],
                    egui::Button::new(self2.data.locale.get("add-placement")),
                )
                .clicked()
                && let Some(first) = part_names.first()
            {
                self3.add_history(editor::HistoryEntry::Mesh(light_mesh::LightMeshSnapshot {
                    id: self3.get_current_mesh_id().unwrap().to_string(),
                    mesh: Box::new(mesh.data.clone()),
                }));
                mesh.data.placements.push(light_mesh::Placement {
                    part: first.clone(),
                    position: Vec3::ZERO,
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                    billboard: None,
                    shader_settings: None,
                    remap_data: IndexMap::new(),
                });
                self3.rebuild_meshes(gl);
            }
        }

        ui.horizontal(|ui| {
            let icon = if toggles.data { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                toggles.data = !toggles.data;
            }
            ui.label(self2.data.locale.get("data"));
        });

        if !toggles.data {
            let mut data_to_remove = None;
            let data_keys: Vec<String> = mesh.data.data.keys().cloned().collect();
            for (di, key) in data_keys.iter().enumerate() {
                let entry = mesh.data.data.get_mut(key).unwrap();
                let di_collapsed = toggles.datas.entry(di).or_insert(true);

                if ui
                    .horizontal(|ui| {
                        let icon = if *di_collapsed { R_ARROW } else { D_ARROW };
                        if ui.small_button(icon).clicked() {
                            *di_collapsed = !*di_collapsed;
                        }
                        let mut name = key.clone();
                        if let WorkingRenameKey::DataTag(ref name2) = self2.state.ui.working_key
                            && *name2 == name
                        {
                            name = self2.state.ui.working_name.take().unwrap_or(name);
                        }
                        if ui
                            .add_sized(
                                [w - ui.spacing().item_spacing.x * 2. - 24., 20.],
                                egui::TextEdit::singleline(&mut name),
                            )
                            .changed()
                        {
                            let _ = self3.rename(editor::Rename::DataTag {
                                view_id: self3.get_current_mesh_id().unwrap().to_string(),
                                swap: editor::DataSwap {
                                    from: key.clone(),
                                    to: name,
                                },
                            });
                            self3.state.ui.working_key = WorkingRenameKey::None;
                            return true;
                        }
                        if name != *key {
                            self2.state.ui.working_key = WorkingRenameKey::DataTag(key.clone());
                            self2.state.ui.working_name = Some(name);
                        }
                        if ui.small_button(SMALL_X).clicked() {
                            data_to_remove = Some(key.clone());
                        }
                        false
                    })
                    .inner
                {
                    break;
                };

                if !*di_collapsed {
                    let mat_label = entry.material.name();
                    let mut current = entry.material;
                    egui::ComboBox::from_id_salt(egui::Id::new("material_id").with(di))
                        .selected_text(mat_label)
                        .show_ui(ui, |ui| {
                            for v in MaterialType::iter_all() {
                                ui.selectable_value(&mut current, v, v.name());
                            }
                        });
                    if current != entry.material {
                        self3.add_history(editor::HistoryEntry::MeshMeta(
                            mesh2.data.snapshot_mesh_meta(
                                self3.get_current_mesh_id().unwrap().to_string(),
                            ),
                        ));
                        entry.material = current;
                        self3.rebuild_meshes(gl);
                    }
                    let mut current = entry.color;
                    let ch = self2.data.locale.get_with_args(
                        "data-channel",
                        &[("channel".into(), current.into())].into(),
                    );
                    egui::ComboBox::from_id_salt(egui::Id::new("data_ch").with(di))
                        .selected_text(ch)
                        .show_ui(ui, |ui| {
                            for ch in 0u8..8 {
                                ui.selectable_value(&mut current, ch, ch.to_string());
                            }
                        });
                    if current != entry.color {
                        self3.add_history(editor::HistoryEntry::MeshMeta(
                            mesh2.data.snapshot_mesh_meta(
                                self3.get_current_mesh_id().unwrap().to_string(),
                            ),
                        ));
                        entry.color = current;
                        self3.rebuild_meshes(gl);
                    }

                    ui.horizontal(|ui| {
                        ui.label(self2.data.locale.get("texture"));
                        let mut current = entry.texture;
                        let resp =
                            ui.add(egui::DragValue::new(&mut current).range(0..=255u8).speed(1));
                        if resp.changed() {
                            self3.add_history(editor::HistoryEntry::MeshMeta(
                                mesh2.data.snapshot_mesh_meta(
                                    self3.get_current_mesh_id().unwrap().to_string(),
                                ),
                            ));
                            entry.texture = current;
                            self3.rebuild_meshes(gl);
                        }
                    });
                }

                ui.separator();
            }

            if let Some(key) = data_to_remove {
                self3.add_history(editor::HistoryEntry::MeshMeta(
                    mesh2
                        .data
                        .snapshot_mesh_meta(self3.get_current_mesh_id().unwrap().to_string()),
                ));
                mesh.data.data.shift_remove(&key);
                self3.rebuild_meshes(gl);
            }

            if ui
                .add_sized(
                    [w, 20.],
                    egui::Button::new(self2.data.locale.get("add-data-label")),
                )
                .clicked()
            {
                self3.add_history(editor::HistoryEntry::MeshMeta(
                    mesh2
                        .data
                        .snapshot_mesh_meta(self3.get_current_mesh_id().unwrap().to_string()),
                ));
                mesh.data.data.insert(
                    format!("new_data_{}", mesh.data.data.len()),
                    Default::default(),
                );
                self3.rebuild_meshes(gl);
            }
        }

        ui.horizontal(|ui| {
            let icon = if toggles.textures { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                toggles.textures = !toggles.textures;
            }
            ui.label(self2.data.locale.get("textures"));
        });

        if !toggles.textures {
            let mut tex_to_remove = None;
            let tex_keys: Vec<String> = mesh.data.textures.keys().cloned().collect();
            for key in tex_keys.iter() {
                let val = mesh.data.textures.get_mut(key).unwrap();
                ui.horizontal(|ui| {
                    ui.label(key);
                    ui.add_sized([w - 60., 20.], egui::TextEdit::singleline(val));
                    if ui
                        .small_button(if self2.render.renderer.texture_paths.contains_key(val) {
                            "R"
                        } else {
                            "?"
                        })
                        .clicked()
                    {
                        let (sx, rx) = mpsc::channel();
                        let id = val.clone();
                        let title = self2.data.locale.get("title-choose-image").to_string();
                        std::thread::spawn(move || {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title(title)
                                .add_filter("png", &["png"])
                                .pick_file()
                            {
                                let _ = sx.send(path);
                            }
                        });
                        self3.add_routine(Box::new(move |s, gl| match rx.try_recv() {
                            Err(mpsc::TryRecvError::Empty) => RoutineAction::None,
                            Err(mpsc::TryRecvError::Disconnected) => RoutineAction::Remove,
                            Ok(src) => {
                                s.render.renderer.texture_paths.insert(id.clone(), src);
                                s.rebuild_meshes(gl);
                                RoutineAction::Remove
                            }
                        }));
                    }
                    if ui.small_button(SMALL_X).clicked() {
                        tex_to_remove = Some(key.clone());
                    }
                });
            }

            if let Some(key) = tex_to_remove {
                mesh.data.textures.shift_remove(&key);
                self3.rebuild_meshes(gl);
            }

            if ui
                .add_sized(
                    [w, 20.],
                    egui::Button::new(self2.data.locale.get("add-texture")),
                )
                .clicked()
            {
                let next_key = (0..)
                    .map(|i| format!("{}", i))
                    .find(|k| !mesh.data.textures.contains_key(k))
                    .unwrap();
                mesh.data.textures.insert(next_key, String::new());
            }
        }

        ui.horizontal(|ui| {
            let icon = if toggles.settings { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                toggles.settings = !toggles.settings;
            }
            ui.label(self2.data.locale.get("label-render-settings"));
        });

        if !toggles.settings {
            ui.horizontal(|ui| {
                let size = egui::vec2(w2, 20.);
                let layout = egui::Layout::left_to_right(egui::Align::Center);
                ui.allocate_ui_with_layout(size, layout, |ui| {
                    ui.set_min_width(w2);
                    ui.checkbox(&mut mesh.data.cull, self2.data.locale.get("setting-cull"))
                });
                ui.allocate_ui_with_layout(size, layout, |ui| {
                    ui.set_min_width(w2);
                    ui.checkbox(
                        &mut mesh.data.do_bloom,
                        self2.data.locale.get("setting-bloom"),
                    )
                });
            });
            ui.horizontal(|ui| {
                let size = egui::vec2(w2, 20.);
                let layout = egui::Layout::left_to_right(egui::Align::Center);
                ui.allocate_ui_with_layout(size, layout, |ui| {
                    ui.set_min_width(w2);
                    ui.checkbox(
                        &mut mesh.data.do_mirroring,
                        self2.data.locale.get("setting-mirror"),
                    )
                });
                ui.allocate_ui_with_layout(size, layout, |ui| {
                    ui.set_min_width(w2);
                    ui.checkbox(
                        &mut mesh.data.do_solid,
                        self2.data.locale.get("setting-solid"),
                    )
                });
            });
            egui::ComboBox::from_id_salt("bloomfog_style")
                .selected_text(mesh.data.bloomfog_style.label(&mut self2.data.locale))
                .width(w)
                .show_ui(ui, |ui| {
                    for style in [
                        BloomfogStyle::BloomOnly,
                        BloomfogStyle::Everything,
                        BloomfogStyle::Nothing,
                    ] {
                        ui.selectable_value(
                            &mut mesh.data.bloomfog_style,
                            style,
                            style.label(&mut self2.data.locale),
                        );
                    }
                });
        }

        ui.horizontal(|ui| {
            let icon = if toggles.credits { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                toggles.credits = !toggles.credits;
            }
            ui.label(self2.data.locale.get("credits"));
        });

        if !toggles.credits {
            let mut to_remove = None;
            for (ci, credit) in mesh.data.credits.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [w - ui.spacing().item_spacing.x - 24., 20.],
                        egui::TextEdit::singleline(credit),
                    );
                    if ui.small_button(SMALL_X).clicked() {
                        to_remove = Some(ci);
                    }
                });
            }
            if let Some(i) = to_remove {
                mesh.data.credits.remove(i);
                self3.rebuild_meshes(gl);
            }
            if ui
                .add_sized(
                    [w, 20.],
                    egui::Button::new(self2.data.locale.get("add-credit")),
                )
                .clicked()
            {
                mesh.data.credits.push(String::new());
            }
        }
    }
}

pub fn draw_assembly_right(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
    let rd = RefDuper;
    let s2 = unsafe { rd.detach_mut_ref(s) };
    let w = ui.available_width();
    if let Some(mesh) = s.get_current_view_mesh_mut() {
        ui.label(s2.data.locale.get("parts-label"));
        let mesh2 = unsafe { rd.detach_mut_ref(mesh) };
        for name in mesh.data.part_names.iter() {
            if ui
                .horizontal(|ui| {
                    let mut new_name = None;
                    ui.add_sized([w - 24., 20.], TextInput::new(name, &mut new_name));
                    if let Some(new_name) = new_name {
                        let _ = s2.rename(editor::Rename::Part {
                            view_id: s2.get_current_mesh_id().unwrap().to_string(),
                            swap: editor::DataSwap {
                                from: name.clone(),
                                to: new_name,
                            },
                        });
                        return true;
                    }
                    if ui.small_button(SMALL_X).clicked() {
                        s2.add_history(editor::HistoryEntry::Mesh(light_mesh::LightMeshSnapshot {
                            id: s2.get_current_mesh_id().unwrap().to_string(),
                            mesh: Box::new(mesh2.data.clone()),
                        }));
                        let _ = mesh2.data.parts.shift_remove(name);
                        mesh2.data.placements.retain(|p| p.part != *name);
                        s2.rebuild_meshes(gl);
                        return true;
                    }
                    false
                })
                .inner
            {
                return;
            };
        }
        let mut new_part = None;
        ui.add_sized(
            [w, 20.],
            TextInput::new(s2.data.locale.get("add-part"), &mut new_part),
        );
        if let Some(name) = new_part {
            s2.add_history(editor::HistoryEntry::Mesh(light_mesh::LightMeshSnapshot {
                id: s2.get_current_mesh_id().unwrap().to_string(),
                mesh: Box::new(mesh2.data.clone()),
            }));
            mesh.data.parts.insert(name, Part::default());
            s2.rebuild_meshes(gl);
        }
    }
}

pub fn draw_assembly_gl(
    s: &UnsafeMutRef<App>,
    gl: &glow::Context,
    view: &Mat4,
    proj: &Mat4,
    window: (i32, i32),
) {
    let vp = proj * view;
    if let Some(sel) = s.editor.mesh.as_deref()
        && let Some(Some(vm)) = s.view.meshes.get(sel)
    {
        let mut instances = Vec::new();
        if let Some(mesh) = vm.render_assembly(&mut instances) {
            match s.state.view_style {
                editor::ViewStyle::Edit => {
                    s.ref_mut().render.renderer.draw_meshes(
                        gl,
                        view,
                        proj,
                        &[MeshDrawCall {
                            mesh,
                            instances: instances.clone(),
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
                        }],
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
                        &[MeshDrawCall {
                            mesh,
                            instances: instances.clone(),
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
                        }],
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
        }
        if s.state.show_verts
            && let Some(handles) = vm.gpu_bufs.2.as_ref()
        {
            s.render.renderer.draw_handles(
                gl,
                &vp,
                &[HandleDrawCall {
                    mesh: handles,
                    instances: vec![InstanceData::new(
                        Vec4::ZERO,
                        Mat4::IDENTITY,
                        [Vec4::splat(1.); 8],
                    )],
                }],
            );
        }

        let mut calls = Vec::new();

        if let Some(selected) = s.render.inst_points.as_ref() {
            calls.push(PointDrawCall {
                mesh: selected,
                instances: vec![InstanceData::new(
                    Vec4::ZERO,
                    Mat4::IDENTITY,
                    [Vec4::splat(1.); 8],
                )],
                size: 6.,
            });
        }

        if !calls.is_empty() {
            s.render.renderer.draw_points_batch(gl, &vp, &calls);
        }
    }
}
