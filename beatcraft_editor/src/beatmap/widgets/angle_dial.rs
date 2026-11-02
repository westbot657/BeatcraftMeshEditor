use egui::Color32;

pub struct AngleDial<'a> {
    value: &'a mut f32,
    notches: &'a [(f32, f32, Color32)],
    image: Option<egui::ImageSource<'a>>,
    image_angle: f32,
    zero_angle: f32,
    show_line: bool,
    line_length: f32,
    circle_color: Option<Color32>,
    snap: bool,
    arc: Option<(f32, f32)>,
}

impl<'a> AngleDial<'a> {
    pub fn new(value: &'a mut f32, notches: &'a [(f32, f32, Color32)]) -> Self {
        Self {
            value,
            notches,
            image: None,
            image_angle: 0.0,
            zero_angle: 0.0,
            show_line: false,
            line_length: 1.0,
            circle_color: None,
            snap: true,
            arc: None,
        }
    }

    /// Image drawn in the center; it rotates with the value.
    pub fn image(mut self, image: impl Into<egui::ImageSource<'a>>) -> Self {
        self.image = Some(image.into());
        self
    }
    
    pub fn image_angle(mut self, angle: f32) -> Self {
        self.image_angle = angle;
        self
    }

    /// Offset applied to the line, notches and pointer mapping.
    /// With the default 0, value 0 points right; use -90 to make 0 point up.
    pub fn zero_angle(mut self, degrees: f32) -> Self {
        self.zero_angle = degrees;
        self
    }

    /// Show the selected-angle line.
    pub fn line(mut self, show: bool) -> Self {
        self.show_line = show;
        self
    }

    /// Length of the selected-angle line as a fraction of the radius, measured
    /// from the rim toward the center (1.0 touches the center, 0.0 is a dot at the rim).
    pub fn line_length(mut self, fraction: f32) -> Self {
        self.line_length = fraction.clamp(0.0, 1.0);
        self
    }

    pub fn circle_color(mut self, color: Color32) -> Self {
        self.circle_color = Some(color);
        self
    }

    /// Snap to the nearest notch while clicking/dragging (default true).
    pub fn snap(mut self, snap: bool) -> Self {
        self.snap = snap;
        self
    }

    pub fn arc(mut self, start: f32, end: f32) -> Self {
        self.arc = Some((start, end));
        self
    }
}

/// Unit vector for an on-screen angle (degrees, clockwise from +x since y points down).
fn dir(deg: f32) -> egui::Vec2 {
    let r = deg.to_radians();
    egui::vec2(r.cos(), r.sin())
}

/// Smallest absolute difference between two angles, in degrees.
fn angle_dist(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

fn in_arc(angle: f32, start: f32, end: f32) -> bool {
    let sweep = (end - start).clamp(0.0, 360.0);
    sweep >= 360.0 || (angle - start).rem_euclid(360.0) <= sweep
}

impl egui::Widget for AngleDial<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // Square from the smallest available dimension (with fallback if unbounded)
        let unit = ui.spacing().interact_size.y;
        let avail = ui.available_size();
        let x = if avail.x.is_finite() { avail.x } else { unit * 3.0 };
        let y = if avail.y.is_finite() { avail.y } else { unit * 3.0 };
        let side = x.min(y);

        let (rect, mut response) =
            ui.allocate_exact_size(egui::Vec2::splat(side), egui::Sense::click_and_drag());
        let center = rect.center();

        // replace the max_notch / radius lines
        let radius = (side * 0.5 - 1.0).max(1.0);

        // Interaction
        if response.clicked() || response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let d = pos - center;
                if d.length() > 1.0 {
                    let mut new =
                        (d.y.atan2(d.x).to_degrees() - self.zero_angle).rem_euclid(360.0);

                    let arc = self.arc;
                    let in_range = |a: f32| arc.map_or(true, |(s, e)| in_arc(a, s, e));

                    // Outside the arc: clamp to the nearer end
                    if let Some((s, e)) = arc {
                        if !in_range(new) {
                            new = if angle_dist(new, s) <= angle_dist(new, e) {
                                s.rem_euclid(360.0)
                            } else {
                                e.rem_euclid(360.0)
                            };
                        }
                    }

                    if self.snap {
                        if let Some(n) = self
                            .notches
                            .iter()
                            .filter(|n| in_range(n.0))
                            .min_by(|a, b| angle_dist(a.0, new).total_cmp(&angle_dist(b.0, new)))
                        {
                            new = n.0;
                        }
                    }
                    if new != *self.value {
                        *self.value = new;
                        response.mark_changed();
                    }
                }
            }
        }

        let value = *self.value;
        response.widget_info(|| egui::WidgetInfo::slider(ui.is_enabled(), value as f64, ""));

        if ui.is_rect_visible(rect) {
            let circle_color = self
                .circle_color
                .unwrap_or_else(|| ui.style().interact(&response).fg_stroke.color);
            let line_color = ui.visuals().strong_text_color();
            let painter = ui.painter().clone();

            // Center image, rotated by the value (inscribed in the rim so rotation stays inside)
            if let Some(src) = self.image {
                let img_side = radius * std::f32::consts::SQRT_2 * 0.9;
                let img_rect = egui::Rect::from_center_size(center, egui::Vec2::splat(img_side));
                egui::Image::new(src)
                    .rotate(value.to_radians() + self.image_angle.to_radians(), egui::Vec2::splat(0.5))
                    .paint_at(ui, img_rect);
            }

            // Rim
            match self.arc {
                Some((start, end)) => {
                    let sweep = (end - start).clamp(0.0, 360.0);
                    let segments = ((sweep / 3.0).ceil() as usize).max(2);
                    let points: Vec<egui::Pos2> = (0..=segments)
                        .map(|i| {
                            let a = start + sweep * i as f32 / segments as f32 + self.zero_angle;
                            center + dir(a) * radius
                        })
                        .collect();
                    painter.add(egui::Shape::line(
                        points,
                        egui::Stroke::new(1.0f32, circle_color),
                    ));
                }
                None => {
                    painter.circle_stroke(center, radius, egui::Stroke::new(1.0f32, circle_color));
                }
            }

            for &(angle, len, color) in self.notches {
                if let Some((s, e)) = self.arc {
                    if !in_arc(angle, s, e) {
                        continue;
                    }
                }
                // ...rest of the notch drawing unchanged
            }

            for &(angle, len, color) in self.notches {
                let d = dir(angle + self.zero_angle);
                let width = if angle_dist(angle, value) < 0.01 { 2.0 } else { 1.0f32 };
                painter.line_segment(
                    [center + d * radius, center + d * (radius - len).max(0.0)],
                    egui::Stroke::new(width, color),
                );
            }


            // Selected-angle line: starts at the rim and extends toward the center
            if self.show_line {
                const W: f32 = 2.0;
                let d = dir(value + self.zero_angle);
                let start = center + d * radius;
                let end = center + d * (radius * (1.0 - self.line_length));
                painter.line_segment([start, end], egui::Stroke::new(W, line_color));
                // Round the ends so a zero-length line still shows as a dot
                painter.circle_filled(start, W * 0.5, line_color);
                painter.circle_filled(end, W * 0.5, line_color);
            }
        }

        response
    }
}