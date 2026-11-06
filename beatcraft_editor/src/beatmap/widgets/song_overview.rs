use egui::{pos2, vec2, Color32, Rect, Response, Sense, Stroke, Ui, Widget};

pub struct SongOverview<'a> {
    position: &'a mut f32,
    view: (f32, f32),
    length: Option<f32>,
    bg_thickness: f32,
    view_thickness: f32,
    bg_color: Option<Color32>,
    view_color: Option<Color32>,
    line_color: Option<Color32>,
    line_width: f32,
    rounding: f32,
    min_view_width: f32,
}

impl<'a> SongOverview<'a> {
    /// `position`: current playback position as a fraction of the song.
    /// `view`: `(start, end)` of the zoomed region as fractions of the song.
    pub fn new(position: &'a mut f32, view: (f32, f32)) -> Self {
        Self {
            position,
            view,
            length: None,
            bg_thickness: 20.0,
            view_thickness: 20.0,
            bg_color: None,
            view_color: None,
            line_color: None,
            line_width: 2.0,
            rounding: 2.0,
            min_view_width: 3.0,
        }
    }

    /// Bar length in points. Defaults to the available width.
    pub fn length(mut self, length: f32) -> Self {
        self.length = Some(length);
        self
    }

    /// Sets the thickness of both the background and the zoom rectangle.
    pub fn thickness(mut self, t: f32) -> Self {
        self.bg_thickness = t;
        self.view_thickness = t;
        self
    }

    pub fn bg_thickness(mut self, t: f32) -> Self {
        self.bg_thickness = t;
        self
    }

    pub fn view_thickness(mut self, t: f32) -> Self {
        self.view_thickness = t;
        self
    }

    pub fn bg_color(mut self, c: Color32) -> Self {
        self.bg_color = Some(c);
        self
    }

    pub fn view_color(mut self, c: Color32) -> Self {
        self.view_color = Some(c);
        self
    }

    pub fn line_color(mut self, c: Color32) -> Self {
        self.line_color = Some(c);
        self
    }

    pub fn line_width(mut self, w: f32) -> Self {
        self.line_width = w;
        self
    }

    pub fn rounding(mut self, r: f32) -> Self {
        self.rounding = r;
        self
    }

    /// Minimum on-screen width of the zoom rectangle, so it stays visible
    /// when zoomed in very far.
    pub fn min_view_width(mut self, w: f32) -> Self {
        self.min_view_width = w;
        self
    }
}

impl Widget for SongOverview<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let length = self.length.unwrap_or_else(|| ui.available_width()).max(0.0);
        let height = self.bg_thickness.max(self.view_thickness);
        let (rect, mut response) =
            ui.allocate_exact_size(vec2(length, height), Sense::click_and_drag());

        if length > 0.0 {
            if let Some(pos) = response.interact_pointer_pos() {
                let new = ((pos.x - rect.left()) / length).clamp(0.0, 1.0);
                if new != *self.position {
                    *self.position = new;
                    response.mark_changed();
                }
            }
        }

        if ui.is_rect_visible(rect) {
            let bg_color = self.bg_color.unwrap_or(ui.visuals().extreme_bg_color);
            let view_color = self.view_color.unwrap_or(Color32::from_white_alpha(40));
            let line_color = self.line_color.unwrap_or(ui.visuals().strong_text_color());
            let cy = rect.center().y;

            let bg_rect = Rect::from_center_size(rect.center(), vec2(length, self.bg_thickness));

            let a = self.view.0.min(self.view.1).clamp(0.0, 1.0);
            let b = self.view.0.max(self.view.1).clamp(0.0, 1.0);
            let mut x0 = rect.left() + a * length;
            let mut x1 = rect.left() + b * length;
            let min_w = self.min_view_width.min(length);
            if x1 - x0 < min_w {
                let mid = (x0 + x1) / 2.0;
                x0 = (mid - min_w / 2.0).clamp(rect.left(), rect.right() - min_w);
                x1 = x0 + min_w;
            }
            let view_rect = Rect::from_min_max(
                pos2(x0, cy - self.view_thickness / 2.0),
                pos2(x1, cy + self.view_thickness / 2.0),
            );

            let p = if self.position.is_finite() {
                self.position.clamp(0.0, 1.0)
            } else {
                0.0
            };
            let half = (self.line_width / 2.0).min(length / 2.0);
            let line_x = (rect.left() + p * length).clamp(rect.left() + half, rect.right() - half);

            let painter = ui.painter();
            painter.rect_filled(bg_rect, self.rounding, bg_color);
            painter.rect_filled(view_rect, self.rounding, view_color);
            painter.line_segment(
                [pos2(line_x, rect.top()), pos2(line_x, rect.bottom())],
                Stroke::new(self.line_width, line_color),
            );
        }

        response
    }
}