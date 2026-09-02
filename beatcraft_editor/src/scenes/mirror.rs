use eframe::glow;
use egui::{Pos2, Ui};
use glam::Vec2;

use crate::{RefDuper, editor};
use crate::editor::App;
use crate::ui_elements::vec2_row;

pub fn draw_mirror_view(
    s: &mut App,
    ui: &mut Ui,
    ctx: &egui::Context,
    gl: &glow::Context,
    shift: bool,
    ctrl: bool,
) {
    let rd = RefDuper;
    let s2 = unsafe { rd.detach_mut_ref(s) };
    let s3 = unsafe { rd.detach_mut_ref(s) };
    let s4 = unsafe { rd.detach_mut_ref(s) };

    let total_verts = s.view.mirror_geometry.len();
    let num_tris = total_verts.div_ceil(3);

    let available = ui.available_size();
    let panel_height = available.y - 60.0;

    ui.horizontal(|ui| {
        let visual_width = available.x * 0.6;
        let (visual_rect, visual_response) = ui.allocate_exact_size(
            (visual_width, panel_height).into(),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(visual_rect);
        painter.rect_filled(visual_rect, 0.0, egui::Color32::from_rgb(30, 30, 30));

        let me = &mut s.state.ui.mirror_editor;
        let zoom = me.zoom;
        let pan = me.pan;

        let to_screen = |p: Vec2| -> Pos2 {
            let cx = visual_rect.center().x;
            let cy = visual_rect.center().y;
            Pos2::new(cx + (-p.x + pan.x) * zoom, cy + (-p.y + pan.y) * zoom)
        };

        let from_screen = |p: Pos2| -> Vec2 {
            let cx = visual_rect.center().x;
            let cy = visual_rect.center().y;
            Vec2::new(-((p.x - cx) / zoom - pan.x), -((p.y - cy) / zoom - pan.y))
        };

        if visual_response.dragged_by(egui::PointerButton::Middle)
            || visual_response.dragged_by(egui::PointerButton::Secondary)
        {
            let delta = visual_response.drag_delta();
            let me = &mut s.state.ui.mirror_editor;
            me.pan += Vec2::new(delta.x, delta.y) / me.zoom;
        }

        if visual_response.hovered() {
            let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let me = &mut s.state.ui.mirror_editor;
                me.zoom = (me.zoom * (1.0 + scroll * 0.001)).clamp(0.01, 500.0);
            }
        }

        let pointer_pos = ctx.input(|i| i.pointer.interact_pos());
        let primary_down = ctx.input(|i| i.pointer.primary_down());
        let primary_released = ctx.input(|i| i.pointer.primary_released());

        {
            let me = &mut s.state.ui.mirror_editor;

            if primary_released && let Some((ti, vi)) = me.dragging_vertex {
                let idx = ti * 3 + vi;
                if idx < s.view.mirror_geometry.len() {
                    let v = s.view.mirror_geometry[idx];
                    s.view.mirror_geometry[idx] =
                        Vec2::new((v.x / 0.25).round() * 0.25, (v.y / 0.25).round() * 0.25);
                    s2.rebuild_meshes(gl);
                }
                me.dragging_vertex = None;
                me.drag_start_pos = None;
            }
        }

        if primary_down {
            let me = &mut s.state.ui.mirror_editor;
            if let Some(drag_vert) = me.dragging_vertex
                && let Some(pp) = pointer_pos
                && visual_rect.contains(pp)
            {
                let world_pos = from_screen(pp);
                let idx = drag_vert.0 * 3 + drag_vert.1;
                if idx < s.view.mirror_geometry.len() {
                    s.view.mirror_geometry[idx] = world_pos;
                }
                s.rebuild_meshes(gl);
            }
        }

        if (visual_response.clicked() || visual_response.drag_started())
            && let Some(pp) = pointer_pos
        {
            let world_pos = from_screen(pp);
            let hit_radius = 16.0 / zoom;

            let mut hit: Option<(usize, usize)> = None;
            'outer: for ti in 0..num_tris {
                let start = ti * 3;
                let end = (start + 3).min(total_verts);
                for vi in 0..(end - start) {
                    let v = s.view.mirror_geometry[start + vi];
                    if (v - world_pos).length() < hit_radius {
                        hit = Some((ti, vi));
                        break 'outer;
                    }
                }
            }

            let me = &mut s.state.ui.mirror_editor;

            if let Some(h) = hit {
                if visual_response.drag_started() {
                    me.dragging_vertex = Some(h);
                }
                if shift && ctrl {
                    if let Some(pos) = me.selected.iter().position(|&x| x == h) {
                        me.selected.remove(pos);
                    } else {
                        me.selected.push(h);
                    }
                } else if shift {
                    if !me.selected.contains(&h) {
                        me.selected.push(h);
                    }
                } else {
                    me.selected = vec![h];
                }
            } else if !shift && !ctrl {
                me.selected.clear();
            }
        }

        let active_tri = s.state.ui.mirror_editor.active_tri;
        let selected = s.state.ui.mirror_editor.selected.clone();

        for ti in 0..num_tris {
            let start = ti * 3;
            let end = (start + 3).min(total_verts);
            let verts: Vec<Vec2> = s.view.mirror_geometry[start..end].to_vec();

            let is_active = ti == active_tri;
            let alpha = if is_active { 255 } else { 80 };
            let fill = egui::Color32::from_rgba_unmultiplied(51, 76, 204, alpha);
            let stroke_color = egui::Color32::from_rgba_unmultiplied(100, 140, 255, alpha);

            if verts.len() == 3 {
                let pts = [
                    to_screen(verts[0]),
                    to_screen(verts[1]),
                    to_screen(verts[2]),
                ];
                painter.add(egui::Shape::convex_polygon(
                    pts.to_vec(),
                    fill,
                    egui::Stroke::NONE,
                ));
                painter.line_segment([pts[0], pts[1]], egui::Stroke::new(1.0, stroke_color));
                painter.line_segment([pts[1], pts[2]], egui::Stroke::new(1.0, stroke_color));
                painter.line_segment([pts[2], pts[0]], egui::Stroke::new(1.0, stroke_color));
            } else {
                for i in 0..verts.len().saturating_sub(1) {
                    painter.line_segment(
                        [to_screen(verts[i]), to_screen(verts[i + 1])],
                        egui::Stroke::new(1.0, stroke_color),
                    );
                }
            }

            for vi in 0..(end - start) {
                let v = s.view.mirror_geometry[start + vi];
                let sp = to_screen(v);
                let is_selected = selected.contains(&(ti, vi));
                let color = if is_selected {
                    egui::Color32::YELLOW
                } else if is_active {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::from_rgba_unmultiplied(180, 180, 180, alpha)
                };
                painter.circle_filled(sp, 4.0, color);
            }
        }

        ui.allocate_rect(visual_rect, egui::Sense::hover());

        ui.vertical(|ui| {
            let w = available.x * 0.4 - 8.0;
            let w2 = w / 2. - 5.;
            ui.set_min_size((w, panel_height).into());
            for vert in s.view.mirror_geometry.iter_mut() {
                vec2_row(
                    ui,
                    vert,
                    w2,
                    || s2.view.mirror_geometry.clone(),
                    |g| {
                        s3.add_history(editor::HistoryEntry::Mirror(
                            s3.view.mirror_id.clone(),
                            s3.view.mirror_path.clone(),
                            g,
                        ))
                    },
                    || s4.rebuild_meshes(gl),
                );
            }
        });
    });

    ui.horizontal(|ui| {
        for ti in 0..num_tris {
            let start = ti * 3;
            let end = (start + 3).min(total_verts);
            let count = end - start;
            let label = if count == 3 {
                s.data
                    .locale
                    .get_with_args("triangle-label", &[("id".into(), ti.into())].into())
            } else {
                s.data.locale.get_with_args(
                    "counted-triangle-label",
                    &[("id".into(), ti.into()), ("count".into(), count.into())].into(),
                )
            };
            let active = s.state.ui.mirror_editor.active_tri == ti;
            if ui.selectable_label(active, label).clicked() {
                s.state.ui.mirror_editor.active_tri = ti;
            }
        }
    });

    ui.horizontal(|ui| {
        if ui
            .button(s.data.locale.get("mirror-add-triangle"))
            .clicked()
        {
            let me = &s.state.ui.mirror_editor;
            let cx = me.pan.x;
            let cy = me.pan.y;
            let size = 50.0 / me.zoom;
            s.view.mirror_geometry.push(Vec2::new(cx, cy + size));
            s.view.mirror_geometry.push(Vec2::new(cx - size, cy - size));
            s.view.mirror_geometry.push(Vec2::new(cx + size, cy - size));
            let new_tri = (s.view.mirror_geometry.len() - 1) / 3;
            s.state.ui.mirror_editor.active_tri = new_tri;
            s.rebuild_meshes(gl);
        }

        if ui.button(s.data.locale.get("mirror-add-vertex")).clicked() {
            let me = &s.state.ui.mirror_editor;
            let cx = me.pan.x;
            let cy = me.pan.y;
            s.view.mirror_geometry.push(Vec2::new(cx, cy));
            s.rebuild_meshes(gl);
        }
    });
}
