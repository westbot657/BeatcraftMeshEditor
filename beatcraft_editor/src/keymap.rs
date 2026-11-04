use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::fmt::{Display, Formatter};
use std::ops::{BitAnd, BitOr, Not};
use std::time::Duration;
use bitflags::bitflags;
use egui::{Event, Id, Modifiers, Response, Ui, Widget};
use keybinds::{Key, KeyInput, KeySeq, Match, Mods};
use crate::{DB_KEYMAP, DB_LOGIC};

/// Use bitwise operators to build: | & !
/// use check on the final object to check the built expression
#[derive(Clone, Debug, PartialEq)]
pub enum ContextCondition {
    Always,
    Ctx(&'static str),
    Or(Vec<Self>),
    And(Vec<Self>),
    Not(Box<Self>),
}

impl ContextCondition {
    pub fn check(&self, context: &[&str]) -> bool {
        self.eval(&|c| context.contains(&c))
    }

    fn eval(&self, active: &dyn Fn(&str) -> bool) -> bool {
        match self {
            Self::Always => true,
            Self::Ctx(c) => active(c),
            Self::Or(v) => v.iter().any(|c| c.eval(active)),
            Self::And(v) => v.iter().all(|c| c.eval(active)),
            Self::Not(s) => !s.eval(active),
        }
    }

    fn collect_names(&self, out: &mut Vec<&'static str>) {
        match self {
            Self::Always => {}
            Self::Ctx(c) => out.push(c),
            Self::Or(v) | Self::And(v) => v.iter().for_each(|c| c.collect_names(out)),
            Self::Not(s) => s.collect_names(out),
        }
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        let mut names = Vec::new();
        self.collect_names(&mut names);
        other.collect_names(&mut names);
        names.sort_unstable();
        names.dedup();
        if names.len() > 20 {
            return true;
        }
        (0u32..1 << names.len()).any(|mask| {
            let active = |n: &str| names.iter().position(|x| *x == n).is_some_and(|i| mask >> i & 1 == 1);
            self.eval(&active) && other.eval(&active)
        })
    }
}

impl<T: Into<Self>> BitAnd<T> for ContextCondition {
    type Output = Self;
    fn bitand(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        if let Self::And(mut v) = self {
            if let Self::And(mut v2) = rhs {
                v.append(&mut v2);
            } else {
                v.push(rhs);
            }
            Self::And(v)
        } else if let Self::And(mut v) = rhs {
            v.push(self);
            Self::And(v)
        } else {
            Self::And(vec![self, rhs])
        }
    }
}

impl<T: Into<Self>> BitOr<T> for ContextCondition {
    type Output = Self;
    fn bitor(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        if let Self::Or(mut v) = self {
            if let Self::Or(mut v2) = rhs {
                v.append(&mut v2);
            } else {
                v.push(rhs);
            }
            Self::Or(v)
        } else if let Self::Or(mut v) = rhs {
            v.push(self);
            Self::Or(v)
        } else {
            Self::Or(vec![self, rhs])
        }
    }
}

impl Not for ContextCondition {
    type Output = Self;
    fn not(self) -> Self::Output {
        if let Self::Not(s) = self {
            *s
        } else {
            Self::Not(Box::new(self))
        }
    }
}

impl Display for ContextCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Always => write!(f, "always"),
            Self::Ctx(c) => write!(f, "<{}>", c),
            Self::And(a) => {
                write!(f, "(")?;
                for (i, c) in a.iter().enumerate() {
                    c.fmt(f)?;
                    if i < a.len()-1 {
                        write!(f, " & ")?;
                    }
                }
                write!(f, ")")
            }
            Self::Or(a) => {
                write!(f, "(")?;
                for (i, c) in a.iter().enumerate() {
                    c.fmt(f)?;
                    if i < a.len()-1 {
                        write!(f, " | ")?;
                    }
                }
                write!(f, ")")
            }
            Self::Not(c) => {
                write!(f, "!")?;
                c.fmt(f)
            }
        }
    }
}

impl From<&'static str> for ContextCondition {
    fn from(value: &'static str) -> Self {
        Self::Ctx(value)
    }
}

pub fn ctx(c: &'static str) -> ContextCondition {
    ContextCondition::Ctx(c)
}

pub const ALWAYS: ContextCondition = ContextCondition::Always;

trait ModConv {
    fn to_mods(&self) -> Mods;
}
impl ModConv for Modifiers {
    fn to_mods(&self) -> Mods {
        let mut out = Mods::NONE;
        if self.ctrl {
            out |= Mods::CTRL;
        }
        if self.alt {
            out |= Mods::ALT;
        }
        if self.shift {
            out |= Mods::SHIFT;
        }
        if self.mac_cmd {
            out |= Mods::CMD;
        }
        out
    }
}

const NAMED_KEYS: &[(egui::Key, Key)] = &[
    (egui::Key::ArrowUp, Key::Up),
    (egui::Key::ArrowDown, Key::Down),
    (egui::Key::ArrowLeft, Key::Left),
    (egui::Key::ArrowRight, Key::Right),
    (egui::Key::Enter, Key::Enter),
    (egui::Key::Backspace, Key::Backspace),
    (egui::Key::Delete, Key::Delete),
    (egui::Key::Home, Key::Home),
    (egui::Key::End, Key::End),
    (egui::Key::PageUp, Key::PageUp),
    (egui::Key::PageDown, Key::PageDown),
    (egui::Key::Escape, Key::Esc),
    (egui::Key::Tab, Key::Tab),
    (egui::Key::Insert, Key::Insert),
    (egui::Key::F1, Key::F1),
    (egui::Key::F2, Key::F2),
    (egui::Key::F3, Key::F3),
    (egui::Key::F4, Key::F4),
    (egui::Key::F5, Key::F5),
    (egui::Key::F6, Key::F6),
    (egui::Key::F7, Key::F7),
    (egui::Key::F8, Key::F8),
    (egui::Key::F9, Key::F9),
    (egui::Key::F10, Key::F10),
    (egui::Key::F11, Key::F11),
    (egui::Key::F12, Key::F12),
];

fn named_key(k: egui::Key) -> Option<Key> {
    NAMED_KEYS.iter().find(|(e, _)| *e == k).map(|(_, n)| *n)
}

fn to_egui_key(k: Key) -> Option<egui::Key> {
    match k {
        Key::Char(' ') => Some(egui::Key::Space),
        Key::Char(c) => egui::Key::from_name(&c.to_uppercase().to_string()),
        named => NAMED_KEYS.iter().find(|(_, n)| *n == named).map(|(e, _)| *e),
    }
}

fn key_char(key: egui::Key) -> Option<char> {
    if key == egui::Key::Space {
        return Some(' ');
    }
    let mut cs = key.symbol_or_name().chars();
    match (cs.next(), cs.next()) {
        (Some(c), None) => Some(c.to_ascii_lowercase()),
        _ => None,
    }
}

pub fn key_input(event: &Event) -> Option<KeyInput> {
    match event {
        Event::Text(s) => {
            let mut chars = s.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) if !c.is_control() => Some(KeyInput::new(c, Mods::NONE)),
                _ => None,
            }
        }
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } => {
            let mods = modifiers.to_mods();
            if let Some(named) = named_key(*key) {
                return Some(KeyInput::new(named, mods));
            }
            if !mods.intersects(Mods::CTRL | Mods::ALT | Mods::CMD) {
                return None;
            }
            let c = key_char(*key)?;
            let c = if mods.contains(Mods::SHIFT) {
                c.to_ascii_uppercase()
            } else {
                c
            };
            Some(KeyInput::new(c, mods))
        }
        _ => None,
    }
}

fn event_input(event: &Event, repeat_chars: &[char]) -> Option<(KeyInput, bool)> {
    let input = key_input(event)?;
    let repeat = match event {
        Event::Key { repeat, .. } => *repeat,
        Event::Text(s) => s
            .chars()
            .next()
            .is_some_and(|c| repeat_chars.contains(&c.to_ascii_lowercase())),
        _ => false,
    };
    Some((input, repeat))
}

const MOD_KEYS: [(Mods, char); 4] = [
    (Mods::SHIFT, '\u{E001}'),
    (Mods::CTRL, '\u{E002}'),
    (Mods::ALT, '\u{E003}'),
    (Mods::CMD, '\u{E004}'),
];

pub fn mod_key(m: Mods) -> Option<KeyInput> {
    MOD_KEYS
        .iter()
        .find(|(x, _)| *x == m)
        .map(|(_, c)| KeyInput::new(*c, Mods::NONE))
}

pub fn key_as_mod(k: Key) -> Option<Mods> {
    match k {
        Key::Char(c) => MOD_KEYS.iter().find(|(_, x)| *x == c).map(|(m, _)| *m),
        _ => None,
    }
}

fn input_down(i: &egui::InputState, input: KeyInput, text_safe: bool) -> bool {
    // A lone modifier is down whenever that modifier is held (others may be too).
    if let Some(m) = key_as_mod(input.key()) {
        return i.modifiers.to_mods().contains(m);
    }
    let want = input.mods();
    if text_safe && !want.intersects(Mods::CTRL | Mods::ALT | Mods::CMD) {
        return false;
    }
    let mut have = i.modifiers.to_mods();
    match input.key() {
        // Letters: `Char('A')` means Shift+a, `Char('a')` means no Shift.
        Key::Char(c) if c.is_alphabetic() => {
            if have.contains(Mods::SHIFT) != c.is_uppercase() {
                return false;
            }
            have.remove(Mods::SHIFT);
        }
        // Symbols: shift is already baked into the character.
        Key::Char(_) => have.remove(Mods::SHIFT),
        _ => {}
    }
    have == want && to_egui_key(input.key()).is_some_and(|k| i.key_down(k))
}

/// Name of the context `poll` keeps in sync with egui's keyboard focus.
pub const TEXT_FOCUS: &str = "text_focus";

fn recording_id() -> Id {
    Id::new("keymap::recording")
}

#[derive(Debug)]
pub struct KeymapSystem {
    context: Vec<&'static str>,
    keymaps: HashMap<&'static str, Keymap>,
    /// What happened to each bind this frame. Rebuilt by `poll`.
    triggered: HashMap<&'static str, KeyAction>,
    /// Single-key binds that were held last frame (for release detection).
    was_down: HashSet<&'static str>,
    /// Keys typed so far in an unfinished sequence.
    pending: Vec<KeyInput>,
    /// Binds that match `pending` exactly but could still become longer ones.
    deferred: Vec<&'static str>,
    deadline: Option<f64>,
    /// Seconds allowed between keys of a sequence.
    pub timeout: f64,
    /// While a text field has focus, ignore binds without Ctrl/Alt/Cmd.
    pub text_safe: bool,
}

#[derive(Debug)]
struct Keymap {
    condition: ContextCondition,
    keybinds: Vec<KeySeq>,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub struct KeyAction: u8 {
        const NONE      = 0b0000;
        const PRESSED   = 0b0001;
        const UNPRESSED = 0b0010;
        const HELD      = 0b0100;
        const DOWN      = 0b1000;
    }
}

impl Default for KeymapSystem {
    fn default() -> Self {
        Self {
            context: Vec::new(),
            keymaps: HashMap::new(),
            triggered: HashMap::new(),
            was_down: HashSet::new(),
            pending: Vec::new(),
            deferred: Vec::new(),
            deadline: None,
            timeout: 1.0,
            text_safe: true,
        }
    }
}

fn seqs_clash(a: &KeySeq, b: &KeySeq) -> bool {
    a.match_to(b.as_slice()) != Match::Unmatch || b.match_to(a.as_slice()) != Match::Unmatch
}

impl KeymapSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_keybind(&mut self, name: &'static str, condition: ContextCondition, keybinds: Vec<KeySeq>) {
        let map = Keymap { condition, keybinds };
        if let Some(Keymap { condition, keybinds }) = self.keymaps.insert(name, map) {
            tracing::warn!(target: DB_LOGIC, ?name, ?keybinds, ?condition, "double-registered keybind, replacing sequences")
        }
    }

    pub fn update_keybind(&mut self, name: &'static str, keybinds: Vec<KeySeq>) {
        if let Some(k) = self.keymaps.get_mut(name) {
            k.keybinds = keybinds;
        } else {
            tracing::warn!(target: DB_LOGIC, ?name, ?keybinds, "Cannot assign key sequences to unregistered keybind");
        }
    }

    pub fn update_from_map(&mut self, map: HashMap<String, Vec<KeySeq>>) {
        for (name, val) in map {
            if let Some(v) = self.keymaps.get_mut(name.as_str()) {
                v.keybinds = val;
            } else {
                tracing::warn!(target: DB_LOGIC, ?name, "Unregistered keybind id")
            }
        }
    }

    pub fn get_sequences(&self, name: &'static str) -> Option<&[KeySeq]> {
        Some(&self.keymaps.get(name)?.keybinds)
    }

    pub fn get_sequence_string(&self, name: &'static str, sequence_separator: &'static str) -> Option<String> {
        let maps = self.get_sequences(name)?;
        Some(
            maps.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(sequence_separator)
        )
    }

    pub fn get_sequence_string2(&self, name: &'static str, sequence_separator: &'static str) -> String {
        if let Some(out) = self.get_sequence_string(name, sequence_separator) {
            out
        } else {
            tracing::warn!(target: DB_LOGIC, ?name, "Keybind not registered");
            format!("<invalid: {name}>")
        }
    }

    pub fn get_map(&self) -> HashMap<String, Vec<KeySeq>> {
        let mut out = HashMap::new();
        for (name, Keymap { keybinds, .. }) in self.keymaps.iter() {
            out.insert(name.to_string(), keybinds.clone());
        }
        out
    }

    pub fn get_conflicts(&self, name: &'static str) -> Option<Vec<&'static str>> {
        let base = self.keymaps.get(name)?;
        let mut out = Vec::with_capacity(16);

        for (other, km) in self.keymaps.iter() {
            if name == *other { continue };
            if !base.condition.overlaps(&km.condition) { continue };
            let clash = base.keybinds.iter().any(|a| km.keybinds.iter().any(|b| seqs_clash(a, b)));
            if clash {
                out.push(*other);
            }
        }

        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    pub fn push_ctx(&mut self, ctx: &'static str) {
        if !self.context.contains(&ctx) {
            self.context.push(ctx);
        }
    }

    pub fn remove_ctx(&mut self, ctx: &'static str) {
        self.context.retain(|c| *c != ctx);
    }

    pub fn poll(&mut self, ctx: &egui::Context) {
        tracing::trace!(target: DB_KEYMAP, "Called poll");
        self.triggered.clear();
        let now = ctx.input(|i| i.time);

        let recording = ctx
            .data(|d| d.get_temp::<f64>(recording_id()))
            .is_some_and(|t| now - t < 0.5);
        if recording {
            self.reset_sequence();
            self.was_down.clear();
            return;
        }

        let text_focus = ctx.wants_keyboard_input();
        if text_focus {
            self.push_ctx(TEXT_FOCUS);
        } else {
            self.remove_ctx(TEXT_FOCUS);
        }
        let text_safe = self.text_safe && text_focus;

        self.tick(now);

        let repeat_chars: Vec<char> = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|ev| match ev {
                    Event::Key { key, pressed: true, repeat: true, .. } => key_char(*key),
                    _ => None,
                })
                .collect()
        });
        ctx.input_mut(|i| {
            i.events.retain(|ev| {
                let Some((input, repeat)) = event_input(ev, &repeat_chars) else { return true };
                if text_safe && !input.mods().intersects(Mods::CTRL | Mods::ALT | Mods::CMD) {
                    return true;
                }
                !self.feed(input, repeat, now)
            });
        });

        let down_now: HashMap<&'static str, bool> = ctx.input(|i| {
            self.keymaps
                .iter()
                .filter(|(_, km)| km.condition.check(&self.context))
                .filter_map(|(name, km)| {
                    let mut hit = false;
                    let mut via_mod = false;
                    for seq in &km.keybinds {
                        if let [input] = seq.as_slice() {
                            if input_down(i, *input, text_safe) {
                                hit = true;
                                via_mod |= key_as_mod(input.key()).is_some();
                            }
                        }
                    }
                    hit.then_some((*name, via_mod))
                })
                .collect()
        });
        for (name, via_mod) in &down_now {
            self.trigger(name, KeyAction::DOWN);
            if *via_mod && !self.was_down.contains(name) {
                self.trigger(name, KeyAction::PRESSED);
            }
        }
        let released: Vec<&'static str> = self
            .was_down
            .iter()
            .filter(|n| !down_now.contains_key(*n))
            .copied()
            .collect();
        for name in released {
            self.trigger(name, KeyAction::UNPRESSED);
        }
        self.was_down = down_now.keys().copied().collect();
    }

    pub fn mark_recording(ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        ctx.data_mut(|d| d.insert_temp(recording_id(), now));
    }

    pub fn check_triggered(&self, name: &'static str) -> KeyAction {
        if !self.keymaps.contains_key(&name) {
            tracing::warn!(target: DB_LOGIC, ?name, "Unregistered keybind is untriggerable");
            KeyAction::NONE
        } else {
            self.triggered.get(name).copied().unwrap_or(KeyAction::NONE)
        }
    }

    pub fn pressed(&self, name: &'static str) -> bool {
        self.check_triggered(name).contains(KeyAction::PRESSED)
    }

    pub fn pressed_or_repeat(&self, name: &'static str) -> bool {
        self.check_triggered(name).intersects(KeyAction::PRESSED | KeyAction::HELD)
    }

    pub fn down(&self, name: &'static str) -> bool {
        self.check_triggered(name).contains(KeyAction::DOWN)
    }

    pub fn released(&self, name: &'static str) -> bool {
        self.check_triggered(name).contains(KeyAction::UNPRESSED)
    }

    fn trigger(&mut self, name: &'static str, action: KeyAction) {
        tracing::debug!(target: DB_KEYMAP, ?name, ?action, "key triggered");
        *self.triggered.entry(name).or_insert(KeyAction::NONE) |= action;
    }

    fn reset_sequence(&mut self) {
        self.pending.clear();
        self.deferred.clear();
        self.deadline = None;
    }

    fn flush_deferred(&mut self) {
        for name in std::mem::take(&mut self.deferred) {
            self.trigger(name, KeyAction::PRESSED);
        }
    }

    fn tick(&mut self, now: f64) {
        if self.deadline.is_some_and(|d| now >= d) {
            self.flush_deferred();
            self.reset_sequence();
        }
    }

    fn feed(&mut self, input: KeyInput, repeat: bool, now: f64) -> bool {
        if repeat {
            // Repeats never advance sequences; only single-key binds get HELD.
            let hits: Vec<&'static str> = self
                .keymaps
                .iter()
                .filter(|(_, km)| km.condition.check(&self.context))
                .filter(|(_, km)| km.keybinds.iter().any(|s| s.as_slice() == [input]))
                .map(|(name, _)| *name)
                .collect();
            let consumed = !hits.is_empty();
            for name in hits {
                self.trigger(name, KeyAction::HELD);
            }
            return consumed;
        }

        self.pending.push(input);
        if self.step(now) {
            return true;
        }
        if self.pending.len() > 1 {
            self.flush_deferred();
            self.reset_sequence();
            self.pending.push(input);
            if self.step(now) {
                return true;
            }
        }
        self.reset_sequence();
        false
    }

    fn step(&mut self, now: f64) -> bool {
        let mut exact: Vec<&'static str> = Vec::new();
        let mut longer = false;
        for (name, km) in &self.keymaps {
            if !km.condition.check(&self.context) {
                continue;
            }
            for seq in &km.keybinds {
                match seq.match_to(&self.pending) {
                    Match::Matched => {
                        if !exact.contains(name) {
                            exact.push(*name);
                        }
                    }
                    Match::Prefix => longer = true,
                    Match::Unmatch => {}
                }
            }
        }

        match (exact.is_empty(), longer) {
            (false, false) => {
                for name in exact {
                    self.trigger(name, KeyAction::PRESSED);
                }
                self.reset_sequence();
                true
            }
            (false, true) => {
                self.deferred = exact;
                self.deadline = Some(now + self.timeout);
                true
            }
            (true, true) => {
                self.deferred.clear();
                self.deadline = Some(now + self.timeout);
                true
            }
            (true, false) => false,
        }
    }
}

fn mod_labels(mods: Mods) -> Vec<&'static str> {
    let mut v = Vec::new();
    for (m, mac, other) in [
        (Mods::CTRL, "\u{2303}", "Ctrl"),
        (Mods::ALT, "\u{2325}", "Alt"),
        (Mods::SHIFT, "\u{21E7}", "Shift"),
        (Mods::CMD, "\u{2318}", "Cmd"),
        (Mods::WIN, "\u{229E}", "Win"),
    ] {
        if mods.contains(m) {
            v.push(if cfg!(target_os = "macos") { mac } else { other });
        }
    }
    v
}

fn key_label(key: Key) -> (String, bool) {
    if let Some(m) = key_as_mod(key) {
        return (mod_labels(m).join(""), false);
    }
    match key {
        Key::Up => ("\u{2191}".into(), false),
        Key::Down => ("\u{2193}".into(), false),
        Key::Left => ("\u{2190}".into(), false),
        Key::Right => ("\u{2192}".into(), false),
        Key::Char(' ') => ("Space".into(), false),
        Key::Char(c) if c.is_alphabetic() => (c.to_uppercase().to_string(), c.is_uppercase()),
        other => (other.to_string(), false),
    }
}

pub fn keycap(ui: &mut Ui, text: &str) -> Response {
    let v = ui.visuals();
    egui::Frame::new()
        .fill(v.widgets.inactive.weak_bg_fill)
        .stroke(v.widgets.noninteractive.bg_stroke)
        .corner_radius(4)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| ui.label(egui::RichText::new(text).monospace()))
        .response
}

pub fn key_input_ui(ui: &mut Ui, input: KeyInput) {
    let (label, implies_shift) = key_label(input.key());
    let mut mods = input.mods();
    if implies_shift {
        mods |= Mods::SHIFT;
    }
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        for m in mod_labels(mods) {
            keycap(ui, m);
            if !cfg!(target_os = "macos") {
                ui.weak("+");
            }
        }
        keycap(ui, &label);
    });
}

pub fn key_seq_ui(ui: &mut Ui, seq: &[KeyInput]) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        for (i, input) in seq.iter().enumerate() {
            if i > 0 {
                ui.weak("\u{203A}");
            }
            key_input_ui(ui, *input);
        }
    });
}

fn outline(ui: &Ui, conflicting: bool, recording: bool) -> egui::Stroke {
    let v = ui.visuals();
    if conflicting {
        egui::Stroke::new(1.5f32, v.error_fg_color)
    } else if recording {
        egui::Stroke::new(1.5f32, v.selection.stroke.color)
    } else {
        v.widgets.inactive.bg_stroke
    }
}

fn conflict_text(conflicts: &[String]) -> String {
    format!("Conflicts with: {}", conflicts.join(", "))
}

pub struct SingleKeyEdit<'a> {
    id: Id,
    key: &'a mut Option<KeyInput>,
    conflicts: Vec<String>,
}

impl<'a> Widget for SingleKeyEdit<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

impl<'a> SingleKeyEdit<'a> {
    pub fn new(id_salt: impl Hash, key: &'a mut Option<KeyInput>) -> Self {
        Self { id: Id::new(("single_key_edit", id_salt)), key, conflicts: Vec::new() }
    }

    pub fn conflicts<S: AsRef<str>>(mut self, names: impl IntoIterator<Item = S>) -> Self {
        self.conflicts = names.into_iter().map(|s| s.as_ref().to_owned()).collect();
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let Self { id, key, conflicts } = self;
        let ctx = ui.ctx().clone();
        let (was_active, prev_mods) = ctx
            .data(|d| d.get_temp::<(bool, Mods)>(id))
            .unwrap_or((false, Mods::NONE));
        let have = ui.input(|i| i.modifiers.to_mods());
        let mut active = was_active;
        let mut changed = false;

        if was_active {
            KeymapSystem::mark_recording(&ctx);
            ctx.request_repaint_after(Duration::from_millis(250));

            let mut captured: Option<KeyInput> = None;
            ui.input_mut(|i| {
                i.events.retain(|ev| match key_input(ev) {
                    Some(k) => {
                        captured.get_or_insert(k);
                        false
                    }
                    None => true,
                })
            });
            if let Some(k) = captured {
                active = false;
                if k.key() != Key::Esc {
                    *key = Some(KeyInput::new(k.key(), Mods::NONE));
                    changed = true;
                }
            } else if let Some((m, _)) = MOD_KEYS
                .iter()
                .find(|(m, _)| have.contains(*m) && !prev_mods.contains(*m))
            {
                *key = mod_key(*m);
                changed = true;
                active = false;
            }
        }

        let conflicting = !conflicts.is_empty();
        let inner = egui::Frame::new()
            .stroke(outline(ui, conflicting, active))
            .corner_radius(4)
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| match (active, *key) {
                (true, _) => {
                    ui.weak("press a key\u{2026}");
                }
                (false, Some(k)) => key_input_ui(ui, k),
                (false, None) => {
                    ui.weak("Unbound");
                }
            });
        let mut response = inner
            .response
            .interact(egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        response = if conflicting {
            response.on_hover_text(conflict_text(&conflicts))
        } else {
            response.on_hover_text("Click to rebind, right-click to unbind")
        };

        if response.clicked() && !was_active {
            active = true;
            KeymapSystem::mark_recording(&ctx);
        }
        if response.secondary_clicked() {
            *key = None;
            changed = true;
            active = false;
        }
        if was_active
            && active
            && ui.input(|i| i.pointer.any_pressed())
            && !response.contains_pointer()
        {
            active = false;
        }

        ctx.data_mut(|d| d.insert_temp(id, (active, have)));
        if changed {
            response.mark_changed();
        }
        response
    }
}

#[derive(Clone, Default)]
struct SeqRec {
    target: Option<usize>,
    buf: Vec<KeyInput>,
    last: f64,
}

pub struct KeySeqEdit<'a> {
    id: Id,
    seqs: &'a mut Vec<KeySeq>,
    conflicts: Vec<String>,
    idle: f64,
}

impl<'a> Widget for KeySeqEdit<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

impl<'a> KeySeqEdit<'a> {
    pub fn new(id_salt: impl Hash, seqs: &'a mut Vec<KeySeq>) -> Self {
        Self { id: Id::new(("key_seq_edit", id_salt)), seqs, conflicts: Vec::new(), idle: 0.9 }
    }

    pub fn conflicts<S: AsRef<str>>(mut self, names: impl IntoIterator<Item = S>) -> Self {
        self.conflicts = names.into_iter().map(|s| s.as_ref().to_owned()).collect();
        self
    }

    pub fn idle(mut self, seconds: f64) -> Self {
        self.idle = seconds;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let Self { id, seqs, conflicts, idle } = self;
        let ctx = ui.ctx().clone();
        let mut rec: SeqRec = ctx.data(|d| d.get_temp(id)).unwrap_or_default();
        let was_recording = rec.target.is_some();
        let now = ui.input(|i| i.time);
        let mut changed = false;

        if was_recording {
            KeymapSystem::mark_recording(&ctx);
            ctx.request_repaint_after(Duration::from_millis(250));

            let mut captured = Vec::new();
            ui.input_mut(|i| {
                i.events.retain(|ev| match key_input(ev) {
                    Some(k) => {
                        captured.push(k);
                        false
                    }
                    None => true,
                })
            });
            for k in captured {
                if k == KeyInput::new(Key::Esc, Mods::NONE) {
                    rec = SeqRec::default();
                    break;
                }
                rec.buf.push(k);
                rec.last = now;
            }
            if let Some(idx) = rec.target {
                if !rec.buf.is_empty() && now - rec.last > idle {
                    let seq: KeySeq = rec.buf.iter().copied().collect();
                    if !seqs.contains(&seq) {
                        if idx < seqs.len() {
                            seqs[idx] = seq;
                        } else {
                            seqs.push(seq);
                        }
                        changed = true;
                    }
                    rec = SeqRec::default();
                }
            }
        }

        let conflicting = !conflicts.is_empty();
        let mut remove = None;
        let mut start = None;
        let inner = egui::Frame::new()
            .stroke(outline(ui, conflicting, was_recording))
            .corner_radius(4)
            .inner_margin(egui::Margin::symmetric(6, 4))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (i, seq) in seqs.iter().enumerate() {
                        if rec.target == Some(i) {
                            recording_chip(ui, &rec.buf);
                        } else {
                            let chip = ui
                                .scope(|ui| key_seq_ui(ui, seq.as_slice()))
                                .response
                                .interact(egui::Sense::click())
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .on_hover_text("Click to re-record");
                            if chip.clicked() {
                                start = Some(i);
                            }
                            if ui.small_button("\u{D7}").on_hover_text("Remove").clicked() {
                                remove = Some(i);
                            }
                        }
                        ui.add_space(8.0);
                    }
                    if rec.target == Some(seqs.len()) {
                        recording_chip(ui, &rec.buf);
                    } else {
                        if seqs.is_empty() {
                            ui.weak("Unbound");
                        }
                        if ui.small_button("\u{FF0B}").on_hover_text("Add binding").clicked() {
                            start = Some(seqs.len());
                        }
                    }
                });
            });
        let mut response = inner.response;
        if conflicting {
            response = response.on_hover_text(conflict_text(&conflicts));
        }

        if let Some(i) = remove {
            seqs.remove(i);
            changed = true;
            rec = SeqRec::default();
        }
        if let Some(i) = start {
            rec = SeqRec { target: Some(i), last: now, ..SeqRec::default() };
            KeymapSystem::mark_recording(&ctx);
        }
        if was_recording
            && start.is_none()
            && ui.input(|i| i.pointer.any_pressed())
            && !response.contains_pointer()
        {
            rec = SeqRec::default();
        }

        ctx.data_mut(|d| d.insert_temp(id, rec));
        if changed {
            response.mark_changed();
        }
        response
    }
}

fn recording_chip(ui: &mut Ui, buf: &[KeyInput]) {
    if buf.is_empty() {
        keycap(ui, "press keys\u{2026}");
    } else {
        key_seq_ui(ui, buf);
    }
}

#[cfg(test)]
mod tests {
    use beatcraft_editor_proc::when;
    use super::*;

    fn k(s: &str) -> KeyInput {
        s.parse().unwrap()
    }

    fn seq(s: &str) -> KeySeq {
        s.parse().unwrap()
    }

    fn sys() -> KeymapSystem {
        KeymapSystem::new()
    }

    #[test]
    fn condition_logic() {
        let c = when!((a | b) & c);
        assert!(c.check(&["a", "c"]));
        assert!(c.check(&["b", "c"]));
        assert!(!c.check(&["a", "b"]));
        assert!(!(ctx("a") & "b" & !ctx("c")).check(&["a", "b", "c"]));
        assert!((ctx("a") & "b" & !ctx("c")).check(&["a", "b"]));
        assert!(ALWAYS.check(&[]));
        assert_eq!(!!ctx("a"), ctx("a"));
    }

    #[test]
    fn overlap() {
        assert!(ctx("a").overlaps(&ctx("b")));
        assert!(!ctx("a").overlaps(&!ctx("a")));
        assert!(!(ctx("a") & "b").overlaps(&!ctx("a")));
        assert!(ALWAYS.overlaps(&ctx("a")));
    }

    #[test]
    fn press_gated_by_context() {
        let mut s = sys();
        s.register_keybind("save", ctx("editor") & !ctx("modal"), vec![seq("Ctrl+s")]);
        assert!(!s.feed(k("Ctrl+s"), false, 0.0));
        s.push_ctx("editor");
        assert!(s.feed(k("Ctrl+s"), false, 0.0));
        assert!(s.pressed("save"));
        s.triggered.clear();
        s.push_ctx("modal");
        assert!(!s.feed(k("Ctrl+s"), false, 0.0));
        assert!(!s.pressed("save"));
    }

    #[test]
    fn multibind_and_sequence() {
        let mut s = sys();
        s.register_keybind("save", ALWAYS, vec![seq("Ctrl+s"), seq("Ctrl+k Ctrl+s")]);
        assert!(s.feed(k("Ctrl+s"), false, 0.0));
        assert!(s.pressed("save"));
        s.triggered.clear();
        assert!(s.feed(k("Ctrl+k"), false, 0.0));
        assert!(!s.pressed("save"));
        assert!(s.feed(k("Ctrl+s"), false, 0.1));
        assert!(s.pressed("save"));
    }

    #[test]
    fn broken_sequence_restarts() {
        let mut s = sys();
        s.register_keybind("save", ALWAYS, vec![seq("Ctrl+s"), seq("Ctrl+k Ctrl+s")]);
        assert!(s.feed(k("Ctrl+k"), false, 0.0));
        assert!(s.feed(k("Ctrl+k"), false, 0.1)); // restarts a fresh sequence
        assert!(s.feed(k("Ctrl+s"), false, 0.2));
        assert!(s.pressed("save"));
    }

    #[test]
    fn ambiguity_timeout_and_flush() {
        let mut s = sys();
        s.register_keybind("g", ALWAYS, vec![seq("g")]);
        s.register_keybind("gg", ALWAYS, vec![seq("g g")]);

        // timeout fires the short bind
        assert!(s.feed(k("g"), false, 0.0));
        assert!(!s.pressed("g"));
        s.tick(0.5);
        assert!(!s.pressed("g"));
        s.tick(1.0);
        assert!(s.pressed("g") && !s.pressed("gg"));

        s.triggered.clear();
        assert!(s.feed(k("g"), false, 5.0));
        assert!(s.feed(k("g"), false, 5.1));
        assert!(s.pressed("gg") && !s.pressed("g"));

        s.triggered.clear();
        assert!(s.feed(k("g"), false, 10.0));
        assert!(!s.feed(k("x"), false, 10.1));
        assert!(s.pressed("g") && !s.pressed("gg"));
        assert!(s.pending.is_empty());
    }

    #[test]
    fn repeat_only_held_for_single_keys() {
        let mut s = sys();
        s.register_keybind("down", ALWAYS, vec![seq("j")]);
        s.register_keybind("seq", ALWAYS, vec![seq("a b")]);
        assert!(s.feed(k("j"), false, 0.0));
        assert_eq!(s.check_triggered("down"), KeyAction::PRESSED);
        s.triggered.clear();
        assert!(s.feed(k("j"), true, 0.1));
        assert_eq!(s.check_triggered("down"), KeyAction::HELD);
        assert!(s.pressed_or_repeat("down"));
        // repeats don't advance sequences
        s.triggered.clear();
        assert!(s.feed(k("a"), false, 1.0));
        assert!(!s.feed(k("a"), true, 1.1));
        assert!(s.feed(k("b"), false, 1.2));
        assert!(s.pressed("seq"));
    }

    #[test]
    fn conflicts() {
        let mut s = sys();
        s.register_keybind("a", ctx("x"), vec![seq("Ctrl+s")]);
        s.register_keybind("b", !ctx("x"), vec![seq("Ctrl+s")]);
        s.register_keybind("c", ctx("y"), vec![seq("Ctrl+s")]);
        s.register_keybind("d", ctx("x"), vec![seq("Ctrl+s a")]);
        let mut a = s.get_conflicts("a").unwrap();
        a.sort();
        assert_eq!(a, vec!["c", "d"]);
        assert_eq!(s.get_conflicts("b").unwrap(), vec!["c"]);
    }

    #[test]
    fn egui_key_mapping() {
        assert_eq!(to_egui_key(Key::Char('a')), Some(egui::Key::A));
        assert_eq!(to_egui_key(Key::Char('A')), Some(egui::Key::A));
        assert_eq!(to_egui_key(Key::Char('5')), Some(egui::Key::Num5));
        assert_eq!(to_egui_key(Key::Char(' ')), Some(egui::Key::Space));
        assert_eq!(to_egui_key(Key::Up), Some(egui::Key::ArrowUp));
        assert_eq!(to_egui_key(Key::F5), Some(egui::Key::F5));
        assert_eq!(named_key(egui::Key::Escape), Some(Key::Esc));
        eprintln!(
            "symbols: ':'={:?} '.'={:?} '-'={:?} '/'={:?} ';'={:?}",
            to_egui_key(Key::Char(':')),
            to_egui_key(Key::Char('.')),
            to_egui_key(Key::Char('-')),
            to_egui_key(Key::Char('/')),
            to_egui_key(Key::Char(';')),
        );
    }

    #[test]
    fn mod_key_roundtrip_and_display() {
        for m in [Mods::SHIFT, Mods::CTRL, Mods::ALT, Mods::CMD] {
            let k = mod_key(m).unwrap();
            assert_eq!(k.mods(), Mods::NONE);
            assert_eq!(key_as_mod(k.key()), Some(m));
            let parsed: KeyInput = k.to_string().parse().unwrap();
            assert_eq!(parsed, k);
            let sq: KeySeq = k.to_string().parse().unwrap();
            assert_eq!(sq, KeySeq::from(k));
        }
        assert_eq!(mod_key(Mods::SHIFT | Mods::CTRL), None);
        assert_eq!(key_as_mod(Key::Char('a')), None);
        assert_eq!(key_label(mod_key(Mods::SHIFT).unwrap().key()).0, if cfg!(target_os = "macos") { "\u{21E7}" } else { "Shift" });
    }

    #[test]
    fn simultaneous_keys_and_modifier_only_down() {
        let mut i = egui::InputState::default();
        i.keys_down.insert(egui::Key::W);
        i.keys_down.insert(egui::Key::A);
        // W and A held together: each single-key bind is independently down
        assert!(input_down(&i, k("w"), false));
        assert!(input_down(&i, k("a"), false));
        assert!(!input_down(&i, k("s"), false));
        // holding Shift changes what the letter binds mean: `w` stops matching, `W` matches
        i.modifiers.shift = true;
        assert!(!input_down(&i, k("w"), false));
        assert!(input_down(&i, k("W"), false));
        // ...but a Shift-only bind is down, regardless of other modifiers
        let shift = mod_key(Mods::SHIFT).unwrap();
        assert!(input_down(&i, shift, false));
        i.modifiers.ctrl = true;
        assert!(input_down(&i, shift, false));
        i.modifiers.shift = false;
        assert!(!input_down(&i, shift, false));
        // modifier-only binds work even while a text field has focus
        i.modifiers.shift = true;
        assert!(input_down(&i, shift, true));
        // plain keys don't while text-safe
        i.modifiers = egui::Modifiers::NONE;
        assert!(!input_down(&i, k("w"), true));
    }
}