// Control Scheme
//   Left-click             select vertex (shift = multi-select) (ctrl = toggle select)
//   Left-drag (empty)      orbit
//   Shift+Left-click       add/remove select
//   Shift+Left-drag        marquee select
//   Right-drag             pan
//   Scroll                 zoom
//   Left-drag (vertex)     move along viewport-parallel plane
//   W                      wireframe toggle
//   G                      grid toggle
//   V                      vertex dots toggle
//   C (part-edit)          spawn vertex at cursor
//   E                      assembly <-> part-edit mode
//   A / D                  cycle active part
//   N                      create/remove triangle from selection
//   R                      flip winding of selected triangles
//   Ctrl+Z / Ctrl+Shift+Z  undo / redo
//   Ctrl+S                 save JSON
//   Escape                 deselect all

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};

use eframe::glow::{self, HasContext};
use egui::{Align2, Frame, Sense, Ui};
use glam::{Mat4, Quat, Vec2, Vec3};
use indexmap::IndexMap;
use indexmap::map::MutableKeys;

use self::data::{NormalId, UvId, VertexId};
use self::easing::Easing;
use self::editor::{App, RotationDisplayMode, Selection, ViewPlacement, WorkingRenameKey};
use self::light_mesh::{BloomfogStyle, ComputeNormal, ComputeVertex, Part, Triangle};
use self::renaming::light_mesh::rehash;
use self::render::{HandleDrawCall, InstanceData, MeshDrawCall, PointDrawCall};
use self::widgets::{MathDragValue, MathDragValueOpt, MultiMathValue, TextInput};

pub mod data;
pub mod easing;
pub mod editor;
pub mod light_mesh;
pub mod math_interp;
pub mod renaming;
pub mod render;
pub mod widgets;

pub static SMALL_X: &str = "×";
pub static R_ARROW: &str = "▶";
pub static D_ARROW: &str = "▼";
pub static SMALL_R_ARROW: &str = "→";

#[derive(Copy, Clone)]
struct UnsafeMutRef<T: 'static> {
    t: *mut T,
}

impl<T: 'static> UnsafeMutRef<T> {
    pub unsafe fn new(t: &mut T) -> Self {
        let ptr = t as *mut T;
        Self { t: ptr }
    }
}

impl<T> Deref for UnsafeMutRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.t.cast::<T>() }
    }
}

impl<T> UnsafeMutRef<T> {
    pub fn ref_mut(&self) -> &'static mut T {
        unsafe { &mut *self.t.cast::<T>() }
    }
}

unsafe impl<T> Send for UnsafeMutRef<T> {}
unsafe impl<T> Sync for UnsafeMutRef<T> {}

pub struct RefDuper;

impl RefDuper {
    /// Detaches a reference from it's owner, allowing
    /// mutable references to exist simultaneously
    /// SAFETY: attaches the lifetime to self for
    /// the illusion of safety
    pub(crate) unsafe fn detach_ref<'a, T>(&'a self, t: &T) -> &'a T {
        unsafe { &*(t as *const T) }
    }

    /// Detaches a mutable reference from it's owner, allowing
    /// more references to be created
    /// SAFETY: attaches the lifetime to self for
    /// the illusion of safety
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn detach_mut_ref<'a, T>(&'a self, t: &mut T) -> &'a mut T {
        unsafe { &mut *(t as *mut T) }
    }
}

fn vec3_row<T: 'static + Clone + Send + Sync>(
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
    ui.horizontal(|ui| {
        let mut current = *v;
        for val in current.as_mut() {
            ui.allocate_ui_with_layout(
                (w3, 20.).into(),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let id = ui.next_auto_id();
                    ui.set_clip_rect(ui.max_rect());
                    let resp = ui.add_sized(
                        [w3, 20.],
                        MathDragValue::new(val, &mut vars)
                            .speed(0.01)
                            .max_decimals(3),
                    );

                    if resp.changed() {
                        on_change();
                    }
                    if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                        ui.memory_mut(|m| {
                            m.data.insert_temp(id, *v);
                            m.data.insert_temp(id, snapshot_provider());
                        });
                    }
                    if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                        && let Some((old, t)) = ui.memory_mut(|m| {
                            let o = m.data.get_temp::<Vec3>(id)?;
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
}

fn vec2_row<T: 'static + Clone + Send + Sync>(
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
                    ui.set_clip_rect(ui.max_rect());
                    let resp = ui.add_sized(
                        [w2, 20.],
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

fn multi_vec3_row<T: 'static + Clone + Send + Sync>(
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
            for ((c, vs), x) in current.iter_mut().zip(vars.iter_mut()).zip(x.into_iter()) {
                c.x = x;
                vs.insert("x".into(), x);
            }
            changed = true;
        }
        let mut vals = None;
        axis_value(ui, "y", &mut vars, w3, &mut vals);
        if let Some(y) = vals {
            for ((c, vs), y) in current.iter_mut().zip(vars.iter_mut()).zip(y.into_iter()) {
                c.y = y;
                vs.insert("y".into(), y);
            }
            changed = true;
        }
        vals = None;
        axis_value(ui, "z", &mut vars, w3, &mut vals);
        if let Some(z) = vals {
            for ((c, vs), z) in current.iter_mut().zip(vars.iter_mut()).zip(z.into_iter()) {
                c.z = z;
                vs.insert("z".into(), z);
            }
            changed = true;
        }
    });

    for (v, c) in vertices.iter_mut().zip(current.into_iter()) {
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

fn vec3_opt_row<T: 'static + Clone + Send + Sync>(
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
                    ui.set_clip_rect(ui.max_rect());
                    let resp = ui.add_sized(
                        [w3, 20.],
                        MathDragValueOpt::new(val, vars).speed(0.01).max_decimals(3),
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

fn delta_function_row<T: 'static + Clone + Send + Sync>(
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
                ui.set_clip_rect(ui.max_rect());
                let resp = ui.add_sized(
                    [w2, 20.],
                    MathDragValueOpt::new(delta, vars)
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
                ui.set_clip_rect(ui.max_rect());
                egui::ComboBox::from_id_salt(egui::Id::new("delta_function").with(salt))
                    .selected_text(func.display_name())
                    .width(w3)
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

fn quat_row<T: 'static + Clone + Send + Sync>(
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
                            ui.set_clip_rect(ui.max_rect());
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
                            ui.set_clip_rect(ui.max_rect());
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
            let normalize_angle = |d: f32| {
                let d = if d == -0.0 { 0.0 } else { d };
                if (d - 180.0).abs() < 0.001 || (d + 180.0).abs() < 0.001 {
                    180.0
                } else {
                    d
                }
            };

            let mut degrees = [
                normalize_angle(ax.to_degrees()),
                normalize_angle(ay.to_degrees()),
                normalize_angle(az.to_degrees()),
            ];

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
                            ui.set_clip_rect(ui.max_rect());
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

/*fn multi_quat_row<T: 'static + Clone + Send + Sync>(
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

fn compute_vertex_row(
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
                        idx: s.get_current_mesh_idx().unwrap(),
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
                    idx: s.get_current_mesh_idx().unwrap(),
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
                    idx: s.get_current_mesh_idx().unwrap(),
                    name: s.get_current_part_name().unwrap().to_string(),
                    part: Box::new(t),
                },
            ))
        },
        || s2.rebuild_meshes(gl),
    );
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let gl = frame.gl().unwrap();

        if self.state.dirty {
            self.rebuild_meshes(gl);
        }

        self.handle_keys(ctx, gl);

        let dt = ctx.input(|i| i.unstable_dt);
        if self.state.status_timer > 0. {
            if let Some(t) = self.state.title_content.as_mut()
                && !t.is_empty()
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                    "{} {}",
                    self.title, t
                )));
                t.clear();
            }
            self.state.status_timer -= dt;
        } else if self.state.title_content.is_some() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.title.to_string()));
            self.state.title_content = None;
        }

        self.handle_file_open(gl);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    // Open Session...
                    if ui.button("Open Session\u{2026}  \u{2502}").clicked() && !self.block_input()
                    {
                        let (sx, rx) = mpsc::channel();
                        self.state.ui.open_session_channel = Some(rx);
                        std::thread::spawn(move || {
                            if let Some(session) = rfd::FileDialog::new()
                                .set_title("Open Session...")
                                .add_filter("json", &["json"])
                                .pick_file()
                            {
                                let _ = sx.send(session);
                            }
                        });
                    }
                    // Open...
                    if ui.button("Open\u{2026}          \u{2502}").clicked() && !self.block_input()
                    {
                        let (sx, rx) = mpsc::channel();
                        self.state.ui.open_mesh_channel = Some(rx);
                        std::thread::spawn(move || {
                            if let Some(meshes) = rfd::FileDialog::new()
                                .set_title("Open Meshes...")
                                .add_filter("json", &["json"])
                                .pick_files()
                            {
                                let _ = sx.send(meshes);
                            }
                        });
                    }
                    if ui.button("Save           \u{2502} [Ctrl+S]").clicked() {
                        match self.mode {
                            editor::EditorMode::View => {}
                            editor::EditorMode::Assembly | editor::EditorMode::Edit => {}
                        }
                        ui.close();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo  \u{2502}       [Ctrl+Z]").clicked() {
                        self.undo(gl);
                        ui.close();
                    }
                    if ui.button("Redo  \u{2502} [Ctrl+Shift+Z]").clicked() {
                        self.redo(gl);
                        ui.close();
                    }
                });
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.state.wireframe, "Wireframe     \u{2502} [W]");
                    ui.checkbox(&mut self.state.show_grid, "Show Grid     \u{2502} [G]");
                    ui.checkbox(&mut self.state.show_verts, "Vertices      \u{2502} [C]");
                    if ui.button("Reframe Camera  \u{2502} [F]").clicked() {
                        self.frame_to_geometry();
                        ui.close();
                    }
                });
                if !self.state.status.is_empty() && self.state.status_timer > 0. {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(&self.state.status)
                                .color(egui::Color32::from_rgb(140, 200, 140)),
                        );
                    });
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let display = match self.mode {
                    editor::EditorMode::View =>
                        " View       | [Ctrl+S] Save session",
                    editor::EditorMode::Assembly =>
                        " Assembly   | [Ctrl+S] Save mesh | [E]dit parts | [I] View ",
                    editor::EditorMode::Edit =>
                        " Edit part  | [Ctrl+S] Save mesh | [E] Assembly | [I] View | [C]reate vertex | [N] Add/Remove tris | [R]ewind triangles",
                };
                ui.label(display);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("[W]ireframe | [G]rid | [V]ertices \u{0}")
                });
            });
        });

        egui::SidePanel::left("left_panel")
            .exact_width(250.)
            .resizable(false)
            .show(ctx, |ui| {
                ui.allocate_ui(ui.available_size(), |ui| {
                    let h = if self.mode == editor::EditorMode::Edit {
                        75.
                    } else {
                        45.
                    };
                    ui.allocate_exact_size((230., 1.).into(), Sense::empty());
                    ui.allocate_ui(
                        (ui.available_width(), ui.available_height() - h).into(),
                        |ui| {
                            egui::ScrollArea::vertical()
                                .id_salt("left_p_scroll")
                                .show(ui, |ui| match self.mode {
                                    editor::EditorMode::View => {
                                        draw_view_left(self, ui, gl);
                                    }
                                    editor::EditorMode::Assembly => {
                                        draw_assembly_left(self, ui, gl);
                                    }
                                    editor::EditorMode::Edit => {
                                        draw_edit_left(self, ui, gl);
                                    }
                                });
                        },
                    );

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
                            ui.label("Camera Pivot");
                            if h > 45. {
                                ui.add_space(5.);
                                ui.allocate_ui_with_layout(
                                    [ui.available_width(), 20.].into(),
                                    egui::Layout::bottom_up(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .add_sized(
                                                ui.available_size(),
                                                egui::Button::new("Show UV Editor")
                                                    .selected(self.state.ui.show_uv_window),
                                            )
                                            .clicked()
                                        {
                                            self.state.ui.show_uv_window =
                                                !self.state.ui.show_uv_window;
                                        }
                                    },
                                );
                            }
                        });
                    });
                });
            });

        egui::SidePanel::right("right_panel")
            .exact_width(300.)
            .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| match self.mode {
                    editor::EditorMode::View => {
                        draw_view_right(self, ui, gl);
                    }
                    editor::EditorMode::Assembly => {
                        draw_assembly_right(self, ui, gl);
                    }
                    editor::EditorMode::Edit => {
                        draw_edit_right(self, ui, gl);
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(Frame::NONE)
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();
                self.state.vp_rect = rect;

                let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                self.handle_3d_input(&resp, ctx, gl);

                let s = unsafe { UnsafeMutRef::new(self) };

                ui.painter().add(egui::PaintCallback {
                    rect,
                    callback: Arc::new(eframe::egui_glow::CallbackFn::new(
                        move |_info, painter| {
                            let gl = painter.gl();
                            unsafe {
                                let w = rect.width();
                                let h = rect.height();
                                let vp = s.ref_mut().cam().vp(w, h);

                                gl.clear_color(0.07, 0.08, 0.11, 1.);
                                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                                gl.enable(glow::DEPTH_TEST);

                                if s.state.show_grid {
                                    let flat = s.render.renderer.flat;
                                    gl.line_width(1.);
                                    gl.use_program(Some(flat));
                                    if let Some(l) = gl.get_uniform_location(flat, "uMVP") {
                                        gl.uniform_matrix_4_f32_slice(
                                            Some(&l),
                                            false,
                                            &vp.to_cols_array(),
                                        );
                                    }
                                    if let Some(l) = gl.get_uniform_location(flat, "uColor") {
                                        gl.uniform_4_f32(Some(&l), 0.27, 0.27, 0.34, 0.5);
                                    }
                                    gl.bind_vertex_array(Some(s.render.renderer.grid_vao));
                                    gl.draw_arrays(glow::LINES, 0, s.render.renderer.grid_n);
                                    gl.line_width(2.);
                                    if let Some(l) = gl.get_uniform_location(flat, "uColor") {
                                        gl.uniform_4_f32(Some(&l), 0.85, 0.2, 0.2, 0.9);
                                    }
                                    gl.bind_vertex_array(Some(s.render.renderer.axis_vao));
                                    gl.draw_arrays(glow::LINES, 0, 2);
                                    if let Some(l) =
                                        gl.get_uniform_location(s.render.renderer.flat, "uColor")
                                    {
                                        gl.uniform_4_f32(Some(&l), 0.2, 0.45, 0.9, 0.9);
                                    }
                                    gl.draw_arrays(glow::LINES, 2, 2);
                                    gl.line_width(1.);
                                    gl.bind_vertex_array(None);
                                }

                                match s.mode {
                                    editor::EditorMode::View => {
                                        draw_view_gl(&s, gl, &vp);
                                    }
                                    editor::EditorMode::Assembly => {
                                        draw_assembly_gl(&s, gl, &vp);
                                    }
                                    editor::EditorMode::Edit => {
                                        draw_edit_gl(&s, gl, &vp);
                                    }
                                }
                            }
                        },
                    )),
                });
            });

        if self.mode == editor::EditorMode::Edit && self.state.ui.show_uv_window {
            egui::Window::new("UV Editor")
                .id(egui::Id::new("uv_editor"))
                .min_size([200., 200.])
                .pivot(Align2::CENTER_CENTER)
                .default_pos([200., 800.])
                .show(ctx, |ui| {
                    draw_uv_view(self, ui, ctx, gl);
                });
        }
    }
}

fn draw_view_left(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
    let mut to_remove = None;
    let w = ui.available_width();
    let rd = RefDuper;
    let s2 = unsafe { rd.detach_mut_ref(s) };
    for (i, mesh) in s.view.meshes.iter_mut().enumerate() {
        let name = mesh.path.with_extension("");
        let name = name
            .file_name()
            .map(|x| x.to_string_lossy())
            .unwrap_or_else(|| std::borrow::Cow::Borrowed("?"));

        ui.add_space(2.0);
        let selected = s.state.ui.view_mesh == Some(i);
        let available = ui.available_size_before_wrap();
        if ui
            .add_sized(
                egui::Vec2::new(available.x, 20.0),
                egui::Button::selectable(selected, name.as_ref()),
            )
            .clicked()
        {
            s.state.ui.view_mesh = if selected { None } else { Some(i) };
        };
        ui.add_space(2.0);

        if ui
            .horizontal(|ui| {
                ui.checkbox(&mut mesh.visible, "");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        to_remove = Some(i);
                    }
                    if ui.button("Edit").clicked() {
                        s.editor.mesh = Some(i);
                        s.last_mode = s.mode;
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
        s.view.meshes.remove(i);
        if let Some(sel) = s.state.ui.view_mesh {
            if sel == i {
                s.state.ui.view_mesh = None;
            } else if sel > i {
                s.state.ui.view_mesh = Some(sel - 1);
            }
        }
        if let Some(sel) = s.editor.mesh {
            if sel == i {
                s.editor.mesh = None;
            } else if sel > i {
                s.editor.mesh = Some(sel - 1);
            }
        }
    }

    if ui
        .add_sized([w, 20.], egui::Button::new("+ Mesh"))
        .clicked()
    {
        let (sx, rx) = mpsc::channel();
        s.state.ui.create_mesh_channel = Some(rx);
        std::thread::spawn(move || {
            if let Some(file) = rfd::FileDialog::new()
                .set_title("Create new mesh")
                .set_file_name("new_mesh")
                .add_filter("json", &["json"])
                .save_file()
            {
                let _ = sx.send(file);
            }
        });
    }

    if s.view.meshes.len() > 1
        && ui
            .add_sized([w, 20.], egui::Button::new("Close All"))
            .clicked()
    {
        let meshes = std::mem::take(&mut s.view.meshes);
        for vm in meshes {
            vm.destroy(gl);
        }
        s.view.session = None;
    }
}

fn draw_view_right(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
    let rd = RefDuper;
    let s2 = unsafe { rd.detach_mut_ref(s) };
    let s3 = unsafe { rd.detach_mut_ref(s) };
    if let Some(sel) = s.state.ui.view_mesh
        && let Some(mesh) = s.view.meshes.get_mut(sel)
    {
        let rd2 = RefDuper;
        let mesh2 = unsafe { rd2.detach_mut_ref(mesh) };
        if ui.button("+ Add Placement").clicked() {
            mesh.view_placements.push(ViewPlacement::default());
            s.state.ui.collapsed.entry(sel).or_default().push(false);
            s.state
                .ui
                .view_rotation_modes
                .entry(sel)
                .or_default()
                .push(Default::default());
            s2.rebuild_meshes(gl);
        }

        let mut to_remove = None;
        for (i, placement) in mesh.view_placements.iter_mut().enumerate() {
            let collapsed = s.state.ui.collapsed.entry(sel).or_default();
            if collapsed.len() <= i {
                collapsed.push(false);
            }
            let is_collapsed = &mut collapsed[i];

            ui.horizontal(|ui| {
                let icon = if *is_collapsed { R_ARROW } else { D_ARROW };
                if ui.button(icon).clicked() {
                    *is_collapsed = !*is_collapsed;
                }
                ui.label(format!("Placement {}", i + 1));
            });

            if !*is_collapsed {
                let w2 = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
                let w3 = (ui.available_width() - ui.spacing().item_spacing.x * 2.0) / 3.0;

                let modes = s.state.ui.view_rotation_modes.entry(sel).or_default();
                if modes.len() <= i {
                    modes.push(Default::default());
                }
                let [rot_mode, off_mode] =
                    &mut s.state.ui.view_rotation_modes.get_mut(&sel).unwrap()[i];

                ui.label("Position");
                vec3_row(
                    ui,
                    &mut placement.position,
                    w3,
                    || mesh2.view_placements.clone(),
                    |t| {
                        s2.add_history(editor::HistoryEntry::ViewPlacement(
                            editor::ViewPlacementsSnapshot {
                                idx: sel,
                                placements: t,
                            },
                        ))
                    },
                    || s3.rebuild_meshes(gl),
                );

                ui.label("Rotation");
                quat_row(
                    ui,
                    &mut placement.rotation,
                    rot_mode,
                    (w2, w3),
                    || mesh2.view_placements.clone(),
                    |t| {
                        s2.add_history(editor::HistoryEntry::ViewPlacement(
                            editor::ViewPlacementsSnapshot {
                                idx: sel,
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
                            ui.set_clip_rect(ui.max_rect());
                            ui.add(
                                egui::DragValue::new(&mut placement.count)
                                    .speed(1)
                                    .range(1..=u32::MAX),
                            );
                        },
                    );
                    ui.label("Count");
                });

                ui.label("Offset Position");
                vec3_row(
                    ui,
                    &mut placement.offset_pos,
                    w3,
                    || mesh2.view_placements.clone(),
                    |t| {
                        s2.add_history(editor::HistoryEntry::ViewPlacement(
                            editor::ViewPlacementsSnapshot {
                                idx: sel,
                                placements: t,
                            },
                        ))
                    },
                    || s3.rebuild_meshes(gl),
                );

                ui.label("Offset Rotation");
                quat_row(
                    ui,
                    &mut placement.offset_rot,
                    off_mode,
                    (w2, w3),
                    || mesh2.view_placements.clone(),
                    |t| {
                        s2.add_history(editor::HistoryEntry::ViewPlacement(
                            editor::ViewPlacementsSnapshot {
                                idx: sel,
                                placements: t,
                            },
                        ))
                    },
                    || s3.rebuild_meshes(gl),
                );

                if ui
                    .add_sized([ui.available_width(), 20.0], egui::Button::new("Delete"))
                    .clicked()
                {
                    to_remove = Some(i);
                }
            }

            ui.separator();
        }

        if let Some(rem) = to_remove {
            mesh.view_placements.remove(rem);
            if let Some(collapsed) = s.state.ui.collapsed.get_mut(&sel) {
                collapsed.remove(rem);
            }
            if let Some(modes) = s.state.ui.view_rotation_modes.get_mut(&sel) {
                modes.remove(rem);
            }
        }
    }
}

fn draw_view_gl(s: &UnsafeMutRef<App>, gl: &glow::Context, vp: &Mat4) {
    let mut calls = Vec::new();
    for vm in s.view.meshes.iter() {
        let mut draws = Vec::new();
        if let Some(mesh) = vm.render_view_placements(&mut draws) {
            calls.push(MeshDrawCall {
                mesh,
                instances: draws,
                wireframe: s.state.wireframe,
            })
        }
    }
    s.render.renderer.draw_meshes(gl, vp, &calls);
}

fn draw_assembly_left(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
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
            ui.label("Placements");
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

                    // Position
                    ui.horizontal(|ui| {
                        ui.label("Position");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Paste").clicked() { /* TODO */ }
                            if ui.small_button("Copy").clicked() { /* TODO */ }
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
                                    view_idx: self3.get_current_mesh_idx().unwrap(),
                                    placements: t,
                                },
                            ))
                        },
                        || self4.rebuild_meshes(gl),
                    );

                    // Rotation
                    ui.horizontal(|ui| {
                        ui.label("Rotation");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Paste").clicked() { /* TODO */ }
                            if ui.small_button("Copy").clicked() { /* TODO */ }
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
                                    view_idx: self3.get_current_mesh_idx().unwrap(),
                                    placements: t,
                                },
                            ))
                        },
                        || self4.rebuild_meshes(gl),
                    );

                    // Scale
                    ui.horizontal(|ui| {
                        ui.label("Scale");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Paste").clicked() { /* TODO */ }
                            if ui.small_button("Copy").clicked() { /* TODO */ }
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
                                    view_idx: self3.get_current_mesh_idx().unwrap(),
                                    placements: t,
                                },
                            ))
                        },
                        || self4.rebuild_meshes(gl),
                    );

                    // Remap Data
                    ui.horizontal(|ui| {
                        let icon = if pt_collapsed.0[1] { R_ARROW } else { D_ARROW };
                        if ui.small_button(icon).clicked() {
                            pt_collapsed.0[1] = !pt_collapsed.0[1];
                        }
                        ui.label("Remap Data");
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
                        if ui.add_sized([w, 20.], egui::Button::new("+ Add")).clicked() {
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
                .add_sized([w, 20.], egui::Button::new("+ Add Placement"))
                .clicked()
                && let Some(first) = part_names.first()
            {
                self3.add_history(editor::HistoryEntry::Mesh(light_mesh::LightMeshSnapshot {
                    idx: self3.get_current_mesh_idx().unwrap(),
                    mesh: Box::new(mesh.data.clone()),
                }));
                mesh.data.placements.push(light_mesh::Placement {
                    part: first.clone(),
                    position: Vec3::ZERO,
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
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
            ui.label("Data");
        });

        if !toggles.data {
            if ui
                .add_sized([w, 20.], egui::Button::new("+ Add Data"))
                .clicked()
            {
                mesh.data.data.insert(
                    format!("new_data_{}", mesh.data.data.len()),
                    Default::default(),
                );
            }

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
                                view_idx: self3.get_current_mesh_idx().unwrap(),
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
                    ui.horizontal(|ui| {
                        let mat_label = format!("Material {}", entry.material);
                        if ui.button(mat_label).clicked() {
                            entry.material = (entry.material + 1) % 3;
                        }

                        egui::ComboBox::from_id_salt(egui::Id::new("data_ch").with(di))
                            .selected_text(format!("Channel {}", entry.color))
                            .show_ui(ui, |ui| {
                                for ch in 0u8..8 {
                                    ui.selectable_value(&mut entry.color, ch, ch.to_string());
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Texture");
                        ui.add(
                            egui::DragValue::new(&mut entry.texture)
                                .range(0..=255u8)
                                .speed(1),
                        );
                    });
                }

                ui.separator();
            }

            if let Some(key) = data_to_remove {
                mesh.data.data.shift_remove(&key);
                self3.rebuild_meshes(gl);
            }
        }

        ui.horizontal(|ui| {
            let icon = if toggles.textures { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                toggles.textures = !toggles.textures;
            }
            ui.label("Textures");
        });

        if !toggles.textures {
            if ui
                .add_sized([w, 20.], egui::Button::new("+ Add Texture"))
                .clicked()
            {
                mesh.data
                    .textures
                    .insert(format!("{}", mesh.data.textures.len()), String::new());
            }

            let mut tex_to_remove = None;
            let tex_keys: Vec<String> = mesh.data.textures.keys().cloned().collect();
            for (ti, key) in tex_keys.iter().enumerate() {
                let val = mesh.data.textures.get_mut(key).unwrap();
                ui.horizontal(|ui| {
                    ui.label(format!("{ti}"));
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
                        self2.state.ui.select_image_channel = Some((val.clone(), rx));
                        std::thread::spawn(move || {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Choose Image")
                                .add_filter("png", &["png"])
                                .pick_file()
                            {
                                let _ = sx.send(path);
                            }
                        });
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
        }

        ui.horizontal(|ui| {
            let icon = if toggles.settings { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                toggles.settings = !toggles.settings;
            }
            ui.label("Render Settings");
        });

        if !toggles.settings {
            ui.horizontal(|ui| {
                let size = egui::vec2(w2, 20.);
                let layout = egui::Layout::left_to_right(egui::Align::Center);
                ui.allocate_ui_with_layout(size, layout, |ui| {
                    ui.set_min_width(w2);
                    ui.checkbox(&mut mesh.data.cull, "Cull")
                });
                ui.allocate_ui_with_layout(size, layout, |ui| {
                    ui.set_min_width(w2);
                    ui.checkbox(&mut mesh.data.do_bloom, "Bloom")
                });
            });
            ui.horizontal(|ui| {
                let size = egui::vec2(w2, 20.);
                let layout = egui::Layout::left_to_right(egui::Align::Center);
                ui.allocate_ui_with_layout(size, layout, |ui| {
                    ui.set_min_width(w2);
                    ui.checkbox(&mut mesh.data.do_mirroring, "Mirror")
                });
                ui.allocate_ui_with_layout(size, layout, |ui| {
                    ui.set_min_width(w2);
                    ui.checkbox(&mut mesh.data.do_solid, "Solid")
                });
            });
            egui::ComboBox::from_id_salt("bloomfog_style")
                .selected_text(mesh.data.bloomfog_style.label())
                .width(w)
                .show_ui(ui, |ui| {
                    for style in [
                        BloomfogStyle::BloomOnly,
                        BloomfogStyle::Everything,
                        BloomfogStyle::Nothing,
                    ] {
                        ui.selectable_value(&mut mesh.data.bloomfog_style, style, style.label());
                    }
                });
        }

        ui.horizontal(|ui| {
            let icon = if toggles.credits { R_ARROW } else { D_ARROW };
            if ui.small_button(icon).clicked() {
                toggles.credits = !toggles.credits;
            }
            ui.label("Credits");
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
                .add_sized([w, 20.], egui::Button::new("+ Add Credit"))
                .clicked()
            {
                mesh.data.credits.push(String::new());
            }
        }
    }
}

fn draw_assembly_right(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
    let rd = RefDuper;
    let s2 = unsafe { rd.detach_mut_ref(s) };
    let w = ui.available_width();
    if let Some(mesh) = s.get_current_view_mesh_mut() {
        ui.label("Parts");
        let mesh2 = unsafe { rd.detach_mut_ref(mesh) };
        for name in mesh.data.part_names.iter() {
            if ui
                .horizontal(|ui| {
                    let mut new_name = None;
                    ui.add_sized([w - 24., 20.], TextInput::new(name, &mut new_name));
                    if let Some(new_name) = new_name {
                        let _ = s2.rename(editor::Rename::Part {
                            view_idx: s2.get_current_mesh_idx().unwrap(),
                            swap: editor::DataSwap {
                                from: name.clone(),
                                to: new_name,
                            },
                        });
                        return true;
                    }
                    if ui.small_button(SMALL_X).clicked() {
                        s2.add_history(editor::HistoryEntry::Mesh(light_mesh::LightMeshSnapshot {
                            idx: s2.get_current_mesh_idx().unwrap(),
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
        ui.add_sized([w, 20.], TextInput::new("+ Part", &mut new_part));
        if let Some(name) = new_part {
            s2.add_history(editor::HistoryEntry::Mesh(light_mesh::LightMeshSnapshot {
                idx: s2.get_current_mesh_idx().unwrap(),
                mesh: Box::new(mesh2.data.clone()),
            }));
            mesh.data.parts.insert(name, Part::default());
            s2.rebuild_meshes(gl);
        }
    }
}

fn draw_assembly_gl(s: &UnsafeMutRef<App>, gl: &glow::Context, vp: &Mat4) {
    if let Some(sel) = s.editor.mesh
        && let Some(mesh) = s.view.meshes.get(sel)
    {
        let mut instances = Vec::new();
        if let Some(mesh) = mesh.render_assembly(&mut instances) {
            s.render.renderer.draw_meshes(
                gl,
                vp,
                &[MeshDrawCall {
                    mesh,
                    instances: instances.clone(),
                    wireframe: s.state.wireframe,
                }],
            );
        }
        if s.state.show_verts
            && let Some(handles) = mesh.gpu_bufs.2.as_ref()
        {
            s.render.renderer.draw_handles(
                gl,
                vp,
                &[HandleDrawCall {
                    mesh: handles,
                    instances: vec![InstanceData::new(Mat4::IDENTITY, 0.5, Some([1., 1., 1.]))],
                }],
            );
        }

        let mut calls = Vec::new();

        if let Some(selected) = s.render.inst_points.as_ref() {
            calls.push(PointDrawCall {
                mesh: selected,
                instances: vec![InstanceData::new(Mat4::IDENTITY, 1., Some([1., 1., 1.]))],
                size: 6.,
            });
        }

        if !calls.is_empty() {
            s.render.renderer.draw_points_batch(gl, vp, &calls);
        }
    }
}

fn draw_edit_left(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
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
            ui.label("Indexed Vertices");
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
                                idx: self3.get_current_mesh_idx().unwrap(),
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
            ui.label("Named Vertices");
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
                            view_idx: self3.get_current_mesh_idx().unwrap(),
                            name: self3.get_current_part_name().unwrap().to_string(),
                        },
                        swap: editor::DataSwap {
                            from: data::VertexId::Named(key.clone()),
                            to: data::VertexId::Named(name),
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
                                idx: self3.get_current_mesh_idx().unwrap(),
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
            ui.label("Compute Vertices");
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
                            view_idx: self3.get_current_mesh_idx().unwrap(),
                            name: self3.get_current_part_name().unwrap().to_string(),
                        },
                        swap: editor::DataSwap {
                            from: data::VertexId::Named(key.clone()),
                            to: data::VertexId::Named(name),
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
            ui.label("Indexed UVs");
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
                                        idx: self3.get_current_mesh_idx().unwrap(),
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
                                idx: self3.get_current_mesh_idx().unwrap(),
                                name: self3.get_current_part_name().unwrap().to_string(),
                                part: Box::new(t),
                            },
                        ))
                    },
                    || self4.rebuild_meshes(gl),
                );
            }
            ui.separator();
            if ui.button("+ UV").clicked() {
                self3.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        idx: self3.get_current_mesh_idx().unwrap(),
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
            ui.label("Named UVs");
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
                                    view_idx: self3.get_current_mesh_idx().unwrap(),
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
                                    idx: self3.get_current_mesh_idx().unwrap(),
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
                                idx: self3.get_current_mesh_idx().unwrap(),
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
            ui.add_sized([w, 20.], TextInput::new("+ Named UV", &mut new_uv));
            if let Some(name) = new_uv {
                self3.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        idx: self3.get_current_mesh_idx().unwrap(),
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
            ui.label("Indexed Normals");
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
                                        idx: self3.get_current_mesh_idx().unwrap(),
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
                                idx: self3.get_current_mesh_idx().unwrap(),
                                name: self3.get_current_part_name().unwrap().to_string(),
                                part: Box::new(t),
                            },
                        ))
                    },
                    || self4.rebuild_meshes(gl),
                );
            }
            ui.separator();
            if ui.button("+ Normal").clicked() {
                self3.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        idx: self3.get_current_mesh_idx().unwrap(),
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
            ui.label("Named Normals");
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
                                    view_idx: self3.get_current_mesh_idx().unwrap(),
                                    name: self3.get_current_part_name().unwrap().to_string(),
                                },
                                swap: editor::DataSwap {
                                    from: data::NormalId::Named(key.clone()),
                                    to: data::NormalId::Named(name.clone()),
                                },
                            });
                            self3.state.ui.working_key = WorkingRenameKey::None;
                            return true;
                        }
                        if ui.small_button(SMALL_X).clicked() {
                            self3.add_history(editor::HistoryEntry::MeshPart(
                                light_mesh::LightMeshPartSnapshot {
                                    idx: self3.get_current_mesh_idx().unwrap(),
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
                                idx: self3.get_current_mesh_idx().unwrap(),
                                name: self3.get_current_part_name().unwrap().to_string(),
                                part: Box::new(t),
                            },
                        ))
                    },
                    || self4.rebuild_meshes(gl),
                );
            }
            let mut new_norm = None;
            ui.add_sized([w, 20.], TextInput::new("+ Named Normal", &mut new_norm));
            if let Some(name) = new_norm {
                self3.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        idx: self3.get_current_mesh_idx().unwrap(),
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
            ui.label("Compute Normals");
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
                                    view_idx: self3.get_current_mesh_idx().unwrap(),
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
                                    idx: self3.get_current_mesh_idx().unwrap(),
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
                                    idx: self3.get_current_mesh_idx().unwrap(),
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

fn draw_edit_right(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
    let rd = RefDuper;
    let s2 = unsafe { rd.detach_mut_ref(s) };
    let self3 = unsafe { rd.detach_mut_ref(s) };
    let self4 = unsafe { rd.detach_mut_ref(s) };

    let w = ui.available_width();

    ui.label("Cycle part [A / D]");

    if let Some(current) = s2.get_current_part_name() {
        let mut rename = None;
        ui.add_sized(
            [w, 20.],
            TextInput::new(&format!("rename {}", current), &mut rename),
        );
        if let Some(rename) = rename {
            let _ = s.rename(editor::Rename::Part {
                view_idx: s.get_current_mesh_idx().unwrap(),
                swap: editor::DataSwap {
                    from: current.to_string(),
                    to: rename,
                },
            });
        }
    }

    if let Selection::Vertices(verts) = &mut s2.selection
        && let Some(part) = s.get_current_part_mut()
    {
        let rd2 = RefDuper;
        let part2 = unsafe { rd2.detach_mut_ref(part) };
        //let tri_verts: Vec<&VertexId> = part.filter_triangle_vertices(verts).collect();
        let verts2: Vec<&VertexId> = verts.iter().collect();
        let mut values: Vec<&mut Vec3> = part.filter_non_compute_vertices(&verts2).collect();
        let w2 = (w - ui.spacing().item_spacing.x) / 2.;
        let w3 = (w - ui.spacing().item_spacing.x * 2.) / 3.;

        ui.label("Multi-Vertex");
        multi_vec3_row(
            ui,
            &mut values,
            w3,
            || part2.clone(),
            |t| {
                self3.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        idx: self3.get_current_mesh_idx().unwrap(),
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
                    ui.label("Vertex Position");
                    vec3_row(
                        ui,
                        v3,
                        w3,
                        || part2.clone(),
                        |t| {
                            self3.add_history(editor::HistoryEntry::MeshPart(
                                light_mesh::LightMeshPartSnapshot {
                                    idx: self3.get_current_mesh_idx().unwrap(),
                                    name: self3.get_current_part_name().unwrap().to_string(),
                                    part: Box::new(t),
                                },
                            ))
                        },
                        || self4.rebuild_meshes(gl),
                    );
                }
                VertVal::C3(c3) => {
                    ui.label("Compute Position");
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
            ui.add_sized([w, 20.], TextInput::new("+ Compute Vertex", &mut comp_name));
            if let Some(name) = comp_name {
                self3.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        idx: self3.get_current_mesh_idx().unwrap(),
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
        let mut tris: Vec<[(&mut NormalId, &mut UvId); 3]> = part3
            .filter_triangles(&verts2)
            .map(|tri| {
                let [a, b, c] = &mut tri.vertices;
                [
                    (&mut a.normal, &mut a.uv),
                    (&mut b.normal, &mut b.uv),
                    (&mut c.normal, &mut c.uv),
                ]
            })
            .collect();

        if verts.len() == 3
            && let [v1, v2, v3] = verts2.as_slice()
        {
            let mut v1 = *v1;
            let mut v2 = *v2;
            let mut v3 = *v3;

            let mut hint = "+ Compute Normal";

            if tris.len() == 1
                && let Some(tri) = part4.filter_triangles(&[v1, v2, v3]).next()
            {
                let [a, b, c] = &tri.vertices;
                v1 = &a.vertex;
                v2 = &b.vertex;
                v3 = &c.vertex;
                hint = "+ Compute Normal [Tri]";
            }

            let mut comp_name = None;
            ui.add_sized([w, 20.], TextInput::new(hint, &mut comp_name));
            if let Some(name) = comp_name {
                self3.add_history(editor::HistoryEntry::MeshPart(
                    light_mesh::LightMeshPartSnapshot {
                        idx: self3.get_current_mesh_idx().unwrap(),
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
            ui.label("Multi-Triangle Data");

            ui.label("Normals");
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
                                idx: self3.get_current_mesh_idx().unwrap(),
                                name: self3.get_current_part_name().unwrap().to_string(),
                                part: Box::new(part.clone()),
                            },
                        ));
                        for tri in tris.iter_mut() {
                            *tri[idx].0 = normal.clone();
                        }
                        self3.rebuild_meshes(gl);
                    }
                }
            });

            ui.label("UVs");
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
                                idx: self3.get_current_mesh_idx().unwrap(),
                                name: self3.get_current_part_name().unwrap().to_string(),
                                part: Box::new(part.clone()),
                            },
                        ));
                        for tri in tris.iter_mut() {
                            *tri[idx].1 = uv.clone();
                        }
                        self3.rebuild_meshes(gl);
                    }
                }
            });
        }
    }
}

fn draw_edit_gl(s: &UnsafeMutRef<App>, gl: &glow::Context, vp: &Mat4) {
    if let Some((_, name, _part)) = s.get_current_part()
        && let Some(sel) = s.editor.mesh
        && let Some(mesh) = s.view.meshes.get(sel)
        && let Some(mesh) = mesh.gpu_bufs.0.get(name)
    {
        let calls = vec![MeshDrawCall {
            mesh,
            instances: vec![InstanceData::new(Mat4::IDENTITY, 1., Some([0.2, 0.2, 0.2]))],
            wireframe: s.state.wireframe,
        }];

        s.render.renderer.draw_meshes(gl, vp, &calls);

        let mut calls = Vec::new();
        if s.state.show_verts {
            calls.push(PointDrawCall {
                mesh,
                instances: vec![InstanceData::new(Mat4::IDENTITY, 1., Some([0., 1., 1.]))],
                size: 2.5,
            });
        }

        if let Some(selected) = s.render.sel_points.as_ref() {
            calls.push(PointDrawCall {
                mesh: selected,
                instances: vec![InstanceData::new(Mat4::IDENTITY, 1., Some([1., 1., 0.]))],
                size: 4.,
            });
        }

        if !calls.is_empty() {
            s.render.renderer.draw_points_batch(gl, vp, &calls);
        }
    }
}

const UV_VERT_COLORS: [egui::Color32; 3] = [
    egui::Color32::from_rgb(220, 80, 80),
    egui::Color32::from_rgb(80, 150, 220),
    egui::Color32::from_rgb(80, 200, 120),
];

const UV_HIT_RADIUS: f32 = 6.0;

fn uv_to_screen(uv: glam::Vec2, rect: egui::Rect, pan: glam::Vec2, zoom: f32) -> egui::Pos2 {
    let origin = rect.min + egui::vec2(rect.width() * 0.5, rect.height() * 0.5);
    let centered = (uv - glam::Vec2::splat(0.5) - pan) * zoom;
    origin + egui::vec2(centered.x, centered.y)
}

fn screen_to_uv(pos: egui::Pos2, rect: egui::Rect, pan: glam::Vec2, zoom: f32) -> glam::Vec2 {
    let origin = rect.min + egui::vec2(rect.width() * 0.5, rect.height() * 0.5);
    let delta = pos - origin;
    glam::Vec2::new(delta.x, delta.y) / zoom + pan + glam::Vec2::splat(0.5)
}

fn snap_uv(uv: glam::Vec2, tex_w: u32, tex_h: u32, modifiers: &egui::Modifiers) -> glam::Vec2 {
    let divisor = match (modifiers.ctrl, modifiers.shift) {
        (true, true) => 8.0,
        (true, false) => 2.0,
        (false, true) => 4.0,
        (false, false) => 1.0,
    };
    let step_x = 1.0 / (tex_w as f32 * divisor);
    let step_y = 1.0 / (tex_h as f32 * divisor);
    glam::Vec2::new(
        (uv.x / step_x).round() * step_x,
        (uv.y / step_y).round() * step_y,
    )
}

fn get_or_load_texture<'a>(
    display_id: &str,
    path: &Path,
    ctx: &egui::Context,
    cache: &'a mut HashMap<String, egui::TextureHandle>,
) -> Option<&'a egui::TextureHandle> {
    if !cache.contains_key(display_id) {
        let image = image::open(path).ok()?;
        let image = image.to_rgba8();
        let (w, h) = image.dimensions();
        let pixels = image.into_raw();
        let color_image =
            egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &pixels);
        let handle = ctx.load_texture(display_id, color_image, egui::TextureOptions::NEAREST);
        cache.insert(display_id.to_string(), handle);
    }
    cache.get(display_id)
}

pub fn draw_uv_view(s: &mut App, ui: &mut Ui, ctx: &egui::Context, gl: &glow::Context) {
    let rd = RefDuper;
    let s2 = unsafe { rd.detach_mut_ref(s) };
    let s3 = unsafe { rd.detach_mut_ref(s) };

    if let Selection::Vertices(verts) = &mut s2.selection
        && let Some(sel) = s.get_current_mesh_idx()
        && let Some(v_mesh) = s2.view.meshes.get_mut(sel)
        && let Some(part) = s3.get_current_part_mut()
        && let part2 = unsafe { rd.detach_mut_ref(part) }
        && let tris = part2.filter_triangles(verts).collect::<Vec<_>>()
        && !tris.is_empty()
    {
        let mesh = unsafe { rd.detach_mut_ref(&mut v_mesh.data) };

        let mut unique_texture_ids = tris
            .iter()
            .filter_map(|tri| {
                mesh.resolve_data(tri.material.as_ref())
                    .map(|mat| mat.texture)
            })
            .collect::<HashSet<_>>()
            .iter()
            .filter_map(|id| mesh.textures.get(&id.to_string()).map(|v| (id, v)))
            .filter_map(|(n, id)| {
                s.render
                    .renderer
                    .texture_paths
                    .get(id)
                    .map(|p| (*n, id.as_str(), p.as_path()))
            })
            .collect::<Vec<(u8, &str, &Path)>>();

        unique_texture_ids.sort();

        let mut groups: IndexMap<u8, Vec<&mut Triangle>> = IndexMap::new();
        for tri in tris.into_iter() {
            let Some(mat_id) = mesh
                .resolve_data(tri.material.as_ref())
                .map(|mat| mat.texture)
            else {
                continue;
            };
            match groups.entry(mat_id) {
                indexmap::map::Entry::Occupied(mut o) => {
                    o.get_mut().push(tri);
                }
                indexmap::map::Entry::Vacant(v) => {
                    v.insert(vec![tri]);
                }
            }
        }

        if !groups.contains_key(&s.state.ui.selected_group) {
            s.state.ui.selected_group = *groups.keys().next().unwrap_or(&0);
            s.state.ui.selected_tris.clear();
        }

        let sel_group = s.state.ui.selected_group;
        let tri_count = groups.get(&sel_group).map(|v| v.len()).unwrap_or(0);
        let sel_tri_entry = s.state.ui.selected_tris.entry(sel_group).or_insert(0);
        if *sel_tri_entry >= tri_count {
            *sel_tri_entry = 0;
        }

        let prev_group = sel_group;

        if unique_texture_ids.len() > 1 {
            ui.horizontal(|ui| {
                for (ref_id, display_id, _path) in &unique_texture_ids {
                    let selected = *ref_id == s.state.ui.selected_group;
                    if ui
                        .selectable_label(selected, format!("Tex {display_id}"))
                        .clicked()
                        && !selected
                    {
                        s.state.ui.selected_group = *ref_id;
                        s.state.ui.selected_tris.insert(*ref_id, 0);
                        s.state.ui.hovered_uv = None;
                        s.state.ui.dragging_uv = None;
                    }
                }
            });
        }

        let group_changed = prev_group != s.state.ui.selected_group;
        let sel_group = s.state.ui.selected_group;
        let sel_tri = *s.state.ui.selected_tris.get(&sel_group).unwrap_or(&0);

        let tris_in_group = groups.get(&sel_group).map(|v| v.len()).unwrap_or(0);

        if tris_in_group > 1 {
            egui::ScrollArea::horizontal()
                .id_salt(ui.id().with("tri_tabs"))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for ti in 0..tris_in_group {
                            let selected = ti == sel_tri;
                            if ui.selectable_label(selected, format!("Tri {ti}")).clicked() {
                                s.state.ui.selected_tris.insert(sel_group, ti);
                                s.state.ui.hovered_uv = None;
                                s.state.ui.dragging_uv = None;
                            }
                        }
                    });

                    if group_changed {
                        let approx_tab_w = 48.0;
                        let offset = sel_tri as f32 * approx_tab_w;
                        ui.scroll_with_delta(egui::vec2(-offset, 0.0));
                    }
                });
        }

        let spacing = ui.spacing().item_spacing.y;
        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 5.0;
        let input_height = row_h * 2.0 + spacing * 3.0 + 10.0 + 10.0 + 1.0;

        let canvas_rect = {
            let size = ui.available_rect_before_wrap();
            egui::Rect::from_min_size(
                size.min,
                egui::vec2(size.width(), (size.height() - input_height).max(32.0)),
            )
        };

        let resp = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());
        s.state.vp_rect = canvas_rect;

        let (tex_w, tex_h, tex_id_opt) = unique_texture_ids
            .iter()
            .find(|(id, _, _)| *id == sel_group)
            .and_then(|(_ref_id, display_id, path)| {
                let handle =
                    get_or_load_texture(display_id, path, ctx, &mut s.state.ui.texture_cache)?;
                let size = handle.size();
                Some((size[0] as u32, size[1] as u32, handle.id()))
            })
            .map(|(w, h, tid)| (w, h, Some(tid)))
            .unwrap_or((1, 1, None));

        if s.state.ui.uv_zoom == 0.0 {
            s.state.ui.uv_zoom = canvas_rect.width().min(canvas_rect.height()) * 0.9;
        }

        {
            let zoom = &mut s.state.ui.uv_zoom;
            let pan = &mut s.state.ui.uv_pan;

            if resp.hovered() {
                let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
                if scroll_delta != 0.0
                    && let Some(mouse_pos) = ctx.input(|i| i.pointer.hover_pos())
                {
                    let uv_before = screen_to_uv(mouse_pos, canvas_rect, *pan, *zoom);
                    let factor = (scroll_delta * 0.002).exp();
                    *zoom = (*zoom * factor).clamp(20.0, 4000.0);
                    let origin = canvas_rect.min
                        + egui::vec2(canvas_rect.width() * 0.5, canvas_rect.height() * 0.5);
                    let delta = mouse_pos - origin;
                    *pan = uv_before
                        - glam::Vec2::new(delta.x, delta.y) / *zoom
                        - glam::Vec2::splat(0.5);
                }
            }

            if resp.dragged_by(egui::PointerButton::Secondary) {
                let d = resp.drag_delta();
                pan.x -= d.x / *zoom;
                pan.y -= d.y / *zoom;
            }
        }

        let zoom_snap = s.state.ui.uv_zoom;
        let pan_snap = s.state.ui.uv_pan;

        struct VertScreenPos {
            tri_idx: usize,
            vert_idx: usize,
            screen: egui::Pos2,
        }

        let mut vert_positions: Vec<VertScreenPos> = Vec::new();
        if let Some(tris_vec) = groups.get(&sel_group)
            && let Some((ti, tri)) = tris_vec.iter().enumerate().nth(sel_tri)
        {
            for (vi, vert) in tri.vertices.iter().enumerate() {
                let uv = part.resolve_uv(&vert.uv);
                vert_positions.push(VertScreenPos {
                    tri_idx: ti,
                    vert_idx: vi,
                    screen: uv_to_screen(uv, canvas_rect, pan_snap, zoom_snap),
                });
            }
        }

        let prev_hovered = s.state.ui.hovered_uv;

        if s.state.ui.dragging_uv.is_none() {
            s.state.ui.hovered_uv = None;
            if let Some(hp) = ctx.input(|i| i.pointer.hover_pos())
                && canvas_rect.contains(hp)
            {
                let mut best_dist = UV_HIT_RADIUS;
                let mut best = None;
                for vsp in &vert_positions {
                    let d = (vsp.screen - hp).length();
                    if d < best_dist {
                        best_dist = d;
                        best = Some((vsp.tri_idx, vsp.vert_idx));
                    }
                }
                s.state.ui.hovered_uv = best;
            }
        }

        if resp.drag_started_by(egui::PointerButton::Primary) {
            s.state.ui.dragging_uv = prev_hovered;
            s.add_history(editor::HistoryEntry::MeshPart(
                light_mesh::LightMeshPartSnapshot {
                    idx: sel,
                    name: s.get_current_part_name().unwrap().to_string(),
                    part: Box::new(part.clone()),
                },
            ));
        }

        if let Some((dt, dv)) = s.state.ui.dragging_uv {
            if resp.drag_stopped() {
                let modifiers = ctx.input(|i| i.modifiers);
                if let Some(tris_vec) = groups.get_mut(&sel_group)
                    && let Some(tri) = tris_vec.get(dt)
                    && let Some(uv_ref) = part.resolve_uv_mut(&tri.vertices[dv].uv)
                {
                    *uv_ref = snap_uv(*uv_ref, tex_w, tex_h, &modifiers);
                }
                s.rebuild_meshes(gl);
                s.state.ui.dragging_uv = None;
                s.state.ui.hovered_uv = None;
            } else if resp.dragged_by(egui::PointerButton::Primary) {
                let delta_screen = resp.drag_delta();
                let delta_uv =
                    glam::Vec2::new(delta_screen.x / zoom_snap, delta_screen.y / zoom_snap);
                if let Some(tris_vec) = groups.get_mut(&sel_group)
                    && let Some(tri) = tris_vec.get(dt)
                    && let Some(uv_ref) = part.resolve_uv_mut(&tri.vertices[dv].uv)
                {
                    *uv_ref += delta_uv;
                    s.rebuild_meshes(gl);
                }
            }
        }

        let painter = ui.painter_at(canvas_rect);

        painter.rect_filled(canvas_rect, 0.0, egui::Color32::from_rgb(18, 20, 28));

        if let Some(tid) = tex_id_opt {
            let tl = uv_to_screen(glam::Vec2::new(0.0, 0.0), canvas_rect, pan_snap, zoom_snap);
            let br = uv_to_screen(glam::Vec2::new(1.0, 1.0), canvas_rect, pan_snap, zoom_snap);
            painter.image(
                tid,
                egui::Rect::from_min_max(tl, br),
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }

        if let Some(tris_vec) = groups.get(&sel_group) {
            for (ti, tri) in tris_vec.iter().enumerate() {
                let is_sel = ti == sel_tri;

                let screen_verts: [egui::Pos2; 3] = std::array::from_fn(|vi| {
                    let uv = part.resolve_uv(&tri.vertices[vi].uv);
                    uv_to_screen(uv, canvas_rect, pan_snap, zoom_snap)
                });

                let fill = if is_sel {
                    egui::Color32::from_rgba_unmultiplied(80, 130, 220, 35)
                } else {
                    egui::Color32::from_rgba_unmultiplied(80, 130, 220, 10)
                };
                painter.add(egui::Shape::convex_polygon(
                    screen_verts.to_vec(),
                    fill,
                    egui::Stroke::NONE,
                ));

                let stroke_color = if is_sel {
                    egui::Color32::from_rgba_unmultiplied(120, 170, 255, 180)
                } else {
                    egui::Color32::from_rgba_unmultiplied(80, 110, 180, 80)
                };
                let stroke = egui::Stroke::new(1.0, stroke_color);
                for i in 0..3 {
                    painter.line_segment([screen_verts[i], screen_verts[(i + 1) % 3]], stroke);
                }

                if is_sel {
                    for vi in 0..3 {
                        let col = UV_VERT_COLORS[vi];
                        let is_hov = s.state.ui.hovered_uv == Some((ti, vi));
                        let is_drag = s.state.ui.dragging_uv == Some((ti, vi));
                        let radius = if is_drag {
                            6.0
                        } else if is_hov {
                            5.5
                        } else {
                            4.0
                        };
                        let inner = if is_hov || is_drag {
                            col
                        } else {
                            egui::Color32::from_rgba_premultiplied(col.r(), col.g(), col.b(), 180)
                        };
                        painter.circle(
                            screen_verts[vi],
                            radius,
                            inner,
                            egui::Stroke::new(1.5, col),
                        );
                    }
                }
            }
        }

        let sel_tri_final = *s.state.ui.selected_tris.get(&sel_group).unwrap_or(&0);

        if let Some(tris_vec) = groups.get_mut(&sel_group)
            && let Some(tri) = tris_vec.get_mut(sel_tri_final)
        {
            ui.separator();

            ui.horizontal(|ui| {
                let col_w = (canvas_rect.width() / 3.0) - 20.0;
                for (vi, col) in UV_VERT_COLORS.iter().enumerate() {
                    egui::Frame::NONE
                        .stroke(egui::Stroke::new(1.5, *col))
                        .inner_margin(egui::Margin::same(5))
                        .show(ui, |ui| {
                            ui.set_width(col_w);
                            ui.set_max_width(col_w);

                            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                let inner_w = col_w - 10.0;
                                let half_w = (inner_w - ui.spacing().item_spacing.x) / 2.0;

                                ui.horizontal(|ui| {
                                    if let Some(uv_ref) = part.resolve_uv_mut(&tri.vertices[vi].uv)
                                    {
                                        let mut vars = std::collections::HashMap::new();
                                        vars.insert("x".to_string(), uv_ref.x);
                                        vars.insert("y".to_string(), uv_ref.y);

                                        let rx = ui.add_sized(
                                            [half_w, 18.0],
                                            MathDragValue::new(&mut uv_ref.x, &mut vars)
                                                .speed(0.001)
                                                .max_decimals(4),
                                        );
                                        if rx.drag_started() || (rx.gained_focus() && !rx.dragged())
                                        {
                                            s.add_history(editor::HistoryEntry::MeshPart(
                                                light_mesh::LightMeshPartSnapshot {
                                                    idx: sel,
                                                    name: s
                                                        .get_current_part_name()
                                                        .unwrap()
                                                        .to_string(),
                                                    part: Box::new(part.clone()),
                                                },
                                            ));
                                        }
                                        if rx.changed() {
                                            s.rebuild_meshes(gl);
                                        }

                                        let uv_ref =
                                            part.resolve_uv_mut(&tri.vertices[vi].uv).unwrap();
                                        vars.insert("x".to_string(), uv_ref.x);
                                        vars.insert("y".to_string(), uv_ref.y);

                                        let ry = ui.add_sized(
                                            [half_w, 18.0],
                                            MathDragValue::new(&mut uv_ref.y, &mut vars)
                                                .speed(0.001)
                                                .max_decimals(4),
                                        );
                                        if ry.drag_started() || (ry.gained_focus() && !ry.dragged())
                                        {
                                            s.add_history(editor::HistoryEntry::MeshPart(
                                                light_mesh::LightMeshPartSnapshot {
                                                    idx: sel,
                                                    name: s
                                                        .get_current_part_name()
                                                        .unwrap()
                                                        .to_string(),
                                                    part: Box::new(part.clone()),
                                                },
                                            ));
                                        }
                                        if ry.changed() {
                                            s.rebuild_meshes(gl);
                                        }
                                    } else {
                                        ui.add_sized([half_w, 18.0], egui::Label::new("0.0000"));
                                        ui.add_sized([half_w, 18.0], egui::Label::new("0.0000"));
                                    }
                                });

                                let valid_ids: Vec<UvId> = part.get_valid_uv_ids().collect();
                                let current = &mut tri.vertices[vi].uv;
                                let old = current.clone();
                                egui::ComboBox::new(("uv_id_combo", vi), "")
                                    .selected_text(format!("{:?}", current))
                                    .width(inner_w)
                                    .show_ui(ui, |ui| {
                                        ui.set_min_width(inner_w);
                                        for id in &valid_ids {
                                            ui.selectable_value(
                                                current,
                                                id.clone(),
                                                format!("{:?}", id),
                                            );
                                        }
                                    });
                                if *current != old {
                                    s.add_history(editor::HistoryEntry::MeshPart(
                                        light_mesh::LightMeshPartSnapshot {
                                            idx: sel,
                                            name: s.get_current_part_name().unwrap().to_string(),
                                            part: Box::new(part.clone()),
                                        },
                                    ));
                                    s.rebuild_meshes(gl);
                                }
                            });
                        });
                }
            });
        }
    } else {
        let rect = ui.available_rect_before_wrap();
        ui.add_sized(
            rect.size(),
            egui::Label::new("Select triangles to edit UVs"),
        );
    }
}

pub fn main() -> Result<(), eframe::Error> {
    let path = std::env::args().nth(1).map(PathBuf::from);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Beatcraft Mesh Editor")
            .with_inner_size([1440., 860.]),
        multisampling: 4,
        depth_buffer: 24,
        ..Default::default()
    };

    eframe::run_native(
        "Beatcraft Mesh Editor",
        options,
        Box::new(move |cc| Ok(Box::new(App::new(cc, path)))),
    )
}
