use egui::{Id, Response, Ui, Widget};
use crate::keymap::{paint_key_seq_hint, KeymapSystem};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    BottomLeft,
    TopRight,
    BottomRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtensionAxis {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtensionCorner {
    /// Sits inside the target, tucked into the corner.
    Interior(Corner),
    /// Sticks out of the target at the corner, along the given axis.
    Exterior(Corner, ExtensionAxis),
}

impl Corner {
    fn is_left(self) -> bool {
        matches!(self, Self::TopLeft | Self::BottomLeft)
    }
    fn is_top(self) -> bool {
        matches!(self, Self::TopLeft | Self::TopRight)
    }
    fn point(self, r: egui::Rect) -> egui::Pos2 {
        match self {
            Self::TopLeft => r.left_top(),
            Self::BottomLeft => r.left_bottom(),
            Self::TopRight => r.right_top(),
            Self::BottomRight => r.right_bottom(),
        }
    }
}

pub struct KeybindHint<'a> {
    name: &'static str,
    rect: egui::Rect,
    response: &'a Response,
    placement: ExtensionCorner,
    gap: f32,
    always: bool,
    require_active: bool,
    scale: f32,
}

impl<'a> KeybindHint<'a> {
    /// `rect` is where the hint attaches; `response` decides when it shows
    /// (it can belong to a bigger area than `rect`, e.g. a whole panel).
    pub fn new(name: &'static str, rect: egui::Rect, response: &'a Response) -> Self {
        Self {
            name,
            rect,
            response,
            placement: ExtensionCorner::Interior(Corner::TopLeft),
            gap: 2.0,
            always: false,
            require_active: false,
            scale: 1.
        }
    }

    pub fn placement(mut self, placement: ExtensionCorner) -> Self {
        self.placement = placement;
        self
    }

    /// Uniform size multiplier for the whole hint. The anchor corner stays put.
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Interior: inset from the corner. Exterior: distance outward from the corner.
    /// Negative values make an exterior hint overlap the target.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Show even when the target isn't hovered.
    pub fn always_visible(mut self, always: bool) -> Self {
        self.always = always;
        self
    }

    /// Hide the hint while the bind's context condition is inactive.
    pub fn require_active(mut self, require: bool) -> Self {
        self.require_active = require;
        self
    }

    pub fn show(self, ui: &Ui, keymaps: &KeymapSystem) {
        let Self { name, rect, response, placement, gap, always, require_active, scale } = self;

        let hint_id = Id::new((
            "keybind_hint",
            name,
            rect.min.x.to_bits(),
            rect.min.y.to_bits(),
        ));
        let hint_layer = egui::LayerId::new(egui::Order::Foreground, hint_id);

        if !always {
            let ctx = ui.ctx();
            let over = ctx
                .input(|i| i.pointer.hover_pos())
                .is_some_and(|p| {
                    response.rect.contains(p)
                        && ctx
                        .layer_id_at(p)
                        .is_none_or(|top| top == response.layer_id || top == hint_layer)
                });
            if !over {
                return;
            }
        }
        if require_active && !keymaps.is_active(name) {
            return;
        }
        let Some(seq) = keymaps.get_sequences(name).and_then(|s| s.first()) else {
            return;
        };

        // (anchor corner, direction away from it, which side of the hint touches the anchor)
        let mm = |b: bool| if b { egui::Align::Min } else { egui::Align::Max };
        let (corner, dir, pivot) = match placement {
            ExtensionCorner::Interior(c) => (
                c,
                egui::vec2(if c.is_left() { 1.0 } else { -1.0 }, if c.is_top() { 1.0 } else { -1.0 }),
                egui::Align2([mm(c.is_left()), mm(c.is_top())]),
            ),
            ExtensionCorner::Exterior(c, ExtensionAxis::Vertical) => (
                c,
                egui::vec2(0.0, if c.is_top() { -1.0 } else { 1.0 }),
                egui::Align2([mm(c.is_left()), mm(!c.is_top())]),
            ),
            ExtensionCorner::Exterior(c, ExtensionAxis::Horizontal) => (
                c,
                egui::vec2(if c.is_left() { -1.0 } else { 1.0 }, 0.0),
                egui::Align2([mm(!c.is_left()), mm(c.is_top())]),
            ),
        };
        let pos = corner.point(rect) + dir * gap;

        let id = hint_id;

        // Scale about `pos` so the anchor stays fixed: p' = scale * p + pos * (1 - scale)
        let scale = scale.max(0.01);
        if (scale - 1.0).abs() > f32::EPSILON {
            let layer = egui::LayerId::new(egui::Order::Foreground, id);
            ui.ctx().set_transform_layer(
                layer,
                egui::emath::TSTransform::new(pos.to_vec2() * (1.0 - scale), scale),
            );
        }
        // Tooltip order draws above everything, including Foreground windows.
        let layer = egui::LayerId::new(egui::Order::Tooltip, id);

        // Always set the transform, so a hint that goes back to scale 1.0 doesn't keep an old one.
        // Scales about `pos`, so the anchor stays fixed.
        let scale = scale.max(0.01);
        ui.ctx().set_transform_layer(
            layer,
            egui::emath::TSTransform::new(pos.to_vec2() * (1.0 - scale), scale),
        );

        let painter = ui.ctx().layer_painter(layer);
        paint_key_seq_hint(ui.ctx(), &painter, seq.as_slice(), pos, pivot);
    }
}

pub struct KeybindHintWidget<'a> {
    inner: KeybindHint<'a>,
    keymap_system: &'a KeymapSystem,
}

impl<'a> KeybindHintWidget<'a> {
    pub fn new(name: &'static str, rect: egui::Rect, response: &'a Response, keymap_system: &'a KeymapSystem) -> Self {
        Self {
            inner: KeybindHint::new(name, rect, response),
            keymap_system,
        }
    }

    pub fn placement(mut self, placement: ExtensionCorner) -> Self {
        self.inner.placement = placement;
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.inner.scale = scale;
        self
    }

    /// Interior: inset from the corner. Exterior: distance outward from the corner.
    /// Negative values make an exterior hint overlap the target.
    pub fn gap(mut self, gap: f32) -> Self {
        self.inner.gap = gap;
        self
    }

    /// Show even when the target isn't hovered.
    pub fn always_visible(mut self, always: bool) -> Self {
        self.inner.always = always;
        self
    }

    /// Hide the hint while the bind's context condition is inactive.
    pub fn require_active(mut self, require: bool) -> Self {
        self.inner.require_active = require;
        self
    }
}

impl<'a> Widget for KeybindHintWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let r = self.inner.response.clone();
        self.inner.show(ui, self.keymap_system);
        r
    }
}


