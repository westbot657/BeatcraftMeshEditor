use std::collections::HashSet;
use std::path::Path;

use eframe::glow;
use egui::Ui;
use indexmap::IndexMap;

use crate::data::mesh::UvId;
use crate::editor::{App, Selection};
use crate::light_mesh::Triangle;
use crate::widgets::MathDragValue;
use crate::{
    RefDuper, UV_HIT_RADIUS, UV_VERT_COLORS, editor, get_or_load_texture, light_mesh, screen_to_uv,
    snap_uv, uv_to_screen,
};

pub fn draw_uv_view(s: &mut App, ui: &mut Ui, ctx: &egui::Context, gl: &glow::Context) {
    let rd = RefDuper;
    let s2 = unsafe { rd.detach_mut_ref(s) };
    let s3 = unsafe { rd.detach_mut_ref(s) };

    if let Selection::Vertices(verts) = &mut s2.selection
        && let Some(sel) = s.get_current_mesh_id()
        && let Some(Some(v_mesh)) = s2.view.meshes.get_mut(sel)
        && let sel = sel.to_string()
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
                for (ref_id, display_id, _path) in unique_texture_ids.iter() {
                    let selected = *ref_id == s.state.ui.selected_group;
                    let tex_label = s.data.locale.get_with_args(
                        "texture-label",
                        &[("id".into(), (*display_id).into())].into(),
                    );
                    if ui.selectable_label(selected, tex_label).clicked() && !selected {
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
                            let tri_label = s.data.locale.get_with_args(
                                "triangle-label",
                                &[("id".into(), ti.into())].into(),
                            );
                            if ui.selectable_label(selected, tri_label).clicked() {
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
                    id: sel.to_string(),
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
                let stroke = egui::Stroke::new(1.0f32, stroke_color);
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
                            egui::Stroke::new(1.5f32, col),
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
                        .stroke(egui::Stroke::new(1.5f32, *col))
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
                                                    id: sel.to_string(),
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
                                                    id: sel.to_string(),
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
                                            id: sel.to_string(),
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
            egui::Label::new(s.data.locale.get("uv-edit-hint")),
        );
    }
}
