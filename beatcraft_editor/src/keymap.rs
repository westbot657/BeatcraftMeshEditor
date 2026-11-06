use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::{BitAnd, BitOr, Not};
use std::rc::Rc;
use std::time::Duration;
use bitflags::bitflags;
use egui::{Color32, Event, Id, Modifiers, Response, Ui, Widget};
use keybinds::{Key, KeyInput, KeySeq, Match, Mods};
use parking_lot::RwLock;
use crate::{DB_KEYMAP, DB_LOGIC};
use crate::beatmap::widgets::keybind_hint::HintPart;

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
        self.overlaps_excluding(other, &[])
    }

    pub fn overlaps_excluding(&self, other: &Self, exclusive: &[Vec<&'static str>]) -> bool {
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
            if exclusive
                .iter()
                .any(|g| g.iter().copied().filter(|c| active(c)).count() > 1)
            {
                return false;
            }
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
                    if i < a.len() - 1 {
                        write!(f, " & ")?;
                    }
                }
                write!(f, ")")
            }
            Self::Or(a) => {
                write!(f, "(")?;
                for (i, c) in a.iter().enumerate() {
                    c.fmt(f)?;
                    if i < a.len() - 1 {
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


fn clipboard_mods() -> Mods {
    if cfg!(target_os = "macos") { Mods::CMD } else { Mods::CTRL }
}

pub fn key_input(event: &Event) -> Option<KeyInput> {
    match event {
        Event::Text(s) => {
            let mut chars = s.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) if !c.is_control() => Some(KeyInput::new(c, Mods::NONE)),
                _ => None,
            }
        },
        Event::Copy => Some(KeyInput::new('c', clipboard_mods())),
        Event::Cut => Some(KeyInput::new('x', clipboard_mods())),
        Event::Paste(_) => Some(KeyInput::new('v', clipboard_mods())),
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
    if let Some(m) = key_as_mod(input.key()) {
        return i.modifiers.to_mods().contains(m);
    }
    let mut want = input.mods();
    if text_safe && !want.intersects(Mods::CTRL | Mods::ALT | Mods::CMD) {
        return false;
    }
    if matches!(input.key(), Key::Char(c) if c.is_uppercase()) {
        want |= Mods::SHIFT;
    }
    let mut have = i.modifiers.to_mods();
    for m in [Mods::SHIFT, Mods::ALT] {
        if !want.contains(m) {
            have.remove(m);
        }
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
    context: Rc<RwLock<Vec<&'static str>>>,
    keymaps: HashMap<&'static str, Keymap>,
    /// Groups of contexts that can never be active together (used by conflict checks).
    exclusive: Vec<Vec<&'static str>>,
    /// list of hidden keys that can't be written to the config file.
    hidden: Vec<&'static str>,
    /// What happened to each bind this frame. Rebuilt by `poll`.
    triggered: HashMap<&'static str, KeyAction>,
    /// Single-key binds that were held last frame (for release detection).
    was_down: HashSet<&'static str>,
    /// Keys typed so far in an unfinished sequence.
    pending: Vec<KeyInput>,
    /// Binds that match `pending` exactly but could still become longer ones.
    deferred: Vec<&'static str>,
    deadline: Option<f64>,

    recording_frame: bool,
    text_focus_frame: bool,

    /// Seconds allowed between keys of a sequence.
    pub timeout: f64,
    /// While a text field has focus, ignore binds without Ctrl/Alt/Cmd.
    pub text_safe: bool,
}

pub struct ContextGuard {
    system_ctx: Rc<RwLock<Vec<&'static str>>>,
    context: Vec<&'static str>,
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        let mut guard = self.system_ctx.write();
        guard.retain(|c| !self.context.contains(c));
        tracing::trace!(target: DB_KEYMAP, dropped = ?self.context, current = ?&*guard, "dropped contexts");
    }
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
            context: Default::default(),
            keymaps: HashMap::new(),
            exclusive: Vec::new(),
            hidden: Vec::new(),
            triggered: HashMap::new(),
            was_down: HashSet::new(),
            pending: Vec::new(),
            deferred: Vec::new(),
            deadline: None,
            recording_frame: false,
            text_focus_frame: false,
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

    pub fn hide_key(&mut self, key: &'static str) {
        self.hidden.push(key);
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
                .join(sequence_separator),
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
            if self.hidden.contains(name) { continue };
            out.insert(name.to_string(), keybinds.clone());
        }
        out
    }

    /// Declares contexts that can never be active at the same time.
    /// Conflict checks ignore any combination containing two of them.
    pub fn exclusive(&mut self, group: &[&'static str]) {
        self.exclusive.push(group.to_vec());
    }

    pub fn get_conflicts(&self, name: &'static str) -> Option<Vec<&'static str>> {
        let base = self.keymaps.get(name)?;
        let mut out = Vec::with_capacity(16);

        for (other, km) in self.keymaps.iter() {
            if name == *other {
                continue;
            }
            if !base.condition.overlaps_excluding(&km.condition, &self.exclusive) {
                continue;
            }
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

    pub fn get_seq_conflicts(&self, name: &'static str, seq: &KeySeq) -> Vec<&'static str> {
        let Some(base) = self.keymaps.get(name) else { return Vec::new() };
        self.keymaps
            .iter()
            .filter(|(other, _)| **other != name)
            .filter(|(_, km)| km.keybinds.iter().any(|b| seqs_clash(seq, b)))
            .filter(|(_, km)| base.condition.overlaps_excluding(&km.condition, &self.exclusive))
            .map(|(other, _)| *other)
            .collect()
    }

    /// WARNING: do not bind to `_`, bind to something like `_scope`, `_guard`, etc.
    /// `_` drops instantly, while `_x` drops at the end of scope.
    #[must_use]
    pub fn scope_ctx(&mut self, context: Vec<&'static str>) -> ContextGuard {
        for ctx in context.iter() {
            self.push_ctx(ctx);
        }
        ContextGuard {
            system_ctx: Rc::clone(&self.context),
            context,
        }
    }

    pub fn push_ctx(&mut self, ctx: &'static str) {
        let mut context = self.context.write();
        if !context.contains(&ctx) {
            for g in self.exclusive.iter().filter(|g| g.contains(&ctx)) {
                if let Some(clash) = g.iter().find(|o| **o != ctx && context.contains(*o)) {
                    tracing::warn!(target: DB_KEYMAP, ?ctx, ?clash, "pushed a context that is declared exclusive with an active one");
                }
            }
            context.push(ctx);
            tracing::trace!(target: DB_KEYMAP, ?ctx, ?context, "pushed context");

        }
    }

    pub fn remove_ctx(&mut self, ctx: &'static str) {
        let mut context = self.context.write();
        let l0 = context.len();
        context.retain(|c| *c != ctx);
        if l0 != context.len() {
            tracing::trace!(target: DB_KEYMAP, ?ctx, "popped context");
        }
    }

    pub fn poll_events(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        self.triggered.clear();
        let now = raw_input.time.unwrap_or_else(|| ctx.input(|i| i.time));

        let recording = ctx
            .data(|d| d.get_temp::<f64>(recording_id()))
            .is_some_and(|t| now - t < 0.5);
        if recording {
            self.reset_sequence();
            self.was_down.clear();
            self.recording_frame = true;
            return;
        }
        self.recording_frame = false;

        let text_focus = ctx.wants_keyboard_input();
        if text_focus {
            self.push_ctx(TEXT_FOCUS);
        } else {
            self.remove_ctx(TEXT_FOCUS);
        }
        let text_safe = self.text_safe && text_focus;
        self.text_focus_frame = text_safe;

        self.tick(now);

        let repeat_chars: Vec<char> = raw_input
            .events
            .iter()
            .filter_map(|ev| match ev {
                Event::Key { key, pressed: true, repeat: true, .. } => key_char(*key),
                _ => None,
            })
            .collect();

        raw_input.events.retain(|ev| {
            if text_focus && matches!(ev, Event::Copy | Event::Cut | Event::Paste(_)) {
                return true;
            }
            let is_tab = matches!(ev, Event::Key { key: egui::Key::Tab, pressed: true, .. });
            let consumed = match event_input(ev, &repeat_chars) {
                Some((input, repeat)) => {
                    if text_safe && !input.mods().intersects(Mods::CTRL | Mods::ALT | Mods::CMD) {
                        false
                    } else {
                        self.feed(input, repeat, now)
                    }
                }
                None => false,
            };
            !(consumed || (is_tab && !text_focus))
        });
    }

    pub fn poll_held(&mut self, ctx: &egui::Context) {
        if self.recording_frame {
            return;
        }
        let text_safe = self.text_focus_frame;

        let down_now: HashMap<&'static str, bool> = ctx.input(|i| {
            self.keymaps
                .iter()
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

    pub fn is_active(&self, name: &'static str) -> bool {
        self.keymaps
            .get(name)
            .is_some_and(|km| km.condition.check(&self.context.read()))
    }

    pub fn check_triggered(&self, name: &'static str) -> KeyAction {
        let Some(km) = self.keymaps.get(name) else {
            tracing::warn!(target: DB_LOGIC, ?name, "Unregistered keybind is untriggerable");
            return KeyAction::NONE;
        };
        let Some(&action) = self.triggered.get(name) else {
            return KeyAction::NONE;
        };
        let context = self.context.read();
        let active = km.condition.check(&context);
        tracing::debug!(
            target: DB_KEYMAP,
            ?name,
            condition = %km.condition,
            context = ?*context,
            ?action,
            active,
            "triggered keybind checked"
        );
        if active { action } else { KeyAction::NONE }
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
        tracing::trace!(target: DB_KEYMAP, ?name, ?action, "key triggered");
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

    fn pending_string(&self) -> String {
        self.pending.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ")
    }

    fn feed(&mut self, input: KeyInput, repeat: bool, now: f64) -> bool {
        if repeat {
            tracing::trace!(target: DB_KEYMAP, key = %input, "key repeat");
            let hits: Vec<&'static str> = self
                .keymaps
                .iter()
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
        tracing::debug!(
            target: DB_KEYMAP,
            key = %input,
            sequence = %self.pending_string(),
            context = ?*self.context.read(),
            "new key pressed"
        );
        if self.step(now) {
            return true;
        }
        if self.pending.len() > 1 {
            self.flush_deferred();
            self.reset_sequence();
            self.pending.push(input);
            tracing::debug!(target: DB_KEYMAP, sequence = %self.pending_string(), "sequence broken, retrying key as a fresh sequence");
            if self.step(now) {
                return true;
            }
        }
        tracing::debug!(target: DB_KEYMAP, key = %input, "no bind matched, key passes through");
        self.reset_sequence();
        false
    }

    fn step(&mut self, now: f64) -> bool {
        let mut exact: Vec<&'static str> = Vec::new();
        let mut longer = false;
        for (name, km) in &self.keymaps {
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

    pub fn seq_edit(&mut self, ui: &mut Ui, name: &'static str) -> Response {
        self.edit_row(ui, name, |ui, seqs, conflicts| {
            ui.add(KeySeqEdit::new(name, seqs).conflicts_per_seq(conflicts))
        })
    }

    pub fn single_edit(&mut self, ui: &mut Ui, name: &'static str) -> Response {
        self.edit_row(ui, name, |ui, seqs, conflicts| {
            ui.add(SingleKeyListEdit::new(name, seqs).conflicts_per_seq(conflicts))
        })
    }

    fn edit_row(
        &mut self,
        ui: &mut Ui,
        name: &'static str,
        edit: impl FnOnce(&mut Ui, &mut Vec<KeySeq>, Vec<Vec<String>>) -> Response,
    ) -> Response {
        let Some(current) = self.get_sequences(name) else {
            tracing::warn!(target: DB_LOGIC, ?name, "Keybind not registered");
            let color = ui.visuals().error_fg_color;
            return ui.colored_label(color, format!("<invalid: {name}>"));
        };
        let mut seqs = current.to_vec();
        let conflicts: Vec<Vec<String>> = seqs
            .iter()
            .map(|s| self.get_seq_conflicts(name, s).into_iter().map(String::from).collect())
            .collect();

        let response = edit(ui, &mut seqs, conflicts);
        if response.changed() {
            self.update_keybind(name, seqs);
        }
        response
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

pub fn paint_key_seq_hint(
    ctx: &egui::Context,
    painter: &egui::Painter,
    seq: &[KeyInput],
    pos: egui::Pos2,
    pivot: egui::Align2,
    bg: Option<Color32>,
) {
    paint_hint_parts(ctx, painter, &[HintPart::Keys(seq)], pos, pivot, bg);
}

/// Paints a framed row of key sequences and plain text, anchored at `pos` by `pivot`.
/// Registers no area and no widgets.
pub fn paint_hint_parts(
    ctx: &egui::Context,
    painter: &egui::Painter,
    parts: &[HintPart],
    pos: egui::Pos2,
    pivot: egui::Align2,
    bg: Option<Color32>,
) {
    use std::sync::Arc;

    let style = ctx.style();
    let v = &style.visuals;
    let font = egui::TextStyle::Monospace.resolve(&style);
    let (cap_pad, cap_gap, seq_gap, part_gap, margin) =
        (egui::vec2(6.0, 2.0), 3.0, 6.0, 8.0, 2.0);

    // (galley, is_keycap, gap before this item)
    let mut items: Vec<(Arc<egui::Galley>, bool, f32)> = Vec::new();
    let text = |s: &str, color: egui::Color32| {
        painter.layout_no_wrap(s.to_owned(), font.clone(), color)
    };

    for (pi, part) in parts.iter().enumerate() {
        let start = items.len();
        match part {
            HintPart::Text(s) => {
                items.push((text(s, v.text_color()), false, 0.0));
            }
            HintPart::Keys(seq) => {
                for (i, input) in seq.iter().enumerate() {
                    if i > 0 {
                        items.push((text("\u{203A}", v.weak_text_color()), false, seq_gap));
                    }
                    let (label, implies_shift) = key_label(input.key());
                    let mut mods = input.mods();
                    if implies_shift {
                        mods |= Mods::SHIFT;
                    }
                    let mut first = i > 0;
                    for m in mod_labels(mods) {
                        let gap = if first { cap_gap } else { 0.0 };
                        items.push((text(m, v.text_color()), true, gap));
                        first = true;
                        if !cfg!(target_os = "macos") {
                            items.push((text("+", v.weak_text_color()), false, cap_gap));
                        }
                    }
                    let gap = if first { cap_gap } else { 0.0 };
                    items.push((text(&label, v.text_color()), true, gap));
                }
            }
            HintPart::Labels(labels) => {
                for (i, label) in labels.iter().enumerate() {
                    if i > 0 {
                        items.push((text("+", v.weak_text_color()), false, cap_gap));
                    }
                    let gap = if i > 0 { cap_gap } else { 0.0 };
                    items.push((text(label, v.text_color()), true, gap));
                }
            }
        }
        if pi > 0 {
            if let Some(first) = items.get_mut(start) {
                first.2 += part_gap;
            }
        }
    }

    if items.is_empty() {
        return;
    }

    let content_h = items
        .iter()
        .map(|(g, cap, _)| g.size().y + if *cap { cap_pad.y * 2.0 } else { 0.0 })
        .fold(0.0, f32::max);
    let content_w: f32 = items
        .iter()
        .map(|(g, cap, gap)| g.size().x + if *cap { cap_pad.x * 2.0 } else { 0.0 } + gap)
        .sum();

    let size = egui::vec2(content_w, content_h) + egui::Vec2::splat(margin * 2.0);
    let rect = pivot.anchor_size(pos, size);

    if let Some(bg) = bg {
        painter.rect(rect, 4, bg, egui::Stroke::new(1f32, bg), egui::StrokeKind::Inside);
    }

    let mut x = rect.min.x + margin;
    let cy = rect.center().y;
    for (galley, cap, gap) in items {
        x += gap;
        let gs = galley.size();
        if cap {
            let cap_rect = egui::Rect::from_min_size(
                egui::pos2(x, cy - (gs.y + cap_pad.y * 2.0) / 2.0),
                gs + cap_pad * 2.0,
            );
            painter.rect(
                cap_rect,
                4,
                v.widgets.inactive.weak_bg_fill,
                v.widgets.noninteractive.bg_stroke,
                egui::StrokeKind::Inside,
            );
            painter.galley(cap_rect.min + cap_pad, galley, v.text_color());
            x = cap_rect.max.x;
        } else {
            painter.galley(egui::pos2(x, cy - gs.y / 2.0), galley, v.text_color());
            x += gs.x;
        }
    }
}

fn commit_seq(seqs: &mut Vec<KeySeq>, idx: usize, buf: &[KeyInput]) -> bool {
    let seq: KeySeq = buf.iter().copied().collect();
    if seqs.contains(&seq) {
        return false;
    }
    if idx < seqs.len() {
        seqs[idx] = seq;
    } else {
        seqs.push(seq);
    }
    true
}

fn recording_chip(ui: &mut Ui, buf: &[KeyInput]) {
    if buf.is_empty() {
        keycap(ui, "press keys\u{2026}");
    } else {
        key_seq_ui(ui, buf);
    }
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

fn remove_button(ui: &mut Ui) -> bool {
    let red = ui.visuals().error_fg_color;
    ui.add(
        egui::Button::new(egui::RichText::new("\u{D7}").size(11.0).color(red))
            .small()
            .frame(false),
    )
        .on_hover_text("Remove")
        .clicked()
}

fn darken(c: egui::Color32, amount: f32) -> egui::Color32 {
    let m = |x: u8| (x as f32 * (1.0 - amount)) as u8;
    egui::Color32::from_rgb(m(c.r()), m(c.g()), m(c.b()))
}

fn add_button(ui: &mut Ui, empty: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 16.0),
        egui::Sense::click(),
    );
    let v = ui.visuals();
    let painter = ui.painter();
    if resp.hovered() {
        let fill = darken(v.widgets.inactive.weak_bg_fill, 0.25);
        painter.rect_filled(rect, 4, fill);
    }
    let color = if resp.hovered() { v.strong_text_color() } else { v.text_color() };
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "\u{FF0B}",
        egui::FontId::proportional(12.0),
        color,
    );
    if empty {
        painter.text(
            rect.left_center() + egui::vec2(6.0, 0.0),
            egui::Align2::LEFT_CENTER,
            "Unbound",
            egui::FontId::proportional(11.0),
            v.weak_text_color(),
        );
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Add binding")
        .clicked()
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
    fn make_id(id_salt: impl Hash) -> Id {
        Id::new(("single_key_edit", id_salt))
    }

    pub fn new(id_salt: impl Hash, key: &'a mut Option<KeyInput>) -> Self {
        Self { id: Self::make_id(id_salt), key, conflicts: Vec::new() }
    }

    pub fn start_recording(ctx: &egui::Context, id_salt: impl Hash) {
        let mods = ctx.input(|i| i.modifiers.to_mods());
        ctx.data_mut(|d| d.insert_temp(Self::make_id(id_salt), (true, mods)));
        KeymapSystem::mark_recording(ctx);
    }

    pub fn is_recording(ctx: &egui::Context, id_salt: impl Hash) -> bool {
        ctx.data(|d| d.get_temp::<(bool, Mods)>(Self::make_id(id_salt)))
            .is_some_and(|(active, _)| active)
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

pub struct SingleKeyListEdit<'a> {
    id: Id,
    seqs: &'a mut Vec<KeySeq>,
    conflicts: Vec<Vec<String>>,
}

impl<'a> Widget for SingleKeyListEdit<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

impl<'a> SingleKeyListEdit<'a> {
    pub fn new(id_salt: impl Hash, seqs: &'a mut Vec<KeySeq>) -> Self {
        Self { id: Id::new(("single_key_list", id_salt)), seqs, conflicts: Vec::new() }
    }

    pub fn conflicts_per_seq(mut self, per_seq: Vec<Vec<String>>) -> Self {
        self.conflicts = per_seq;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let Self { id, seqs, conflicts } = self;
        let ctx = ui.ctx().clone();
        let pending_id = id.with("pending");
        let new_slot = id.with("new_slot");
        let mut pending: bool = ctx.data(|d| d.get_temp(pending_id)).unwrap_or(false);
        let mut changed = false;
        let mut remove: Option<usize> = None;

        let inner = ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            for i in 0..seqs.len() {
                let single = match seqs[i].as_slice() {
                    [k] => *k,
                    _ => continue,
                };
                ui.horizontal(|ui| {
                    let mut key = Some(single);
                    let slot_conflicts = conflicts.get(i).cloned().unwrap_or_default();
                    let resp = ui.add(SingleKeyEdit::new(id.with(i), &mut key).conflicts(&slot_conflicts));
                    if resp.changed() {
                        match key {
                            Some(k) => {
                                let new = KeySeq::from(k);
                                if !seqs.contains(&new) {
                                    seqs[i] = new;
                                    changed = true;
                                }
                            }
                            None => remove = Some(i),
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if remove_button(ui) {
                            remove = Some(i);
                        }
                    });
                });
            }

            if pending {
                let mut key = None;
                ui.add(SingleKeyEdit::new(new_slot, &mut key));
                if let Some(k) = key {
                    let new = KeySeq::from(k);
                    if !seqs.contains(&new) {
                        seqs.push(new);
                        changed = true;
                    }
                    pending = false;
                } else if !SingleKeyEdit::is_recording(&ctx, new_slot) {
                    pending = false;
                }
            } else if add_button(ui, seqs.is_empty()) {
                SingleKeyEdit::start_recording(&ctx, new_slot);
                pending = true;
            }
        });

        if let Some(i) = remove {
            seqs.remove(i);
            changed = true;
        }
        ctx.data_mut(|d| d.insert_temp(pending_id, pending));

        let mut response = inner.response;
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
    conflicts: Vec<Vec<String>>,
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

    pub fn conflicts_per_seq(mut self, per_seq: Vec<Vec<String>>) -> Self {
        self.conflicts = per_seq;
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
                    changed |= commit_seq(seqs, idx, &rec.buf);
                    rec = SeqRec::default();
                }
            }
        }

        let mut remove = None;
        let mut start = None;
        let inner = ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            for (i, seq) in seqs.iter().enumerate() {
                let recording_this = rec.target == Some(i);
                let row_conflicts = conflicts.get(i).map(Vec::as_slice).unwrap_or(&[]);
                ui.horizontal(|ui| {
                    let frame = egui::Frame::new()
                        .stroke(outline(ui, !row_conflicts.is_empty(), recording_this))
                        .corner_radius(4)
                        .inner_margin(egui::Margin::symmetric(6, 4))
                        .show(ui, |ui| {
                            if recording_this {
                                recording_chip(ui, &rec.buf);
                            } else {
                                key_seq_ui(ui, seq.as_slice());
                            }
                        })
                        .response;
                    if !recording_this {
                        let chip = frame
                            .interact(egui::Sense::click())
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                        let chip = if row_conflicts.is_empty() {
                            chip.on_hover_text("Click to re-record")
                        } else {
                            chip.on_hover_text(conflict_text(row_conflicts))
                        };
                        if chip.clicked() {
                            start = Some(i);
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if remove_button(ui) {
                                remove = Some(i);
                            }
                        });
                    }
                });
            }
            if rec.target == Some(seqs.len()) {
                egui::Frame::new()
                    .stroke(outline(ui, false, true))
                    .corner_radius(4)
                    .inner_margin(egui::Margin::symmetric(6, 4))
                    .show(ui, |ui| recording_chip(ui, &rec.buf));
            } else if add_button(ui, seqs.is_empty()) {
                start = Some(seqs.len());
            }
        });
        let mut response = inner.response;

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
            if let Some(idx) = rec.target {
                if !rec.buf.is_empty() {
                    changed |= commit_seq(seqs, idx, &rec.buf);
                }
            }
            rec = SeqRec::default();
        }

        ctx.data_mut(|d| d.insert_temp(id, rec));
        if changed {
            response.mark_changed();
        }
        response
    }
}
