use crate::data::mesh::{LightGroup, LightMeshData, SpectrogramData};
use crate::editor::{ActionType, RingType, RoutineAction, SpinSide, ViewPlacement};
use crate::light_mesh::BloomfogStyle;
use crate::renaming::light_mesh::rehash;
use crate::render::{GridType, InstanceData, LIGHT_COLORS, MeshDrawCall};
use crate::ui_elements::*;
use crate::widgets::TextInput;
use crate::{
    App, D_ARROW, R_ARROW, RefDuper, SMALL_X, UnsafeMutRef, close_environment, data, editor,
};
use bs_mapping_data::easing::Easing;
use eframe::glow;
use egui::{Layout, Ui};
use fluent_templates::fluent_bundle::FluentValue;
use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};
use indexmap::IndexMap;
use indexmap::map::MutableKeys;
use std::borrow::Cow;
use std::fs;
use std::sync::mpsc;

pub fn draw_view_left(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
    let mut to_remove = None;
    let w = ui.available_width();
    let w2 = (w - ui.spacing().item_spacing.x) / 2.0;
    let w3 = (w - ui.spacing().item_spacing.x * 2.0) / 3.0;
    let rd = RefDuper;
    let s2 = unsafe { rd.detach_mut_ref(s) };
    let s3 = unsafe { rd.detach_mut_ref(s) };
    let s4 = unsafe { rd.detach_mut_ref(s) };

    ui.horizontal(|ui| {
        let icon = if s2.state.ui.meshes_collapse {
            R_ARROW
        } else {
            D_ARROW
        };
        if ui.small_button(icon).clicked() {
            s2.state.ui.meshes_collapse = !s2.state.ui.meshes_collapse;
        }
        ui.label(s.data.locale.get("meshes"));
    });

    if !s2.state.ui.meshes_collapse {
        for (id, mesh) in s
            .view
            .meshes
            .iter_mut()
            .filter_map(|(i, v)| v.as_mut().map(|v| (i, v)))
        {
            ui.add_space(2.0);
            let selected = s.state.ui.view_mesh == Some(id.clone());
            let available = ui.available_size_before_wrap();
            if ui
                .add_sized(
                    egui::Vec2::new(available.x, 20.0),
                    egui::Button::selectable(selected, id.as_str()),
                )
                .clicked()
            {
                s.state.ui.view_mesh = if selected { None } else { Some(id.clone()) };
            };
            ui.add_space(2.0);

            if ui
                .horizontal(|ui| {
                    ui.checkbox(&mut mesh.visible, "");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(s.data.locale.get("button-close")).clicked() {
                            to_remove = Some(id.clone());
                        }
                        if ui.button(s.data.locale.get("button-edit")).clicked() {
                            s.editor.mesh = Some(id.clone());
                            s.mode = editor::EditorMode::Assembly;
                            s2.rebuild_meshes(gl);
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

            ui.separator();
        }
        if let Some(i) = to_remove {
            s.view.meshes.shift_remove(&i);
            if let Some(sel) = s.state.ui.view_mesh.as_ref() {
                if *sel == i {
                    s.state.ui.view_mesh = None;
                } else if !s.view.meshes.is_empty() {
                    s.state.ui.view_mesh = Some(s.view.meshes.keys().next().unwrap().clone());
                }
            }
            if let Some(sel) = s.editor.mesh.as_ref() {
                if *sel == i {
                    s.editor.mesh = None;
                } else if !s.view.meshes.is_empty() {
                    s.editor.mesh = Some(s.view.meshes.keys().next().unwrap().clone());
                }
            }
        }

        if ui
            .add_sized(
                [w, 20.],
                egui::Button::new(s.data.locale.get("button-create-mesh")),
            )
            .clicked()
        {
            let (sx, rx) = mpsc::channel();
            let title = s.data.locale.get("title-create-mesh").to_string();
            std::thread::spawn(move || {
                if let Some(file) = rfd::FileDialog::new()
                    .set_title(title)
                    .set_file_name("new_mesh")
                    .add_filter("json", &["json"])
                    .save_file()
                {
                    let _ = sx.send(file);
                }
            });
            s.add_routine(Box::new(move |s, gl| match rx.try_recv() {
                Err(mpsc::TryRecvError::Empty) => RoutineAction::None,
                Err(mpsc::TryRecvError::Disconnected) => RoutineAction::Remove,
                Ok(dest) => {
                    if let Err(e) = fs::write(
                        &dest,
                        serde_json::to_string(&LightMeshData::default()).unwrap(),
                    ) {
                        eprintln!("Failed to write mesh file\n{e}");
                    } else if let Err(e) = s.load_meshes(
                        {
                            let mut m = IndexMap::new();
                            m.insert(s.get_unique_mesh_id(), dest);
                            m
                        },
                        gl,
                    ) {
                        eprintln!("Failed to load mesh\n{e}");
                    }
                    RoutineAction::Remove
                }
            }));
        }
    }

    ui.separator();

    let mut remove_spect = false;
    match s.view.spectrogram.as_mut() {
        Some(spect) => {
            let spect2 = unsafe { rd.detach_ref(spect) };

            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    [w, 20.].into(),
                    Layout::right_to_left(egui::Align::Min),
                    |ui| {
                        if ui.small_button(SMALL_X).clicked() {
                            remove_spect = true;
                        }
                        ui.allocate_ui_with_layout(
                            [w - 25., 20.].into(),
                            Layout::left_to_right(egui::Align::Min),
                            |ui| {
                                let icon = if s2.state.ui.spectrogram_collapse {
                                    R_ARROW
                                } else {
                                    D_ARROW
                                };
                                if ui.small_button(icon).clicked() {
                                    s2.state.ui.spectrogram_collapse =
                                        !s2.state.ui.spectrogram_collapse;
                                }
                                ui.label(s.data.locale.get("spectrogram"));
                            },
                        );
                    },
                );
            });

            if !s2.state.ui.spectrogram_collapse {
                ui.label(s.data.locale.get("position"));
                vec3_row(
                    ui,
                    &mut spect.position,
                    w3,
                    || Some(spect2.clone()),
                    |s| s3.add_history(editor::HistoryEntry::Spectrogram(s)),
                    || s2.rebuild_meshes(gl),
                );
                ui.label(s.data.locale.get("offset"));
                vec3_row(
                    ui,
                    &mut spect.offset,
                    w3,
                    || Some(spect2.clone()),
                    |s| s3.add_history(editor::HistoryEntry::Spectrogram(s)),
                    || s2.rebuild_meshes(gl),
                );
                ui.label(s.data.locale.get("rotation"));
                quat_row(
                    ui,
                    &mut spect.rotation,
                    &mut s2.state.ui.spectrogram_mode,
                    (w2, w3),
                    || Some(spect2.clone()),
                    |s| s3.add_history(editor::HistoryEntry::Spectrogram(s)),
                    || s4.rebuild_meshes(gl),
                );
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        [w2, 45.].into(),
                        Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.label(s.data.locale.get("item-count"));
                            let resp = ui.add_sized(
                                [w2, 20.],
                                egui::DragValue::new(&mut spect.count)
                                    .speed(0.25)
                                    .range(1..=500),
                            );
                            trigger_history(
                                ui,
                                &[resp],
                                || spect2.clone(),
                                |x| s2.add_history(editor::HistoryEntry::Spectrogram(Some(x))),
                                || s3.rebuild_meshes(gl),
                            );
                        },
                    );
                    ui.allocate_ui_with_layout(
                        [w2, 45.].into(),
                        Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.label(s.data.locale.get("visual-style"));
                            let old = spect.style;
                            ui.allocate_ui_with_layout(
                                [w2, 20.].into(),
                                Layout::left_to_right(egui::Align::Max),
                                |ui| {
                                    egui::ComboBox::from_id_salt("spect-style")
                                        .selected_text(spect.style.name())
                                        .width(w2)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut spect.style,
                                                data::mesh::TowerStyle::Cuboid,
                                                "cuboid",
                                            );
                                        })
                                },
                            );
                            if old != spect.style {
                                let mut old_s = spect.clone();
                                old_s.style = old;
                                s2.add_history(editor::HistoryEntry::Spectrogram(Some(old_s)));
                                s2.rebuild_meshes(gl);
                            }
                        },
                    );
                });
                ui.horizontal(|ui| {
                    if ui
                        .allocate_ui_with_layout(
                            [w2, 45.].into(),
                            Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.label(s.data.locale.get("tower-half-split"));
                                ui.add_sized(
                                    [w2, 20.],
                                    egui::Button::new(if spect.half_split {
                                        s.data.locale.get("enabled")
                                    } else {
                                        s.data.locale.get("disabled")
                                    }),
                                )
                            },
                        )
                        .inner
                        .clicked()
                    {
                        s2.add_history(editor::HistoryEntry::Spectrogram(Some(spect.clone())));
                        spect.half_split = !spect.half_split;
                        s2.rebuild_meshes(gl);
                    }
                    ui.allocate_ui_with_layout(
                        [w2, 45.].into(),
                        Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.label(s.data.locale.get("tower-height-modifier"));
                            let resp = ui.add_sized(
                                [w2, 20.],
                                egui::DragValue::new(&mut spect.level_modifier)
                                    .speed(0.01)
                                    .range(0..=5),
                            );
                            trigger_history(
                                ui,
                                &[resp],
                                || spect2.clone(),
                                |x| s2.add_history(editor::HistoryEntry::Spectrogram(Some(x))),
                                || s3.rebuild_meshes(gl),
                            );
                        },
                    );
                });
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        [w3, 45.].into(),
                        Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.label(s.data.locale.get("tower-height-position"));
                            let resp = ui.add_sized(
                                [w3 + 5., 20.],
                                egui::DragValue::new(&mut spect.base_height)
                                    .speed(1.)
                                    .range(0..=800),
                            );
                            trigger_history(
                                ui,
                                &[resp],
                                || spect2.clone(),
                                |x| s2.add_history(editor::HistoryEntry::Spectrogram(Some(x))),
                                || s3.rebuild_meshes(gl),
                            );
                        },
                    );
                    ui.allocate_ui_with_layout(
                        [w2, 45.].into(),
                        Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.label(s.data.locale.get("easing"));
                            let old = spect.easing;
                            ui.allocate_ui_with_layout(
                                [w - w3 - 12.5, 22.].into(),
                                Layout::left_to_right(egui::Align::Max),
                                |ui| {
                                    egui::ComboBox::from_id_salt("spectrogram-easing")
                                        .selected_text(spect.easing.display_name())
                                        .width(w - w3 - 12.5)
                                        .wrap_mode(egui::TextWrapMode::Truncate)
                                        .show_ui(ui, |ui| {
                                            for (name, easing) in Easing::iter_all() {
                                                ui.selectable_value(
                                                    &mut spect.easing,
                                                    easing,
                                                    name,
                                                );
                                            }
                                        })
                                },
                            );
                            if old != spect.easing {
                                let mut old_s = spect.clone();
                                old_s.easing = old;
                                s2.add_history(editor::HistoryEntry::Spectrogram(Some(old_s)));
                                s2.rebuild_meshes(gl);
                            }
                        },
                    );
                });

                let mut add_plane = false;
                let mut remove_plane = false;
                match spect.mirror.as_mut() {
                    Some(v4) => {
                        remove_plane = ui
                            .with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                                let clicked = ui.small_button(SMALL_X).clicked();
                                ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                                    ui.label(s.data.locale.get("mirror-geometry-plane"));
                                });
                                clicked
                            })
                            .inner;
                        let (a, b) = ui
                            .horizontal(|ui| {
                                let a = ui
                                    .allocate_ui_with_layout(
                                        [w2, 45.].into(),
                                        Layout::top_down(egui::Align::Min),
                                        |ui| {
                                            ui.label("X");
                                            ui.add_sized(
                                                [w2, 20.],
                                                egui::DragValue::new(&mut v4.x)
                                                    .speed(0.001)
                                                    .range(-1..=1),
                                            )
                                        },
                                    )
                                    .inner;
                                let b = ui
                                    .allocate_ui_with_layout(
                                        [w2, 45.].into(),
                                        Layout::top_down(egui::Align::Min),
                                        |ui| {
                                            ui.label("Y");
                                            ui.add_sized(
                                                [w2, 20.],
                                                egui::DragValue::new(&mut v4.y)
                                                    .speed(0.001)
                                                    .range(-1..=1),
                                            )
                                        },
                                    )
                                    .inner;
                                (a, b)
                            })
                            .inner;
                        let (c, d) = ui
                            .horizontal(|ui| {
                                let c = ui
                                    .allocate_ui_with_layout(
                                        [w2, 45.].into(),
                                        Layout::top_down(egui::Align::Min),
                                        |ui| {
                                            ui.label("Z");
                                            ui.add_sized(
                                                [w2, 20.],
                                                egui::DragValue::new(&mut v4.z)
                                                    .speed(0.001)
                                                    .range(-1..=1),
                                            )
                                        },
                                    )
                                    .inner;
                                let d = ui
                                    .allocate_ui_with_layout(
                                        [w2, 45.].into(),
                                        Layout::top_down(egui::Align::Min),
                                        |ui| {
                                            ui.label(s.data.locale.get("offset"));
                                            ui.add_sized(
                                                [w2, 20.],
                                                egui::DragValue::new(&mut v4.w)
                                                    .speed(0.001)
                                                    .range(-1..=1),
                                            )
                                        },
                                    )
                                    .inner;
                                (c, d)
                            })
                            .inner;

                        trigger_history(
                            ui,
                            &[a, b, c, d],
                            || spect2.clone(),
                            |x| s2.add_history(editor::HistoryEntry::Spectrogram(Some(x))),
                            || s3.rebuild_meshes(gl),
                        );

                        if ui
                            .add_sized(
                                [w, 20.],
                                egui::Button::new(s.data.locale.get("normalize-vector-button")),
                            )
                            .clicked()
                        {
                            s2.add_history(editor::HistoryEntry::Spectrogram(Some(spect2.clone())));
                            *v4 = v4.xyz().normalize().extend(v4.w);
                            s2.rebuild_meshes(gl);
                        }
                    }
                    None => {
                        add_plane = ui
                            .add_sized(
                                [w, 20.],
                                egui::Button::new(s.data.locale.get("add-mirror-plane")),
                            )
                            .clicked();
                    }
                }
                if remove_plane {
                    s2.add_history(editor::HistoryEntry::Spectrogram(Some(spect.clone())));
                    spect.mirror = None;
                    s2.rebuild_meshes(gl);
                }
                if add_plane {
                    s2.add_history(editor::HistoryEntry::Spectrogram(Some(spect.clone())));
                    spect.mirror = Some(Vec4::new(1., 0., 0., 0.));
                    s2.rebuild_meshes(gl);
                }
            }
        }
        None => {
            if ui
                .add_sized(
                    [w, 20.],
                    egui::Button::new(s.data.locale.get("add-spectrogram")),
                )
                .clicked()
            {
                s.add_history(editor::HistoryEntry::Spectrogram(None));
                s.view.spectrogram = Some(SpectrogramData::default())
            }
        }
    }
    if remove_spect {
        let spect = s.view.spectrogram.take();
        s.add_history(editor::HistoryEntry::Spectrogram(spect));
    }

    ui.separator();

    let mut remove_mirror = false;
    match s.view.mirror_id.as_mut() {
        Some(id) => {
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    [w, 20.].into(),
                    Layout::right_to_left(egui::Align::Min),
                    |ui| {
                        if ui
                            .add_sized(
                                [w2, 20.],
                                egui::Button::new(s.data.locale.get("button-edit"))
                                    .selected(s2.state.ui.show_mirror_window),
                            )
                            .clicked()
                        {
                            s2.state.ui.show_mirror_window = !s2.state.ui.show_mirror_window;
                        }
                        ui.allocate_ui_with_layout(
                            [w2, 20.].into(),
                            Layout::left_to_right(egui::Align::Min),
                            |ui| {
                                ui.label(s.data.locale.get("mirror"));
                            },
                        );
                    },
                );
            });
            ui.horizontal(|ui| {
                ui.add_sized([w - 50., 20.], egui::TextEdit::singleline(id));
                if ui
                    .small_button(if s2.view.mirror_path.is_some() {
                        "R"
                    } else {
                        "?"
                    })
                    .clicked()
                {
                    let (sx, rx) = mpsc::channel();
                    let title = s.data.locale.get("open-mirror-file").to_string();
                    std::thread::spawn(move || {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title(title)
                            .add_filter("json", &["json"])
                            .pick_file()
                        {
                            let _ = sx.send(path);
                        }
                    });
                    s2.add_routine(Box::new(move |s, gl| match rx.try_recv() {
                        Err(mpsc::TryRecvError::Empty) => RoutineAction::None,
                        Err(mpsc::TryRecvError::Disconnected) => RoutineAction::Remove,
                        Ok(src) => {
                            if let Err(e) = s.load_mirror(src.as_path(), gl) {
                                let st = s.data.locale.get("failed-to-load-mirror").to_string();
                                s.set_status(None, st, 2.);
                                eprintln!("Failed to load mirror geometry: {e}");
                            } else {
                                s.view.mirror_path = Some(src);
                                s.rebuild_meshes(gl);
                            }
                            RoutineAction::Remove
                        }
                    }));
                }
                remove_mirror = ui.small_button(SMALL_X).clicked();
            });
        }
        None => {
            if ui
                .add_sized([w, 20.], egui::Button::new(s.data.locale.get("add-mirror")))
                .clicked()
            {
                s.add_history(editor::HistoryEntry::Mirror(
                    s.view.mirror_id.clone(),
                    s.view.mirror_path.clone(),
                    s.view.mirror_geometry.clone(),
                ));
                s.view.mirror_id = Some("env:mirror".to_string());
            }
        }
    }
    if remove_mirror {
        let id = s.view.mirror_id.take();
        let path = s.view.mirror_path.take();
        s.add_history(editor::HistoryEntry::Mirror(
            id,
            path,
            s.view.mirror_geometry.clone(),
        ));
        if let Some(mesh) = s.render.mirror.take() {
            mesh.destroy(gl);
        }
        return;
    }

    ui.separator();

    let mut remove_heights = false;
    match s.view.fog_heights.as_mut() {
        Some(heights) => {
            ui.label(s.data.locale.get("fog-heights"));
            ui.horizontal(|ui| {
                let [low, high] = *heights;
                let r0 = ui.add_sized(
                    [w2 - 12.5, 20.],
                    egui::DragValue::new(&mut heights[0])
                        .range(-500f32..=high)
                        .speed(1.),
                );
                let r1 = ui.add_sized(
                    [w2 - 12.5, 20.],
                    egui::DragValue::new(&mut heights[1])
                        .range(low..=500f32)
                        .speed(1.),
                );
                trigger_history(
                    ui,
                    &[r0, r1],
                    || s2.view.fog_heights,
                    |x| s3.add_history(editor::HistoryEntry::FogHeights(x)),
                    || (), // Intentional no-op
                );
                remove_heights = ui.small_button(SMALL_X).clicked();
            });
        }
        None => {
            if ui
                .add_sized(
                    [w, 20.],
                    egui::Button::new(s.data.locale.get("add-fog-heights")),
                )
                .clicked()
            {
                let h = s.view.fog_heights.replace([-50., -30.]);
                s.add_history(editor::HistoryEntry::FogHeights(h));
            }
        }
    }
    if remove_heights {
        let h = s.view.fog_heights.take();
        s.add_history(editor::HistoryEntry::FogHeights(h));
    }

    ui.separator();

    if s.view.session.is_some()
        && ui
            .add_sized(
                [w, 20.],
                egui::Button::new(s.data.locale.get("close-environment")),
            )
            .clicked()
    {
        close_environment(s, gl);
    }
}

pub fn draw_view_right(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
    let rd = RefDuper;
    let s2 = unsafe { rd.detach_mut_ref(s) };
    let s3 = unsafe { rd.detach_mut_ref(s) };
    if let Some(sel) = s.state.ui.view_mesh.as_deref()
        && let Some(Some(mesh)) = s.view.meshes.get_mut(sel)
    {
        let w = ui.available_width();

        ui.add_space(5.);

        let mut renamed = None;
        let rename = s.data.locale.get_with_args(
            "rename-asset",
            &[(Cow::Borrowed("name"), FluentValue::String(sel.into()))].into(),
        );
        ui.add_sized([w, 20.], TextInput::new(&rename, &mut renamed));
        if let Some(new_name) = renamed {
            let mut meshes = std::mem::take(&mut s2.view.meshes);
            for (id, _) in meshes.iter_mut2() {
                if *id == sel {
                    *id = new_name.clone();
                }
            }
            s2.view.meshes = rehash(meshes);
            s.state.ui.view_mesh = Some(new_name);
            return;
        }

        ui.add_space(5.);

        let rd2 = RefDuper;
        let mesh2 = unsafe { rd2.detach_mut_ref(mesh) };

        let mut to_remove = None;
        for (p_i, placement) in mesh.view_placements.iter_mut().enumerate() {
            let collapsed = s.state.ui.collapsed.entry(sel.to_string()).or_default();
            if collapsed.len() <= p_i {
                collapsed.push(false);
            }
            let is_collapsed = &mut collapsed[p_i];

            ui.horizontal(|ui| {
                let icon = if *is_collapsed { R_ARROW } else { D_ARROW };
                if ui.button(icon).clicked() {
                    *is_collapsed = !*is_collapsed;
                }
                let placement = s.data.locale.get_with_args(
                    "placement-counter",
                    &[(Cow::Borrowed("index"), (p_i + 1).into())].into(),
                );
                ui.label(placement);
            });

            if !*is_collapsed {
                let w2 = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
                let w3 = (ui.available_width() - ui.spacing().item_spacing.x * 2.0) / 3.0;

                let modes = s
                    .state
                    .ui
                    .view_rotation_modes
                    .entry(sel.to_string())
                    .or_default();
                if modes.len() <= p_i {
                    modes.push(Default::default());
                }
                let [ori_mode, rot_mode, ori_off_mode, off_mode] =
                    &mut s.state.ui.view_rotation_modes.get_mut(sel).unwrap()[p_i];

                ui.label(s.data.locale.get("position"));
                vec3_row(
                    ui,
                    &mut placement.position,
                    w3,
                    || mesh2.view_placements.clone(),
                    |t| {
                        s2.add_history(editor::HistoryEntry::ViewPlacement(
                            editor::ViewPlacementsSnapshot {
                                id: sel.to_string(),
                                placements: t,
                            },
                        ))
                    },
                    || s3.rebuild_meshes(gl),
                );

                ui.horizontal(|ui| {
                    ui.label(s.data.locale.get("orientation"));

                    ui.add(egui::Label::new(
                        egui::RichText::new("?")
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    ))
                    .on_hover_text(s.data.locale.get("orientation-desc"));
                });
                quat_row(
                    ui,
                    &mut placement.orientation,
                    ori_mode,
                    (w2, w3),
                    || mesh2.view_placements.clone(),
                    |t| {
                        s2.add_history(editor::HistoryEntry::ViewPlacement(
                            editor::ViewPlacementsSnapshot {
                                id: sel.to_string(),
                                placements: t,
                            },
                        ))
                    },
                    || s3.rebuild_meshes(gl),
                );

                ui.horizontal(|ui| {
                    ui.label(s.data.locale.get("rotation"));

                    ui.add(egui::Label::new(
                        egui::RichText::new("?")
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    ))
                    .on_hover_text(s.data.locale.get("rotation-desc"));
                });
                quat_row(
                    ui,
                    &mut placement.rotation,
                    rot_mode,
                    (w2, w3),
                    || mesh2.view_placements.clone(),
                    |t| {
                        s2.add_history(editor::HistoryEntry::ViewPlacement(
                            editor::ViewPlacementsSnapshot {
                                id: sel.to_string(),
                                placements: t,
                            },
                        ))
                    },
                    || s3.rebuild_meshes(gl),
                );

                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(w3, 20.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.add(
                                egui::DragValue::new(&mut placement.count)
                                    .speed(0.25)
                                    .range(1..=u32::MAX),
                            );
                        },
                    );
                    ui.label(s.data.locale.get("item-count"));
                });

                ui.label(s.data.locale.get("offset-position"));
                vec3_row(
                    ui,
                    &mut placement.offset,
                    w3,
                    || mesh2.view_placements.clone(),
                    |t| {
                        s2.add_history(editor::HistoryEntry::ViewPlacement(
                            editor::ViewPlacementsSnapshot {
                                id: sel.to_string(),
                                placements: t,
                            },
                        ))
                    },
                    || s3.rebuild_meshes(gl),
                );

                ui.label(s.data.locale.get("offset-orientation"));
                quat_row(
                    ui,
                    &mut placement.orientation_offset,
                    ori_off_mode,
                    (w2, w3),
                    || mesh2.view_placements.clone(),
                    |t| {
                        s2.add_history(editor::HistoryEntry::ViewPlacement(
                            editor::ViewPlacementsSnapshot {
                                id: sel.to_string(),
                                placements: t,
                            },
                        ))
                    },
                    || s3.rebuild_meshes(gl),
                );

                ui.label(s.data.locale.get("offset-rotation"));
                quat_row(
                    ui,
                    &mut placement.rotation_offset,
                    off_mode,
                    (w2, w3),
                    || mesh2.view_placements.clone(),
                    |t| {
                        s2.add_history(editor::HistoryEntry::ViewPlacement(
                            editor::ViewPlacementsSnapshot {
                                id: sel.to_string(),
                                placements: t,
                            },
                        ))
                    },
                    || s3.rebuild_meshes(gl),
                );

                ui.label(s.data.locale.get("id-label"));
                ui.horizontal(|ui| {
                    let layout = Layout::left_to_right(egui::Align::Min);
                    ui.allocate_ui_with_layout([w2, 20.].into(), layout, |ui| {
                        ui.set_width(w2);
                        ui.label("ID group")
                    });
                    ui.allocate_ui_with_layout([w2 / 2. - 15., 20.].into(), layout, |ui| {
                        ui.set_width(w2 / 2. - 15.);
                        ui.label("ID")
                    });
                    ui.allocate_ui_with_layout([w2 / 2. - 15., 20.].into(), layout, |ui| {
                        ui.set_width(w2 / 2. - 15.);
                        ui.label("Step")
                    });
                });
                let mut first_none = true;
                let mut add_entry = false;
                let mut pop_entry = false;
                let ids_len = placement.ids.len();
                for (idx, (entry, step)) in placement
                    .ids
                    .list_mut()
                    .iter_mut()
                    .zip(placement.id_step.list_mut())
                    .enumerate()
                {
                    match (first_none, entry) {
                        (_, Some((group, id))) => {
                            ui.horizontal(|ui| {
                                let mut g2 = *group;
                                egui::ComboBox::from_id_salt(format!("{sel}-ids-{idx}-{p_i}"))
                                    .selected_text(g2.name())
                                    .width(w2)
                                    .show_ui(ui, |ui| {
                                        for g in LightGroup::iter_all() {
                                            ui.selectable_value(&mut g2, g, g.name());
                                        }
                                    });
                                if g2 != *group {
                                    s2.add_history(editor::HistoryEntry::ViewPlacement(
                                        editor::ViewPlacementsSnapshot {
                                            id: sel.to_string(),
                                            placements: mesh2.view_placements.clone(),
                                        },
                                    ));
                                    *group = g2;
                                }
                                let mut id2 = *id;
                                let resp = ui.add_sized(
                                    [(w2 / 2.) - 15., 20.],
                                    egui::DragValue::new(&mut id2).speed(0.25).range(1..=500),
                                );

                                match step.as_mut() {
                                    Some(step) if placement.count > 1 => {
                                        let mut step2 = *step;
                                        let resp = ui.add_sized(
                                            [w2 / 2. - 15., 20.],
                                            egui::DragValue::new(&mut step2)
                                                .speed(0.25)
                                                .range(-500..=500),
                                        );
                                        trigger_history(
                                            ui,
                                            &[resp],
                                            || mesh2.view_placements.clone(),
                                            |x| {
                                                s2.add_history(editor::HistoryEntry::ViewPlacement(
                                                    editor::ViewPlacementsSnapshot {
                                                        id: sel.to_string(),
                                                        placements: x,
                                                    },
                                                ))
                                            },
                                            || s3.rebuild_meshes(gl),
                                        );
                                        *step = step2;
                                    }
                                    _ => {
                                        ui.add_sized([w2 / 2. - 15., 20.], egui::Label::new("--"));
                                    }
                                }
                                trigger_history(
                                    ui,
                                    &[resp],
                                    || mesh2.view_placements.clone(),
                                    |x| {
                                        s2.add_history(editor::HistoryEntry::ViewPlacement(
                                            editor::ViewPlacementsSnapshot {
                                                id: sel.to_string(),
                                                placements: x,
                                            },
                                        ))
                                    },
                                    || s3.rebuild_meshes(gl),
                                );
                                *id = id2;
                                if ids_len - 1 == idx {
                                    pop_entry = ui.small_button(SMALL_X).clicked();
                                }
                            });
                        }
                        (true, None) => {
                            add_entry = ui
                                .add_sized([w, 20.], egui::Button::new(s.data.locale.get("add-id")))
                                .clicked();
                            first_none = false;
                        }
                        (false, None) => {
                            ui.add_sized([w, 20.], egui::Label::new("---"));
                        }
                    }
                }
                if add_entry {
                    placement
                        .ids
                        .push((data::mesh::LightGroup::CenterLasers, 0));
                    placement.id_step.push(0);
                }
                if pop_entry {
                    let _ = placement.ids.pop();
                    let _ = placement.id_step.pop();
                }

                ui.label(s.data.locale.get("action-group-settings"));
                let mut repl = None;
                match &mut placement.action_type {
                    editor::ActionType::Spinning { side, axis } => {
                        let mut current_side = *side;
                        egui::ComboBox::from_id_salt(format!("{p_i}-{sel}-spin-action"))
                            .width(w)
                            .selected_text(current_side.name())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut current_side,
                                    SpinSide::Left,
                                    SpinSide::Left.name(),
                                );
                                ui.selectable_value(
                                    &mut current_side,
                                    SpinSide::Right,
                                    SpinSide::Right.name(),
                                );
                            });
                        if current_side != *side {
                            s2.add_history(editor::HistoryEntry::ViewPlacement(
                                editor::ViewPlacementsSnapshot {
                                    id: sel.to_string(),
                                    placements: mesh2.view_placements.clone(),
                                },
                            ));
                            *side = current_side;
                        }

                        ui.allocate_ui_with_layout(
                            [w, 20.].into(),
                            Layout::right_to_left(egui::Align::Min),
                            |ui| {
                                if axis.is_some() && ui.small_button(SMALL_X).clicked() {
                                    *axis = None;
                                }
                                ui.allocate_ui_with_layout(
                                    [w - if axis.is_some() { 25. } else { 0. }, 20.].into(),
                                    Layout::left_to_right(egui::Align::Min),
                                    |ui| ui.label(s.data.locale.get("spin-axis")),
                                );
                            },
                        );

                        let mut set_axis = false;
                        match axis.as_mut() {
                            None => {
                                set_axis = ui
                                    .add_sized(
                                        [w, 20.],
                                        egui::Button::new(s.data.locale.get("set-spin-axis")),
                                    )
                                    .clicked();
                            }
                            Some(axs) => {
                                vec3_row(
                                    ui,
                                    axs,
                                    w3,
                                    || mesh2.view_placements.clone(),
                                    |x| {
                                        s2.add_history(editor::HistoryEntry::ViewPlacement(
                                            editor::ViewPlacementsSnapshot {
                                                id: sel.to_string(),
                                                placements: x,
                                            },
                                        ))
                                    },
                                    || s3.rebuild_meshes(gl),
                                );
                            }
                        }
                        if set_axis {
                            *axis = Some(Vec3::Y);
                        }

                        ui.add_space(5.);

                        if ui
                            .add_sized(
                                [w, 20.],
                                egui::Button::new(s.data.locale.get("remove-action")),
                            )
                            .clicked()
                        {
                            repl = Some(ActionType::Static);
                        }
                    }
                    editor::ActionType::Ring {
                        layer,
                        angles,
                        deltas,
                        start,
                    } => {
                        let mut current_layer = *layer;
                        egui::ComboBox::from_id_salt(format!("{p_i}-{sel}-ring-action"))
                            .width(w)
                            .selected_text(current_layer.name())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut current_layer,
                                    RingType::Inner,
                                    RingType::Inner.name(),
                                );
                                ui.selectable_value(
                                    &mut current_layer,
                                    RingType::Outer,
                                    RingType::Outer.name(),
                                );
                            });

                        if current_layer != *layer {
                            s2.add_history(editor::HistoryEntry::ViewPlacement(
                                editor::ViewPlacementsSnapshot {
                                    id: sel.to_string(),
                                    placements: mesh2.view_placements.clone(),
                                },
                            ));
                            *layer = current_layer;
                        }

                        ui.allocate_ui_with_layout(
                            [w, 20.].into(),
                            Layout::right_to_left(egui::Align::Min),
                            |ui| {
                                if start.is_some() && ui.small_button(SMALL_X).clicked() {
                                    *start = None;
                                }
                                ui.allocate_ui_with_layout(
                                    [w - if start.is_some() { 25. } else { 0. }, 20.].into(),
                                    Layout::left_to_right(egui::Align::Min),
                                    |ui| ui.label(s.data.locale.get("default-positions")),
                                );
                            },
                        );

                        let mut add_start = false;
                        match start.as_mut() {
                            Some([in_angle, in_offset, out_angle, out_offset]) => {
                                let (resp, vals) = ui
                                    .horizontal(|ui| {
                                        let (a, b, ia, io) = ui
                                            .vertical(|ui| {
                                                ui.label(s.data.locale.get("inner-ring-angle"));
                                                let mut ia = *in_angle;
                                                let a = ui.add_sized(
                                                    [w2, 20.],
                                                    egui::DragValue::new(&mut ia)
                                                        .speed(1.)
                                                        .range(-360..=360),
                                                );

                                                ui.label(s.data.locale.get("inner-ring-offset"));
                                                let mut io = *in_offset;
                                                let b = ui.add_sized(
                                                    [w2, 20.],
                                                    egui::DragValue::new(&mut io)
                                                        .speed(1.)
                                                        .range(-360..=360),
                                                );

                                                (a, b, ia, io)
                                            })
                                            .inner;
                                        let (c, d, oa, oo) = ui
                                            .vertical(|ui| {
                                                ui.label(s.data.locale.get("outer-ring-angle"));
                                                let mut ia = *out_angle;
                                                let a = ui.add_sized(
                                                    [w2, 20.],
                                                    egui::DragValue::new(&mut ia)
                                                        .speed(1.)
                                                        .range(-360..=360),
                                                );

                                                ui.label(s.data.locale.get("outer-ring-offset"));
                                                let mut io = *out_offset;
                                                let b = ui.add_sized(
                                                    [w2, 20.],
                                                    egui::DragValue::new(&mut io)
                                                        .speed(1.)
                                                        .range(-360..=360),
                                                );

                                                (a, b, ia, io)
                                            })
                                            .inner;
                                        ([a, b, c, d], (ia, io, oa, oo))
                                    })
                                    .inner;
                                trigger_history(
                                    ui,
                                    &resp,
                                    || mesh2.view_placements.clone(),
                                    |x| {
                                        s2.add_history(editor::HistoryEntry::ViewPlacement(
                                            editor::ViewPlacementsSnapshot {
                                                id: sel.to_string(),
                                                placements: x,
                                            },
                                        ))
                                    },
                                    || s3.rebuild_meshes(gl),
                                );
                                *in_angle = vals.0;
                                *in_offset = vals.1;
                                *out_angle = vals.2;
                                *out_offset = vals.3;
                            }
                            None => {
                                add_start = ui
                                    .add_sized(
                                        [w, 20.],
                                        egui::Button::new(
                                            s.data.locale.get("add-default-positions"),
                                        ),
                                    )
                                    .clicked()
                            }
                        }
                        if add_start {
                            *start = Some([0., 0., 0., 0.]);
                        }

                        ui.allocate_ui_with_layout(
                            [w, 20.].into(),
                            Layout::left_to_right(egui::Align::Min),
                            |ui| {
                                ui.allocate_ui_with_layout(
                                    [w2, 0.].into(),
                                    Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.label(s.data.locale.get("ring-angles"))
                                            .on_hover_text(s.data.locale.get("ring-angles-desc"));

                                        let mut to_remove = None;
                                        for (i, angle) in angles.iter_mut().enumerate() {
                                            let mut current_angle = *angle;

                                            ui.horizontal(|ui| {
                                                let resp = ui.add_sized(
                                                    [w2 - 25., 20.],
                                                    egui::DragValue::new(&mut current_angle)
                                                        .speed(1.)
                                                        .range(-360..=360),
                                                );
                                                trigger_history(
                                                    ui,
                                                    &[resp],
                                                    || mesh2.view_placements.clone(),
                                                    |x| {
                                                        s2.add_history(
                                                            editor::HistoryEntry::ViewPlacement(
                                                                editor::ViewPlacementsSnapshot {
                                                                    id: sel.to_string(),
                                                                    placements: x,
                                                                },
                                                            ),
                                                        )
                                                    },
                                                    || s3.rebuild_meshes(gl),
                                                );
                                                if current_angle != *angle {
                                                    *angle = current_angle;
                                                }
                                                if ui.small_button(SMALL_X).clicked() {
                                                    to_remove = Some(i)
                                                }
                                            });
                                        }
                                        if let Some(i) = to_remove {
                                            angles.remove(i);
                                        }
                                        if ui
                                            .add_sized(
                                                [w2, 20.],
                                                egui::Button::new(s.data.locale.get("add")),
                                            )
                                            .clicked()
                                        {
                                            angles.push(0.);
                                        }
                                    },
                                );
                                ui.allocate_ui_with_layout(
                                    [w2, 0.].into(),
                                    Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.label(s.data.locale.get("ring-deltas"))
                                            .on_hover_text(s.data.locale.get("ting-deltas-desc"));

                                        let mut to_remove = None;
                                        for (i, angle) in deltas.iter_mut().enumerate() {
                                            let mut current_angle = *angle;

                                            ui.horizontal(|ui| {
                                                let resp = ui.add_sized(
                                                    [w2 - 25., 20.],
                                                    egui::DragValue::new(&mut current_angle)
                                                        .speed(1.)
                                                        .range(-360..=360),
                                                );
                                                trigger_history(
                                                    ui,
                                                    &[resp],
                                                    || mesh2.view_placements.clone(),
                                                    |x| {
                                                        s2.add_history(
                                                            editor::HistoryEntry::ViewPlacement(
                                                                editor::ViewPlacementsSnapshot {
                                                                    id: sel.to_string(),
                                                                    placements: x,
                                                                },
                                                            ),
                                                        )
                                                    },
                                                    || s3.rebuild_meshes(gl),
                                                );
                                                if current_angle != *angle {
                                                    *angle = current_angle;
                                                }
                                                if ui.small_button(SMALL_X).clicked() {
                                                    to_remove = Some(i)
                                                }
                                            });
                                        }
                                        if let Some(i) = to_remove {
                                            deltas.remove(i);
                                        }
                                        if ui
                                            .add_sized(
                                                [w2, 20.],
                                                egui::Button::new(s.data.locale.get("add")),
                                            )
                                            .clicked()
                                        {
                                            deltas.push(0.);
                                        }
                                    },
                                );
                            },
                        );

                        ui.add_space(5.);

                        if ui
                            .add_sized(
                                [w, 20.],
                                egui::Button::new(s.data.locale.get("remove-action")),
                            )
                            .clicked()
                        {
                            repl = Some(ActionType::Static);
                        }
                    }
                    editor::ActionType::Static => {
                        ui.horizontal(|ui| {
                            if ui
                                .add_sized(
                                    [w2, 20.],
                                    egui::Button::new(s.data.locale.get("add-spinning")),
                                )
                                .clicked()
                            {
                                repl = Some(ActionType::Spinning {
                                    side: editor::SpinSide::Left,
                                    axis: None,
                                })
                            }
                            if ui
                                .add_sized(
                                    [w2, 20.],
                                    egui::Button::new(s.data.locale.get("add-rings")),
                                )
                                .clicked()
                            {
                                repl = Some(ActionType::Ring {
                                    layer: editor::RingType::Inner,
                                    angles: Vec::new(),
                                    deltas: Vec::new(),
                                    start: None,
                                })
                            }
                        });
                    }
                }
                if let Some(repl) = repl {
                    s2.add_history(editor::HistoryEntry::ViewPlacement(
                        editor::ViewPlacementsSnapshot {
                            id: sel.to_string(),
                            placements: mesh2.view_placements.clone(),
                        },
                    ));
                    placement.action_type = repl;
                }

                ui.add_space(5.);

                if ui
                    .add_sized(
                        [ui.available_width(), 20.0],
                        egui::Button::new(s.data.locale.get("delete")),
                    )
                    .clicked()
                {
                    to_remove = Some(p_i);
                }
            }

            ui.separator();
        }

        if let Some(rem) = to_remove {
            mesh.view_placements.remove(rem);
            if let Some(collapsed) = s.state.ui.collapsed.get_mut(sel) {
                collapsed.remove(rem);
            }
            if let Some(modes) = s.state.ui.view_rotation_modes.get_mut(sel) {
                modes.remove(rem);
            }
        }

        if ui
            .add_sized(
                [w, 20.],
                egui::Button::new(s.data.locale.get("add-Placement")),
            )
            .clicked()
        {
            mesh.view_placements.push(ViewPlacement::default());
            s.state
                .ui
                .collapsed
                .entry(sel.to_string())
                .or_default()
                .push(false);
            s.state
                .ui
                .view_rotation_modes
                .entry(sel.to_string())
                .or_default()
                .push(Default::default());
            s2.rebuild_meshes(gl);
        }
    }
}

pub(crate) fn draw_view_gl(
    s: &UnsafeMutRef<App>,
    gl: &glow::Context,
    view: &Mat4,
    proj: &Mat4,
    window: (i32, i32),
) {
    let mut calls = Vec::new();
    for (_id, vm) in s.view.meshes.iter() {
        if let Some(vm) = vm {
            let mut draws = Vec::new();
            if let Some(mesh) = vm.render_view_placements(&mut draws) {
                calls.push(MeshDrawCall {
                    mesh,
                    instances: draws,
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
                })
            }
        }
    }
    if let Some(towers) = s.render.spectrogram.as_ref() {
        calls.push(MeshDrawCall {
            mesh: towers,
            instances: vec![InstanceData::new(Vec4::ZERO, Mat4::IDENTITY, LIGHT_COLORS)],
            wireframe: s.state.wireframe,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: false,
            mirror: true,
            obstacle: false,
            highlight: false,
        })
    }
    match s.state.view_style {
        editor::ViewStyle::Edit => {
            s.ref_mut().render.renderer.draw_meshes(
                gl,
                view,
                proj,
                &calls,
                s.render.mirror.as_ref(),
                s.state.wireframe,
                true,
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
                true,
                if blackout_sky {
                    (0., 0., 0., 1.)
                } else {
                    (0.07, 0.08, 0.11, 1.)
                },
            );
        }
    }
}
