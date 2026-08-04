

use std::collections::HashMap;
use std::hash::Hash;

use eframe::glow;
use egui::Ui;
use glam::{Quat, Vec2, Vec3};

use crate::data::VertexId;
use crate::easing::Easing;
use crate::editor::{self, App, RotationDisplayMode};
use crate::light_mesh::{self, ComputeVertex, Part};
use crate::widgets::{MathDragValue, MathDragValueOpt, MultiMathValue};

use crate::RefDuper;


pub fn trigger_history<T: 'static + Clone + Send + Sync>(
    ui: &mut egui::Ui,
    resp: &[egui::Response],
    snapshot_provider: impl Fn() -> T,
    mut history_pusher: impl FnMut(T),
    mut on_change: impl FnMut(),
) {
    let mut changed = false;
    for resp in resp {
        let id = ui.next_auto_id();
        if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
            ui.memory_mut(|m| {
                m.data.insert_temp(id, snapshot_provider());
            });
            changed = true;
        }
        if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
            && let Some(t) = ui.memory_mut(|m| {
                let t = m.data.get_temp::<T>(id)?;
                Some(t)
            })
        {
            changed = true;
            history_pusher(t);
        }
    }
    if changed {
        on_change();
    }
}

pub fn vec3_row<T: 'static + Clone + Send + Sync>(
    ui: &mut egui::Ui,
    v: &mut Vec3,
    w3: f32,
    snapshot_provider: impl Fn() -> T,
    mut history_pusher: impl FnMut(T),
    mut on_change: impl FnMut(),
) {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), v.x);
    vars.insert("y".to_string(), v.y);
    vars.insert("z".to_string(), v.z);
    let mut changed = false;
    ui.horizontal(|ui| {
        let mut current = *v;
        for val in current.as_mut() {
            ui.allocate_ui_with_layout(
                (w3, 20.).into(),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let id = ui.next_auto_id();

                    let (rect, _) = ui.allocate_exact_size([w3, 20.].into(), egui::Sense::empty());

                    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(*ui.layout()));
                    child_ui.set_max_width(w3);

                    let resp = child_ui.add(
                        MathDragValue::new(val, &mut vars)
                            .speed(0.01)
                            .max_decimals(3),
                    );

                    changed |= resp.changed();

                    if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                        ui.memory_mut(|m| {
                            m.data.insert_temp(id, *v);
                            m.data.insert_temp(id, snapshot_provider());
                        });
                        changed = true;
                    }
                    if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                        && let Some((old, t)) = ui.memory_mut(|m| {
                            let o = m.data.get_temp::<Vec3>(id)?;
                            let t = m.data.get_temp::<T>(id)?;
                            Some((o, t))
                        })
                        && old != *v
                    {
                        changed = true;
                        history_pusher(t);
                    }
                },
            );
        }
        *v = current;
    });
    if changed {
        on_change();
    }
}

pub fn vec2_row<T: 'static + Clone + Send + Sync>(
    ui: &mut egui::Ui,
    v: &mut Vec2,
    w2: f32,
    snapshot_provider: impl Fn() -> T,
    mut history_pusher: impl FnMut(T),
    mut on_change: impl FnMut(),
) {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), v.x);
    vars.insert("y".to_string(), v.y);
    let mut changed = false;
    ui.horizontal(|ui| {
        let mut current = *v;
        for val in current.as_mut() {
            ui.allocate_ui_with_layout(
                (w2, 20.).into(),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let id = ui.next_auto_id();


                    let (rect, _) = ui.allocate_exact_size([w2, 20.].into(), egui::Sense::empty());

                    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(*ui.layout()));
                    child_ui.set_max_width(w2);

                    let resp = child_ui.add(
                        MathDragValue::new(val, &mut vars)
                            .speed(0.01)
                            .max_decimals(3),
                    );

                    changed |= resp.changed();
                    if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                        ui.memory_mut(|m| {
                            m.data.insert_temp(id, *v);
                            m.data.insert_temp(id, snapshot_provider());
                        });
                        changed = true;
                    }
                    if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                        && let Some((old, t)) = ui.memory_mut(|m| {
                            let o = m.data.get_temp::<Vec2>(id)?;
                            let t = m.data.get_temp::<T>(id)?;
                            Some((o, t))
                        })
                        && old != *v
                    {
                        history_pusher(t);
                    }
                },
            );
        }
        *v = current;
    });
    if changed {
        on_change();
    }
}

pub fn multi_vec3_row<T: 'static + Clone + Send + Sync>(
    ui: &mut Ui,
    vertices: &mut [&mut Vec3],
    w3: f32,
    snapshot_provider: impl Fn() -> T,
    mut history_pusher: impl FnMut(T),
    mut on_change: impl FnMut(),
) {
    let mut vars = Vec::with_capacity(vertices.len());
    for v3 in vertices.iter() {
        let mut v = Box::new(HashMap::with_capacity(3));
        v.insert("x".to_string(), v3.x);
        v.insert("y".into(), v3.y);
        v.insert("z".into(), v3.z);
        let b = Box::into_raw(v);
        vars.push(unsafe { &mut *b });
    }

    let mut current: Vec<_> = vertices.iter().map(|r| **r).collect();
    let mut changed = false;
    fn axis_value(
        ui: &mut Ui,
        v: &'static str,
        vars: &mut [&mut HashMap<String, f32>],
        w3: f32,
        vals: &mut Option<Vec<f32>>,
    ) {
        ui.allocate_ui_with_layout(
            (w3, 20.).into(),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.add_sized(
                    [w3, 20.],
                    MultiMathValue::new(v, vals, vars).max_decimals(3),
                );
            },
        );
    }

    ui.horizontal(|ui| {
        let mut vals = None;
        axis_value(ui, "x", &mut vars, w3, &mut vals);
        if let Some(x) = vals {
            for ((c, vs), x) in current.iter_mut().zip(vars.iter_mut()).zip(x) {
                c.x = x;
                vs.insert("x".into(), x);
            }
            changed = true;
        }
        let mut vals = None;
        axis_value(ui, "y", &mut vars, w3, &mut vals);
        if let Some(y) = vals {
            for ((c, vs), y) in current.iter_mut().zip(vars.iter_mut()).zip(y) {
                c.y = y;
                vs.insert("y".into(), y);
            }
            changed = true;
        }
        vals = None;
        axis_value(ui, "z", &mut vars, w3, &mut vals);
        if let Some(z) = vals {
            for ((c, vs), z) in current.iter_mut().zip(vars.iter_mut()).zip(z) {
                c.z = z;
                vs.insert("z".into(), z);
            }
            changed = true;
        }
    });

    for (v, c) in vertices.iter_mut().zip(current) {
        **v = c;
    }

    for var in vars {
        let _ = unsafe { Box::from_raw(var as *mut _) };
    }
    if changed {
        history_pusher(snapshot_provider());
        on_change();
    }
}

pub fn value_opt_row<T: 'static + Clone + Send + Sync>(
    ui: &mut Ui,
    v: &mut Option<u32>,
    w: f32,
    vars: &mut HashMap<String, f32>,
    snapshot_provider: impl Fn() -> T,
    mut history_pusher: impl FnMut(T),
    mut on_change: impl FnMut(),
) {
    let mut changed = false;
    ui.horizontal(|ui| {
        let mut current = *v;
        ui.allocate_ui_with_layout(
            (w, 20.).into(),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let id = ui.next_auto_id();
                let resp = ui.add_sized(
                    [w, 20.],
                    MathDragValueOpt::<u32, _>::new(&mut current, vars),
                );

                changed |= resp.changed();

                if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                    ui.memory_mut(|m| {
                        m.data.insert_temp(id, *v);
                        m.data.insert_temp(id, snapshot_provider());
                    });
                    changed = true;
                }
                if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                    && let Some((old, t)) = ui.memory_mut(|m| {
                        let o = m.data.get_temp::<Option<u32>>(id)?;
                        let t = m.data.get_temp::<T>(id)?;
                        Some((o, t))
                    })
                    && old != current
                {
                    history_pusher(t);
                }
            },
        );
        *v = current;
        if changed {
            on_change();
        }
    });
}

pub fn vec3_opt_row<T: 'static + Clone + Send + Sync>(
    ui: &mut Ui,
    mut v: [&mut Option<f32>; 3],
    w3: f32,
    vars: &mut HashMap<String, f32>,
    snapshot_provider: impl Fn() -> T,
    mut history_pusher: impl FnMut(T),
    mut on_change: impl FnMut(),
) {
    if let Some(x) = v[0] {
        vars.insert("x".into(), *x);
    }
    if let Some(y) = v[1] {
        vars.insert("y".into(), *y);
    }
    if let Some(z) = v[2] {
        vars.insert("z".into(), *z);
    }
    let mut changed = false;
    ui.horizontal(|ui| {
        let mut current = [*v[0], *v[1], *v[2]];
        for val in current.iter_mut() {
            ui.allocate_ui_with_layout(
                (w3, 20.).into(),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let id = ui.next_auto_id();
                    let resp = ui.add_sized(
                        [w3, 20.],
                        MathDragValueOpt::<f32, _>::new(val, vars).speed(0.01).max_decimals(3),
                    );

                    changed |= resp.changed();

                    if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                        ui.memory_mut(|m| {
                            let c2 = [*v[0], *v[1], *v[2]];
                            m.data.insert_temp(id, c2);
                            m.data.insert_temp(id, snapshot_provider());
                        });
                        changed = true;
                    }
                    if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                        && let Some((old, t)) = ui.memory_mut(|m| {
                            let o = m.data.get_temp::<[Option<f32>; 3]>(id)?;
                            let t = m.data.get_temp::<T>(id)?;
                            Some((o, t))
                        })
                        && (old[0] != *v[0] || old[1] != *v[1] || old[2] != *v[2])
                    {
                        history_pusher(t);
                    }
                },
            );
        }
        *v[0] = current[0];
        *v[1] = current[1];
        *v[2] = current[2];
        if changed {
            on_change();
        }
    });
}

pub fn delta_function_row<T: 'static + Clone + Send + Sync>(
    ui: &mut Ui,
    func_delta_vars: (&mut Easing, &mut Option<f32>, &mut HashMap<String, f32>),
    salt: impl Hash,
    w: (f32, f32),
    snapshot_provider: impl Fn() -> T,
    mut history_pusher: impl FnMut(T),
    mut on_change: impl FnMut(),
) {
    let (func, delta, vars) = func_delta_vars;
    let (w2, w3) = w;
    if let Some(d) = delta {
        vars.insert("d".into(), *d);
    }
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            (w2, 20.).into(),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let id = ui.next_auto_id();
                let resp = ui.add_sized(
                    [w2, 20.],
                    MathDragValueOpt::<f32, _>::new(delta, vars)
                        .speed(0.01)
                        .max_decimals(3),
                );

                changed |= resp.changed();

                if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                    ui.memory_mut(|m| {
                        m.data.insert_temp(id, *delta);
                        m.data.insert_temp(id, snapshot_provider());
                    });
                    changed = true;
                }
                if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                    && let Some((old, t)) = ui.memory_mut(|m| {
                        let o = m.data.get_temp::<Option<f32>>(id)?;
                        let t = m.data.get_temp::<T>(id)?;
                        Some((o, t))
                    })
                    && old == *delta
                {
                    history_pusher(t);
                }
            },
        );
        ui.allocate_ui_with_layout(
            (w3, 20.).into(),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let old = *func;
                egui::ComboBox::from_id_salt(egui::Id::new("delta_function").with(salt))
                    .selected_text(func.display_name())
                    .width(w3)
                    .wrap_mode(egui::TextWrapMode::Truncate)
                    .show_ui(ui, |ui| {
                        for (name, easing) in Easing::iter_all() {
                            ui.selectable_value(func, easing, name);
                        }
                    });
                if old != *func {
                    on_change();
                    history_pusher(snapshot_provider());
                }
            },
        );
    });
    if changed {
        on_change();
    }
}

pub fn quat_row<T: 'static + Clone + Send + Sync>(
    ui: &mut egui::Ui,
    q: &mut Quat,
    mode: &mut RotationDisplayMode,
    w: (f32, f32),
    snapshot_provider: impl Fn() -> T,
    mut history_pusher: impl FnMut(T),
    mut on_change: impl FnMut(),
) {
    let (w2, w3) = w;
    ui.horizontal(|ui| {
        let mode_label = match mode {
            RotationDisplayMode::Quaternion => "QUAT",
            RotationDisplayMode::Euler(s) => s.label(),
        };
        if ui.small_button(mode_label).clicked() {
            *mode = mode.cycle();
        }
    });
    let current = *q;
    let mut changed = false;
    match mode {
        RotationDisplayMode::Quaternion => {
            let mut v = current.to_array();
            let mut vars = HashMap::new();
            vars.insert("x".to_string(), v[0]);
            vars.insert("y".to_string(), v[1]);
            vars.insert("z".to_string(), v[2]);
            vars.insert("w".to_string(), v[3]);
            ui.horizontal(|ui| {
                for val in &mut v[0..2] {
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(w2, 20.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let id = ui.next_auto_id();
                            let resp = ui.add_sized(
                                [w2, 20.],
                                MathDragValue::new(val, &mut vars)
                                    .speed(0.001)
                                    .max_decimals(3),
                            );

                            changed |= resp.changed();

                            if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                                ui.memory_mut(|m| {
                                    m.data.insert_temp(id, *q);
                                    m.data.insert_temp(id, snapshot_provider());
                                });
                            }
                            if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                                && let Some((old, t)) = ui.memory_mut(|m| {
                                    let o = m.data.get_temp::<Quat>(id)?;
                                    let t = m.data.get_temp::<T>(id)?;
                                    Some((o, t))
                                })
                                && old != *q
                            {
                                history_pusher(t);
                            }
                        },
                    );
                }
            });
            ui.horizontal(|ui| {
                for val in &mut v[2..4] {
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(w2, 20.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let id = ui.next_auto_id();
                            let resp = ui.add_sized(
                                [w2, 20.],
                                MathDragValue::new(val, &mut vars)
                                    .speed(0.001)
                                    .max_decimals(3),
                            );

                            changed |= resp.changed();

                            if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                                ui.memory_mut(|m| {
                                    m.data.insert_temp(id, *q);
                                    m.data.insert_temp(id, snapshot_provider());
                                });
                            }
                            if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                                && let Some((old, t)) = ui.memory_mut(|m| {
                                    let o = m.data.get_temp::<Quat>(id)?;
                                    let t = m.data.get_temp::<T>(id)?;
                                    Some((o, t))
                                })
                                && old != *q
                            {
                                history_pusher(t);
                            }
                        },
                    );
                }
            });
            *q = Quat::from_array(v);
        }
        RotationDisplayMode::Euler(swizzle) => {
            let (ax, ay, az) = q.to_euler(swizzle.to_glam());
            let [n1, n2, n3] = swizzle.names();

            let anchor_id = ui.id().with("euler_anchor");

            let raw = [ax.to_degrees(), ay.to_degrees(), az.to_degrees()];

            let normalize_angle = |d: f32| -> f32 {
                let d = if d == -0.0 { 0.0 } else { d };
                let d = ((d + 180.0).rem_euclid(360.0)) - 180.0;
                if (d.abs() - 180.0).abs() < 0.001 { 180.0 } else { d }
            };

            let candidate_a = [
                normalize_angle(raw[0]),
                normalize_angle(raw[1]),
                normalize_angle(raw[2]),
            ];
            let candidate_b = [
                normalize_angle(raw[0] + 180.0),
                normalize_angle(180.0 - raw[1]),
                normalize_angle(raw[2] + 180.0),
            ];

            let anchor: [f32; 3] = ui
                .memory(|m| m.data.get_temp(anchor_id))
                .unwrap_or(candidate_a);

            let dist = |c: &[f32; 3]| -> f32 {
                c.iter().zip(anchor.iter()).map(|(a, b)| {
                    let d = (a - b + 180.0).rem_euclid(360.0) - 180.0;
                    d * d
                }).sum()
            };

            let mut degrees = if dist(&candidate_b) < dist(&candidate_a) {
                candidate_b
            } else {
                candidate_a
            };

            let mut vars = HashMap::new();
            vars.insert(n1.to_string(), degrees[0]);
            vars.insert(n2.to_string(), degrees[1]);
            vars.insert(n3.to_string(), degrees[2]);

            ui.horizontal(|ui| {
                for val in &mut degrees {
                    ui.allocate_ui_with_layout(
                        (w3, 20.).into(),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let id = ui.next_auto_id();
                            let resp = ui.add_sized(
                                [w3, 20.],
                                MathDragValue::new(val, &mut vars)
                                    .speed(0.5)
                                    .max_decimals(1)
                                    .suffix("\u{b0}")
                                    .degrees(),
                            );

                            changed |= resp.changed();

                            if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                                ui.memory_mut(|m| {
                                    m.data.insert_temp(id, *q);
                                    m.data.insert_temp(id, snapshot_provider());
                                });
                            }
                            if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                                && let Some((old, t)) = ui.memory_mut(|m| {
                                    let o = m.data.get_temp::<Quat>(id)?;
                                    let t = m.data.get_temp::<T>(id)?;
                                    Some((o, t))
                                })
                                && old != *q
                            {
                                history_pusher(t);
                            }
                        },
                    );
                }
            });

            ui.memory_mut(|m| m.data.insert_temp(anchor_id, degrees));

            let rot = glam::EulerRot::from(*swizzle);
            *q = Quat::from_euler(
                rot,
                degrees[0].to_radians(),
                degrees[1].to_radians(),
                degrees[2].to_radians(),
            );
        }
    }
    if changed {
        on_change();
    }
}

/*pub fn multi_quat_row<T: 'static + Clone + Send + Sync>(
    ui: &mut egui::Ui,
    quats: &mut [&mut Quat],
    mode: &mut RotationDisplayMode,
    w: (f32, f32),
    snapshot_provider: impl Fn() -> T,
    mut history_pusher: impl FnMut(T),
    mut on_change: impl FnMut(),
) {
    let (w2, w3) = w;

    ui.horizontal(|ui| {
        let mode_label = match mode {
            RotationDisplayMode::Quaternion => "QUAT",
            RotationDisplayMode::Euler(s) => s.label(),
        };
        if ui.small_button(mode_label).clicked() {
            *mode = mode.cycle();
        }
    });

    let mut changed = false;

    match mode {
        RotationDisplayMode::Quaternion => {
            let mut arrays: Vec<[f32; 4]> = quats.iter().map(|q| q.to_array()).collect();

            let mut vars: Vec<Box<HashMap<String, f32>>> = arrays
                .iter()
                .map(|a| {
                    let mut m = Box::new(HashMap::with_capacity(4));
                    m.insert("x".into(), a[0]);
                    m.insert("y".into(), a[1]);
                    m.insert("z".into(), a[2]);
                    m.insert("w".into(), a[3]);
                    m
                })
                .collect();
            let mut var_ptrs: Vec<&mut HashMap<String, f32>> =
                vars.iter_mut().map(|b| b.as_mut()).collect();

            fn component_col(
                ui: &mut egui::Ui,
                axis: &'static str,
                var_ptrs: &mut [&mut HashMap<String, f32>],
                arrays: &mut [[f32; 4]],
                axis_idx: usize,
                w2: f32,
                changed: &mut bool,
            ) {
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(w2, 20.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        let mut vals: Option<Vec<f32>> = None;
                        ui.add_sized([w2, 20.], MultiMathValue::new(axis, &mut vals, var_ptrs));
                        if let Some(new_vals) = vals {
                            for (arr, &v) in arrays.iter_mut().zip(new_vals.iter()) {
                                arr[axis_idx] = v;
                            }
                            // sync vars back
                            for (vp, &v) in var_ptrs.iter_mut().zip(new_vals.iter()) {
                                vp.insert(axis.into(), v);
                            }
                            *changed = true;
                        }
                    },
                );
            }

            // Row 1: x, y
            ui.horizontal(|ui| {
                component_col(ui, "x", &mut var_ptrs, &mut arrays, 0, w2, &mut changed);
                component_col(ui, "y", &mut var_ptrs, &mut arrays, 1, w2, &mut changed);
            });
            // Row 2: z, w
            ui.horizontal(|ui| {
                component_col(ui, "z", &mut var_ptrs, &mut arrays, 2, w2, &mut changed);
                component_col(ui, "w", &mut var_ptrs, &mut arrays, 3, w2, &mut changed);
            });

            for (q, arr) in quats.iter_mut().zip(arrays.into_iter()) {
                **q = Quat::from_array(arr);
            }
        }

        RotationDisplayMode::Euler(swizzle) => {
            let normalize_angle = |d: f32| {
                let d = if d == -0.0 { 0.0 } else { d };
                if (d - 180.0).abs() < 0.001 || (d + 180.0).abs() < 0.001 {
                    180.0
                } else {
                    d
                }
            };

            let [n1, n2, n3] = swizzle.names();
            let glam_rot = glam::EulerRot::from(*swizzle);

            // degrees[i] = [ax, ay, az] for quats[i]
            let mut degrees: Vec<[f32; 3]> = quats
                .iter()
                .map(|q| {
                    let (ax, ay, az) = q.to_euler(swizzle.to_glam());
                    [
                        normalize_angle(ax.to_degrees()),
                        normalize_angle(ay.to_degrees()),
                        normalize_angle(az.to_degrees()),
                    ]
                })
                .collect();

            let mut vars: Vec<Box<HashMap<String, f32>>> = degrees
                .iter()
                .map(|d| {
                    let mut m = Box::new(HashMap::with_capacity(3));
                    m.insert(n1.into(), d[0]);
                    m.insert(n2.into(), d[1]);
                    m.insert(n3.into(), d[2]);
                    m
                })
                .collect();
            let mut var_ptrs: Vec<&mut HashMap<String, f32>> =
                vars.iter_mut().map(|b| b.as_mut()).collect();

            fn euler_col(
                ui: &mut egui::Ui,
                axis: &'static str,
                var_ptrs: &mut [&mut HashMap<String, f32>],
                degrees: &mut [[f32; 3]],
                axis_idx: usize,
                w3: f32,
                changed: &mut bool,
            ) {
                ui.allocate_ui_with_layout(
                    (w3, 20.).into(),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        let mut vals: Option<Vec<f32>> = None;
                        ui.add_sized(
                            [w3, 20.],
                            MultiMathValue::new(axis, &mut vals, var_ptrs)
                                .suffix("\u{b0}")
                                .degrees(),
                        );
                        if let Some(new_vals) = vals {
                            for (deg, &v) in degrees.iter_mut().zip(new_vals.iter()) {
                                deg[axis_idx] = v;
                            }
                            for (vp, &v) in var_ptrs.iter_mut().zip(new_vals.iter()) {
                                vp.insert(axis.into(), v);
                            }
                            *changed = true;
                        }
                    },
                );
            }

            ui.horizontal(|ui| {
                euler_col(ui, n1, &mut var_ptrs, &mut degrees, 0, w3, &mut changed);
                euler_col(ui, n2, &mut var_ptrs, &mut degrees, 1, w3, &mut changed);
                euler_col(ui, n3, &mut var_ptrs, &mut degrees, 2, w3, &mut changed);
            });

            for (q, deg) in quats.iter_mut().zip(degrees.into_iter()) {
                **q = Quat::from_euler(
                    glam_rot,
                    deg[0].to_radians(),
                    deg[1].to_radians(),
                    deg[2].to_radians(),
                );
            }
        }
    }

    if changed {
        history_pusher(snapshot_provider());
        on_change();
    }
}*/

pub fn compute_vertex_row(
    ui: &mut Ui,
    w: (f32, f32),
    comp: &mut ComputeVertex,
    key: &String,
    part: &mut Part,
    s: &mut App,
    gl: &glow::Context,
) {
    let (w2, w3) = w;
    let rd = RefDuper;
    let s2 = unsafe { rd.detach_mut_ref(s) };
    ui.horizontal(|ui| {
        for idx in 0..2 {
            let id = &mut comp.points[idx];
            let check = id.clone();
            let disp = match id {
                VertexId::Named(n) => n.to_string(),
                VertexId::Index(i) => format!("{i}"),
            };

            egui::ComboBox::from_id_salt(format!("{key}-{idx}"))
                .selected_text(disp)
                .width(w2)
                .show_ui(ui, |ui| {
                    for name in part.get_valid_vertex_ids() {
                        let disp = match &name {
                            VertexId::Named(n) => n.to_string(),
                            VertexId::Index(i) => format!("{i}"),
                        };
                        ui.selectable_value(id, name, disp);
                    }
                });

            if *id != check {
                s.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        id: s.get_current_mesh_id().unwrap().to_string(),
                        name: s.get_current_part_name().unwrap().to_string(),
                        part: Box::new(part.clone()),
                    },
                ));
                s.rebuild_meshes(gl);
            }
        }
    });

    let mut vars = HashMap::new();

    let w4 = w3 * 2. + ui.spacing().item_spacing.x;

    ui.horizontal(|ui| {
        let size = egui::vec2(w3, 20.);
        let size2 = egui::vec2(w4, 20.);
        let layout = egui::Layout::left_to_right(egui::Align::Center);
        ui.allocate_ui_with_layout(size, layout, |ui| {
            ui.set_min_width(w3);
            ui.label("D");
        });
        ui.allocate_ui_with_layout(size2, layout, |ui| {
            ui.set_min_width(w4);
            ui.label("Easing");
        });
    });
    delta_function_row(
        ui,
        (&mut comp.function, &mut comp.delta, &mut vars),
        key.as_str(),
        (w3, w4),
        || part.clone(),
        |t| {
            s.add_history(editor::HistoryEntry::MeshPart(
                light_mesh::LightMeshPartSnapshot {
                    id: s.get_current_mesh_id().unwrap().to_string(),
                    name: s.get_current_part_name().unwrap().to_string(),
                    part: Box::new(t),
                },
            ))
        },
        || s2.rebuild_meshes(gl),
    );

    ui.horizontal(|ui| {
        let size = egui::vec2(w3, 20.);
        let layout = egui::Layout::left_to_right(egui::Align::Center);
        ui.allocate_ui_with_layout(size, layout, |ui| {
            ui.set_min_width(w3);
            ui.label("X");
        });
        ui.allocate_ui_with_layout(size, layout, |ui| {
            ui.set_min_width(w3);
            ui.label("Y");
        });
        ui.allocate_ui_with_layout(size, layout, |ui| {
            ui.set_min_width(w3);
            ui.label("Z");
        });
    });
    vec3_opt_row(
        ui,
        [&mut comp.x, &mut comp.y, &mut comp.z],
        w3,
        &mut vars,
        || part.clone(),
        |t| {
            s.add_history(editor::HistoryEntry::MeshPart(
                light_mesh::LightMeshPartSnapshot {
                    id: s.get_current_mesh_id().unwrap().to_string(),
                    name: s.get_current_part_name().unwrap().to_string(),
                    part: Box::new(t),
                },
            ))
        },
        || s2.rebuild_meshes(gl),
    );
}
