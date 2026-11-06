use egui::emath::Numeric;
use egui::text::{CCursor, CCursorRange};
use egui::{
    pos2, vec2, Align, Align2, Color32, Direction, FontId, Layout, Rect, Response, Sense, Ui,
    UiBuilder, Widget,
};
use std::ops::RangeInclusive;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct BarSlider<'a, T: Numeric> {
    value: &'a mut T,
    range: RangeInclusive<T>,
    orientation: Orientation,
    length: f32,
    bg_thickness: f32,
    fill_thickness: f32,
    bg_color: Option<Color32>,
    fill_color: Option<Color32>,
    rounding: f32,
    step: Option<f64>,
    show_value: bool,
    decimals: usize,
    suffix: String,
    text_color: Option<Color32>,
    text_offset: f32,
    font: Option<FontId>,
    text_padding: f32,
    editable: bool,
}

impl<'a, T: Numeric> BarSlider<'a, T> {
    pub fn new(value: &'a mut T, range: RangeInclusive<T>) -> Self {
        Self {
            value,
            range,
            orientation: Orientation::Horizontal,
            length: 200.0,
            bg_thickness: 20.0,
            fill_thickness: 20.0,
            bg_color: None,
            fill_color: None,
            rounding: 2.0,
            step: None,
            show_value: false,
            decimals: 1,
            suffix: String::new(),
            text_color: None,
            text_offset: 0.0,
            font: None,
            text_padding: 4.0,
            editable: true,
        }
    }

    /// Length of the bar along its main axis, in points.
    pub fn length(mut self, length: f32) -> Self {
        self.length = length;
        self
    }

    /// Thickness (cross-axis size) of the background rectangle.
    pub fn bg_thickness(mut self, t: f32) -> Self {
        self.bg_thickness = t;
        self
    }

    /// Thickness (cross-axis size) of the filled rectangle.
    /// Both rectangles are centered on the cross axis.
    pub fn fill_thickness(mut self, t: f32) -> Self {
        self.fill_thickness = t;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.orientation = Orientation::Vertical;
        self
    }

    pub fn orientation(mut self, o: Orientation) -> Self {
        self.orientation = o;
        self
    }

    pub fn bg_color(mut self, c: Color32) -> Self {
        self.bg_color = Some(c);
        self
    }

    pub fn fill_color(mut self, c: Color32) -> Self {
        self.fill_color = Some(c);
        self
    }

    pub fn rounding(mut self, r: f32) -> Self {
        self.rounding = r;
        self
    }

    /// Snap dragged values to multiples of `step` (measured from the range start).
    pub fn step_by(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// Show the numeric value on the bar.
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    pub fn text_color(mut self, c: Color32) -> Self {
        self.text_color = Some(c);
        self
    }

    /// Shift the label (and the text box while editing) along the bar's cross
    /// axis, in points: vertically for horizontal bars, horizontally for
    /// vertical ones.
    pub fn text_offset(mut self, offset: f32) -> Self {
        self.text_offset = offset;
        self
    }

    pub fn font(mut self, font: FontId) -> Self {
        self.font = Some(font);
        self
    }

    /// Minimum gap between the text and the bar's ends.
    pub fn text_padding(mut self, p: f32) -> Self {
        self.text_padding = p;
        self
    }

    /// Whether double-clicking lets you type a value (default: true).
    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }
}

fn clamp_or_center(v: f32, min: f32, max: f32, fallback: f32) -> f32 {
    if min <= max {
        v.clamp(min, max)
    } else {
        fallback
    }
}

fn apply<T: Numeric>(value: &mut T, new: f64) -> bool {
    let new_t = T::from_f64(new);
    if new_t.to_f64() != value.to_f64() {
        *value = new_t;
        true
    } else {
        false
    }
}

impl<T: Numeric> Widget for BarSlider<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let horizontal = self.orientation == Orientation::Horizontal;
        let cross = self.bg_thickness.max(self.fill_thickness);
        let size = if horizontal {
            vec2(self.length, cross)
        } else {
            vec2(cross, self.length)
        };

        let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());

        let lo = self.range.start().to_f64();
        let hi = self.range.end().to_f64();
        let (vmin, vmax) = (lo.min(hi), lo.max(hi));
        let span = hi - lo;

        let font = self
            .font
            .clone()
            .unwrap_or_else(|| egui::TextStyle::Body.resolve(ui.style()));
        let color = self.text_color.unwrap_or(ui.visuals().text_color());

        let edit_id = response.id.with("bar_slider_edit");
        let prev_id = response.id.with("bar_slider_prev");
        let mut buffer: Option<String> = ui.data(|d| d.get_temp::<String>(edit_id));
        let mut just_started = false;

        if buffer.is_none() {
            if self.editable
                && response.is_pointer_button_down_on()
                && ui.input(|i| i.pointer.any_pressed())
            {
                let now = ui.input(|i| i.time);
                let kept = ui
                    .data(|d| d.get_temp::<(f64, f64)>(prev_id))
                    .filter(|(_, t)| now - t < 0.4);
                let prev = kept.map(|(p, _)| p).unwrap_or(self.value.to_f64());
                ui.data_mut(|d| d.insert_temp(prev_id, (prev, now)));
            }

            if self.editable && response.double_clicked() {
                let now = ui.input(|i| i.time);
                if let Some((prev, t)) = ui.data(|d| d.get_temp::<(f64, f64)>(prev_id)) {
                    if now - t < 1.0 && apply(self.value, prev) {
                        response.mark_changed();
                    }
                }
                let v = self.value.to_f64().clamp(vmin, vmax);
                buffer = Some(format!("{:.*}", self.decimals, v));
                just_started = true;
            } else if let Some(pos) = response.interact_pointer_pos() {
                let t = if horizontal {
                    (pos.x - rect.left()) / rect.width()
                } else {
                    (rect.bottom() - pos.y) / rect.height()
                };
                let mut new = lo + (t.clamp(0.0, 1.0) as f64) * span;
                if let Some(step) = self.step {
                    if step > 0.0 {
                        new = lo + ((new - lo) / step).round() * step;
                        new = new.clamp(vmin, vmax);
                    }
                }
                if apply(self.value, new) {
                    response.mark_changed();
                }
            }
        }

        let editing = buffer.is_some();

        if ui.is_rect_visible(rect) {
            let v = self.value.to_f64().clamp(vmin, vmax);
            let t = if span.abs() > f64::EPSILON {
                ((v - lo) / span) as f32
            } else {
                0.0
            };

            let bg_color = self.bg_color.unwrap_or(ui.visuals().extreme_bg_color);
            let fill_color = self.fill_color.unwrap_or(ui.visuals().selection.bg_fill);
            let center = rect.center();

            let bg_rect = if horizontal {
                Rect::from_center_size(center, vec2(self.length, self.bg_thickness))
            } else {
                Rect::from_center_size(center, vec2(self.bg_thickness, self.length))
            };

            let fill_rect = if horizontal {
                Rect::from_min_max(
                    pos2(rect.left(), center.y - self.fill_thickness / 2.0),
                    pos2(rect.left() + t * self.length, center.y + self.fill_thickness / 2.0),
                )
            } else {
                Rect::from_min_max(
                    pos2(center.x - self.fill_thickness / 2.0, rect.bottom() - t * self.length),
                    pos2(center.x + self.fill_thickness / 2.0, rect.bottom()),
                )
            };

            let painter = ui.painter();
            painter.rect_filled(bg_rect, self.rounding, bg_color);
            if fill_rect.width() > 0.0 && fill_rect.height() > 0.0 {
                painter.rect_filled(fill_rect, self.rounding, fill_color);
            }

            if self.show_value && !editing {
                let text = format!("{:.*}{}", self.decimals, v, self.suffix);

                let galley = painter.layout_no_wrap(text.clone(), font.clone(), color);
                let text_size = galley.size();

                let text_center = if horizontal {
                    let handle_x = rect.left() + t * self.length;
                    let half = text_size.x / 2.0 + self.text_padding;
                    let x = clamp_or_center(
                        handle_x,
                        rect.left() + half,
                        rect.right() - half,
                        center.x,
                    );
                    pos2(x, center.y + self.text_offset)
                } else {
                    let handle_y = rect.bottom() - t * self.length;
                    let half = text_size.y / 2.0 + self.text_padding;
                    let y = clamp_or_center(
                        handle_y,
                        rect.top() + half,
                        rect.bottom() - half,
                        center.y,
                    );
                    pos2(center.x + self.text_offset, y)
                };

                painter.text(text_center, Align2::CENTER_CENTER, text, font.clone(), color);
            }
        }

        if let Some(mut buf) = buffer {
            let text_shift = if horizontal {
                vec2(0.0, self.text_offset)
            } else {
                vec2(self.text_offset, 0.0)
            };
            let mut child = ui.new_child(
                UiBuilder::new()
                    .max_rect(rect.translate(text_shift))
                    .layout(Layout::centered_and_justified(Direction::TopDown)),
            );
            let mut out = egui::TextEdit::singleline(&mut buf)
                .frame(false)
                .font(font)
                .text_color(color)
                .horizontal_align(Align::Center)
                .vertical_align(Align::Center)
                .desired_width(rect.width())
                .show(&mut child);

            if just_started {
                out.response.request_focus();
                let n = buf.chars().count();
                out.state
                    .cursor
                    .set_char_range(Some(CCursorRange::two(CCursor::new(0), CCursor::new(n))));
                out.state.store(ui.ctx(), out.response.id);
            }

            let escaped = ui.input(|i| i.key_pressed(egui::Key::Escape));
            let finished = !just_started && !out.response.has_focus();

            if finished {
                if !escaped {
                    if let Ok(x) = buf.trim().replace(',', ".").parse::<f64>() {
                        if x.is_finite() && apply(self.value, x.clamp(vmin, vmax)) {
                            response.mark_changed();
                        }
                    }
                }
                ui.data_mut(|d| d.remove::<String>(edit_id));
            } else {
                ui.data_mut(|d| d.insert_temp(edit_id, buf));
            }
        }

        response
    }
}