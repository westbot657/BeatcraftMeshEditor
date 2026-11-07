use egui::{Color32, Id, Response, Ui, Widget};
use keybinds::KeyInput;
use crate::keymap::{KeymapSystem, paint_hint_parts};

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
    bg: Option<Color32>,
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
            scale: 1.,
            bg: None,
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

    pub fn background(mut self, bg: Color32) -> Self {
        self.bg = Some(bg);
        self
    }

    pub fn show(self, ui: &Ui, keymaps: &KeymapSystem) {
        let Self { name, rect, response, placement, gap, always, require_active, scale, bg } = self;

        let id = Id::new(("keybind_hint", name, rect.min.x.to_bits(), rect.min.y.to_bits()));
        let layer = egui::LayerId::new(egui::Order::Foreground, id);

        if !always && !hint_is_hovered(ui, layer, response) {
            return;
        }
        if require_active && !keymaps.is_active(name) {
            return;
        }
        let Some(seq) = keymaps.get_sequences(name).and_then(|s| s.first()) else {
            return;
        };

        paint_hint(ui, layer, rect, placement, gap, scale, &[HintPart::Keys(seq.as_slice())], bg);
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

pub enum HintPart<'a> {
    Keys(&'a [KeyInput]),
    Text(&'a str),
    /// Fake bind: each label is drawn as a keycap, joined by "+".
    Labels(&'a [&'a str]),
}


fn hint_is_hovered(ui: &Ui, hint_layer: egui::LayerId, response: &Response) -> bool {
    let ctx = ui.ctx();
    ctx.input(|i| i.pointer.hover_pos()).is_some_and(|p| {
        (response.rect.contains(p)
            && ctx
            .layer_id_at(p)
            .is_none_or(|top| top == response.layer_id || top == hint_layer))
        || response.dragged()
    })
}

fn paint_hint(
    ui: &Ui,
    layer: egui::LayerId,
    rect: egui::Rect,
    placement: ExtensionCorner,
    gap: f32,
    scale: f32,
    parts: &[HintPart],
    bg: Option<Color32>,
) {
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

    // Scale about `pos` so the anchor stays fixed. Always set, so a hint that
    // returns to scale 1.0 doesn't keep an old transform.
    let scale = scale.max(0.01);
    ui.ctx().set_transform_layer(
        layer,
        egui::emath::TSTransform::new(pos.to_vec2() * (1.0 - scale), scale),
    );

    let painter = ui.ctx().layer_painter(layer);
    paint_hint_parts(ui.ctx(), &painter, parts, pos, pivot, bg);
}

use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HintItem {
    /// Plain text, drawn as-is.
    Text(Cow<'static, str>),
    /// A keybind name. Its first listed sequence is drawn as keycaps.
    Bind(&'static str),
    Fake(&'static [&'static str]),
}

impl HintItem {
    pub fn text(s: impl Into<Cow<'static, str>>) -> Self {
        Self::Text(s.into())
    }
    pub fn bind(name: &'static str) -> Self {
        Self::Bind(name)
    }
    pub fn fake(labels: &'static [&'static str]) -> Self {
        Self::Fake(labels)
    }
}

pub struct KeybindHintList<'a> {
    items: Vec<HintItem>,
    rect: egui::Rect,
    response: &'a Response,
    placement: ExtensionCorner,
    gap: f32,
    always: bool,
    require_active: bool,
    scale: f32,
    invert_visibility: bool,
    bg: Option<Color32>,
}

impl<'a> KeybindHintList<'a> {
    pub fn new(items: impl Into<Vec<HintItem>>, rect: egui::Rect, response: &'a Response) -> Self {
        Self {
            items: items.into(),
            rect,
            response,
            placement: ExtensionCorner::Interior(Corner::TopLeft),
            gap: 2.0,
            always: false,
            require_active: false,
            scale: 1.0,
            invert_visibility: false,
            bg: None,
        }
    }

    pub fn placement(mut self, placement: ExtensionCorner) -> Self {
        self.placement = placement;
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn always_visible(mut self, always: bool) -> Self {
        self.always = always;
        self
    }

    /// Hide the hint unless at least one `Bind` in the list is currently active.
    pub fn require_active(mut self, require: bool) -> Self {
        self.require_active = require;
        self
    }

    pub fn invert_visibility(mut self, invert: bool) -> Self {
        self.invert_visibility = invert;
        self
    }

    pub fn background(mut self, bg: Color32) -> Self {
        self.bg = Some(bg);
        self
    }

    pub fn show(self, ui: &Ui, keymaps: &KeymapSystem) {
        let Self { items, rect, response, placement, gap, always, require_active, scale, invert_visibility, bg } = self;

        let id = Id::new((
            "keybind_hint_list",
            &items,
            rect.min.x.to_bits(),
            rect.min.y.to_bits(),
        ));
        let layer = egui::LayerId::new(egui::Order::Foreground, id);

        let mut hint = hint_is_hovered(ui, layer, response);

        if invert_visibility {
            hint = !hint;
        }

        if !always && !hint {
            return;
        }
        if require_active
            && !items.iter().any(|i| matches!(i, HintItem::Bind(n) if keymaps.is_active(n)))
        {
            return;
        }

        // Binds that aren't registered or have no sequence are skipped.
        let parts: Vec<HintPart> = items
            .iter()
            .filter_map(|item| match item {
                HintItem::Text(s) => Some(HintPart::Text(s.as_ref())),
                HintItem::Bind(name) => keymaps
                    .get_sequences(name)
                    .and_then(|s| s.first())
                    .map(|seq| HintPart::Keys(seq.as_slice())),
                HintItem::Fake(labels) => Some(HintPart::Labels(labels)),
            })
            .collect();
        if parts.is_empty() {
            return;
        }

        paint_hint(ui, layer, rect, placement, gap, scale, &parts, bg);
    }
}

pub struct KeybindHintListWidget<'a> {
    inner: KeybindHintList<'a>,
    keymap_system: &'a KeymapSystem,
}

impl<'a> KeybindHintListWidget<'a> {
    pub fn new(
        items: impl Into<Vec<HintItem>>,
        rect: egui::Rect,
        response: &'a Response,
        keymap_system: &'a KeymapSystem,
    ) -> Self {
        Self { inner: KeybindHintList::new(items, rect, response), keymap_system }
    }

    pub fn placement(mut self, placement: ExtensionCorner) -> Self {
        self.inner.placement = placement;
        self
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.inner.scale = scale;
        self
    }
    pub fn gap(mut self, gap: f32) -> Self {
        self.inner.gap = gap;
        self
    }
    pub fn always_visible(mut self, always: bool) -> Self {
        self.inner.always = always;
        self
    }
    pub fn require_active(mut self, require: bool) -> Self {
        self.inner.require_active = require;
        self
    }
}

impl Widget for KeybindHintListWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let r = self.inner.response.clone();
        self.inner.show(ui, self.keymap_system);
        r
    }
}





