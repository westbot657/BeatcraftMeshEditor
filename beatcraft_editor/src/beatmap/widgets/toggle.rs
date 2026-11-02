#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SwitchDirection {
    Up,
    Down,
    Left,
    #[default]
    Right,
}

pub struct ToggleSwitch<'a> {
    on: &'a mut bool,
    direction: SwitchDirection,
    on_color: egui::Color32,
    off_color: egui::Color32,
}

impl<'a> ToggleSwitch<'a> {
    pub fn new(on: &'a mut bool) -> Self {
        Self {
            on,
            direction: SwitchDirection::default(),
            on_color: egui::Color32::from_gray(0x70),
            off_color: egui::Color32::from_gray(0x30),
        }
    }

    /// The slider moves toward this direction when on, and away from it when off.
    pub fn direction(mut self, direction: SwitchDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn on_color(mut self, color: egui::Color32) -> Self {
        self.on_color = color;
        self
    }

    pub fn off_color(mut self, color: egui::Color32) -> Self {
        self.off_color = color;
        self
    }
}

/// The half of `r` that lies toward `dir`.
fn half_toward(r: egui::Rect, dir: SwitchDirection) -> egui::Rect {
    let c = r.center();
    match dir {
        SwitchDirection::Right => egui::Rect::from_min_max(egui::pos2(c.x, r.top()), r.max),
        SwitchDirection::Left => egui::Rect::from_min_max(r.min, egui::pos2(c.x, r.bottom())),
        SwitchDirection::Down => egui::Rect::from_min_max(egui::pos2(r.left(), c.y), r.max),
        SwitchDirection::Up => egui::Rect::from_min_max(r.min, egui::pos2(r.right(), c.y)),
    }
}

fn opposite(dir: SwitchDirection) -> SwitchDirection {
    match dir {
        SwitchDirection::Up => SwitchDirection::Down,
        SwitchDirection::Down => SwitchDirection::Up,
        SwitchDirection::Left => SwitchDirection::Right,
        SwitchDirection::Right => SwitchDirection::Left,
    }
}

impl egui::Widget for ToggleSwitch<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let unit = ui.spacing().interact_size.y;
        let mut size = ui.available_size();
        if !size.x.is_finite() { size.x = unit * 2.0; }
        if !size.y.is_finite() { size.y = unit * 2.0; }

        let (rect, mut response) = ui.allocate_exact_size(size, egui::Sense::click());

        if response.clicked() {
            *self.on = !*self.on;
            response.mark_changed();
        }

        let on = *self.on;
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), on, "")
        });

        if ui.is_rect_visible(rect) {
            let t = ui.ctx().animate_bool_responsive(response.id, on);

            let color = egui::Color32::from(egui::lerp(
                egui::Rgba::from(self.off_color)..=egui::Rgba::from(self.on_color),
                t,
            ));

            let painter = ui.painter();

            painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);

            const B: f32 = 1.0;
            let (l, r, tp, bt) = (rect.left(), rect.right(), rect.top(), rect.bottom());
            for edge in [
                egui::Rect::from_min_max(egui::pos2(l, tp), egui::pos2(r, tp + B)),
                egui::Rect::from_min_max(egui::pos2(l, bt - B), egui::pos2(r, bt)),
                egui::Rect::from_min_max(egui::pos2(l, tp + B), egui::pos2(l + B, bt - B)),
                egui::Rect::from_min_max(egui::pos2(r - B, tp + B), egui::pos2(r, bt - B)),
            ] {
                painter.rect_filled(edge, 0.0, color);
            }

            let inner = rect.shrink(B);
            let off_rect = half_toward(inner, opposite(self.direction));
            let on_rect = half_toward(inner, self.direction);

            let slider = egui::Rect::from_min_max(
                egui::pos2(
                    egui::lerp(off_rect.min.x..=on_rect.min.x, t),
                    egui::lerp(off_rect.min.y..=on_rect.min.y, t),
                ),
                egui::pos2(
                    egui::lerp(off_rect.max.x..=on_rect.max.x, t),
                    egui::lerp(off_rect.max.y..=on_rect.max.y, t),
                ),
            );
            painter.rect_filled(slider, 0.0, color);
        }

        response
    }
}