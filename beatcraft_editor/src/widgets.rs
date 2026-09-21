use egui::emath::TSTransform;
use egui::{Color32, Image, ImageSource, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget};
use num_traits::Num;

use crate::math_interp::MapIndexable;

pub struct MathDragValue<'a, T: MapIndexable> {
    value: &'a mut f32,
    vars: &'a mut T,
    speed: f64,
    max_decimals: usize,
    suffix: Option<&'a str>,
    degrees: bool,
}

impl<'a, T: MapIndexable> MathDragValue<'a, T> {
    pub fn new(value: &'a mut f32, vars: &'a mut T) -> Self {
        Self {
            value,
            vars,
            speed: 0.1,
            max_decimals: 3,
            suffix: None,
            degrees: false,
        }
    }
    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }
    pub fn max_decimals(mut self, d: usize) -> Self {
        self.max_decimals = d;
        self
    }
    pub fn suffix(mut self, s: &'a str) -> Self {
        self.suffix = Some(s);
        self
    }
    pub fn degrees(mut self) -> Self {
        self.degrees = true;
        self
    }
}

impl<'a, T: MapIndexable> egui::Widget for MathDragValue<'a, T> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let id = ui.next_auto_id();

        let (mut editing, mut text) = ui.memory_mut(|m| {
            let editing = m.data.get_temp::<bool>(id).unwrap_or(false);
            let text = m
                .data
                .get_temp::<String>(id)
                .unwrap_or_else(|| format!("{:.prec$}", self.value, prec = self.max_decimals));
            (editing, text)
        });

        let response = if editing {
            let edit_response = ui.add(
                egui::TextEdit::singleline(&mut text)
                    .desired_width(ui.available_width())
                    .clip_text(true)
                    .font(egui::TextStyle::Monospace),
            );

            let commit =
                edit_response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter));
            let cancel = ui.input(|i| i.key_pressed(egui::Key::Escape));

            if commit {
                if let Some(result) = crate::math_interp::eval_inner(&text, self.vars, self.degrees)
                {
                    *self.value = result;
                }
                text = format!("{:.prec$}", self.value, prec = self.max_decimals);
                editing = false;
                let mut r = edit_response;
                r.mark_changed();
                r
            } else if cancel {
                text = format!("{:.prec$}", self.value, prec = self.max_decimals);
                editing = false;
                edit_response
            } else {
                edit_response
            }
        } else {
            let available = ui.available_width();
            let display = match self.suffix {
                Some(s) => format!("{:.prec$}{s}", self.value, prec = self.max_decimals),
                None => format!("{:.prec$}", self.value, prec = self.max_decimals),
            };

            let prev = *self.value;

            let drag = egui::DragValue::new(self.value)
                .speed(self.speed)
                .max_decimals(self.max_decimals)
                .custom_formatter(move |_, _| display.clone())
                .custom_parser(|s| s.parse::<f64>().ok());

            let mut drag_response = ui.add_sized([available, 20.], drag);

            if drag_response.clicked() {
                editing = true;
                text = format!("{:.prec$}", self.value, prec = self.max_decimals);
            }

            if (*self.value - prev).abs() > f32::EPSILON {
                drag_response.mark_changed();
            }

            drag_response
        };

        let wants_focus = editing
            && !ui.memory(|m| {
                m.data
                    .get_temp::<bool>(id.with("had_focus"))
                    .unwrap_or(false)
            });

        ui.memory_mut(|m| {
            m.data.insert_temp(id, editing);
            m.data.insert_temp(id, text);
            m.data.insert_temp(id.with("had_focus"), editing);
        });

        if wants_focus {
            response.request_focus();
        }

        response
    }
}

pub struct MultiMathValue<'a, 'b, 'c, 'd, T: MapIndexable> {
    hint_text: &'static str,
    values: &'b mut Option<Vec<f32>>,
    vars: &'c mut [&'d mut T],
    max_decimals: usize,
    suffix: Option<&'a str>,
    degrees: bool,
}

impl<'a, 'b, 'c, 'd, T: MapIndexable> MultiMathValue<'a, 'b, 'c, 'd, T> {
    pub fn new(
        hint_text: &'static str,
        values: &'b mut Option<Vec<f32>>,
        vars: &'c mut [&'d mut T],
    ) -> Self {
        Self {
            hint_text,
            values,
            vars,
            max_decimals: 3,
            suffix: None,
            degrees: false,
        }
    }

    pub fn max_decimals(mut self, d: usize) -> Self {
        self.max_decimals = d;
        self
    }

    pub fn suffix(mut self, s: &'a str) -> Self {
        self.suffix = Some(s);
        self
    }

    pub fn degrees(mut self) -> Self {
        self.degrees = true;
        self
    }
}

impl<'a, 'b, 'c, 'd, T: MapIndexable> egui::Widget for MultiMathValue<'a, 'b, 'c, 'd, T> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let id = ui.next_auto_id();

        let mut text = ui.memory_mut(|m| m.data.get_temp::<String>(id).unwrap_or_else(String::new));

        let edit_response = ui.add(
            egui::TextEdit::singleline(&mut text)
                .desired_width(ui.available_width())
                .clip_text(true)
                .font(egui::TextStyle::Monospace)
                .hint_text(self.hint_text),
        );

        let commit = ui.input(|i| i.key_pressed(egui::Key::Enter));
        let cancel = ui.input(|i| i.key_pressed(egui::Key::Escape));

        if commit && !text.trim().is_empty() {
            let evaluated: Option<Vec<f32>> = self
                .vars
                .iter_mut()
                .map(|vars| crate::math_interp::eval_inner(&text, *vars, self.degrees))
                .collect();

            if let Some(results) = evaluated {
                text.clear();
                *self.values = Some(results);
            }
        } else if cancel {
            text = String::new();
            *self.values = None;
        }

        ui.memory_mut(|m| {
            m.data.insert_temp(id, text);
        });

        edit_response
    }
}

pub struct MathDragValueOpt<'a, V: Num, T: MapIndexable> {
    value: &'a mut Option<V>,
    vars: &'a mut T,
    speed: V,
    max_decimals: usize,
    suffix: Option<&'a str>,
    degrees: bool,
}

impl<'a, T: MapIndexable> MathDragValueOpt<'a, f32, T> {
    pub fn new(value: &'a mut Option<f32>, vars: &'a mut T) -> Self {
        Self {
            value,
            vars,
            speed: 0.1,
            max_decimals: 3,
            suffix: None,
            degrees: false,
        }
    }
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
    pub fn max_decimals(mut self, d: usize) -> Self {
        self.max_decimals = d;
        self
    }
    pub fn suffix(mut self, s: &'a str) -> Self {
        self.suffix = Some(s);
        self
    }
    pub fn degrees(mut self) -> Self {
        self.degrees = true;
        self
    }
}

impl<'a, T: MapIndexable> egui::Widget for MathDragValueOpt<'a, f32, T> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let id = ui.next_auto_id();

        let (mut editing, mut text) = ui.memory_mut(|m| {
            let editing = m.data.get_temp::<bool>(id).unwrap_or(false);
            let text = m
                .data
                .get_temp::<String>(id)
                .unwrap_or_else(|| match self.value {
                    Some(v) => format!("{:.prec$}", v, prec = self.max_decimals),
                    None => String::new(),
                });
            (editing, text)
        });

        let response = if editing || self.value.is_none() {
            let edit_response = ui.add(
                egui::TextEdit::singleline(&mut text)
                    .desired_width(ui.available_width())
                    .clip_text(true)
                    .font(egui::TextStyle::Monospace),
            );

            let commit =
                edit_response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter));
            let cancel = ui.input(|i| i.key_pressed(egui::Key::Escape));

            if commit {
                if text.trim().is_empty() {
                    *self.value = None;
                } else if let Some(result) =
                    crate::math_interp::eval_inner(&text, self.vars, self.degrees)
                {
                    *self.value = Some(result);
                    text = format!("{:.prec$}", result, prec = self.max_decimals);
                }
                editing = false;
                let mut r = edit_response;
                r.mark_changed();
                r
            } else if cancel {
                text = match self.value {
                    Some(v) => format!("{:.prec$}", v, prec = self.max_decimals),
                    None => String::new(),
                };
                editing = false;
                edit_response
            } else {
                edit_response
            }
        } else {
            let v = self.value.as_mut().unwrap();
            let prev = *v;

            let display = match self.suffix {
                Some(s) => format!("{:.prec$}{s}", v, prec = self.max_decimals),
                None => format!("{:.prec$}", v, prec = self.max_decimals),
            };

            let drag = egui::DragValue::new(v)
                .speed(self.speed)
                .max_decimals(self.max_decimals)
                .custom_formatter(move |_, _| display.clone())
                .custom_parser(|s| s.parse::<f64>().ok());

            let mut drag_response = ui.add(drag);

            if drag_response.clicked() {
                editing = true;
                text = format!("{:.prec$}", v, prec = self.max_decimals);
            }

            if (*v - prev).abs() > f32::EPSILON {
                drag_response.mark_changed();
            }

            drag_response
        };

        let wants_focus = editing
            && !ui.memory(|m| {
                m.data
                    .get_temp::<bool>(id.with("had_focus"))
                    .unwrap_or(false)
            });

        ui.memory_mut(|m| {
            m.data.insert_temp(id, editing);
            m.data.insert_temp(id, text);
            m.data.insert_temp(id.with("had_focus"), editing);
        });

        if wants_focus {
            response.request_focus();
        }

        response
    }
}

impl<'a, T: MapIndexable> MathDragValueOpt<'a, u32, T> {
    pub fn new(value: &'a mut Option<u32>, vars: &'a mut T) -> Self {
        Self {
            value,
            vars,
            speed: 1,
            max_decimals: 0,
            suffix: None,
            degrees: false,
        }
    }
    pub fn speed(mut self, speed: u32) -> Self {
        self.speed = speed;
        self
    }
    pub fn max_decimals(mut self, d: usize) -> Self {
        self.max_decimals = d;
        self
    }
    pub fn suffix(mut self, s: &'a str) -> Self {
        self.suffix = Some(s);
        self
    }
    pub fn degrees(mut self) -> Self {
        self.degrees = true;
        self
    }
}

impl<'a, T: MapIndexable> egui::Widget for MathDragValueOpt<'a, u32, T> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let id = ui.next_auto_id();

        let (mut editing, mut text) = ui.memory_mut(|m| {
            let editing = m.data.get_temp::<bool>(id).unwrap_or(false);
            let text = m
                .data
                .get_temp::<String>(id)
                .unwrap_or_else(|| match self.value {
                    Some(v) => format!("{v}"),
                    None => String::new(),
                });
            (editing, text)
        });

        let response = if editing || self.value.is_none() {
            let edit_response = ui.add(
                egui::TextEdit::singleline(&mut text)
                    .desired_width(ui.available_width())
                    .clip_text(true)
                    .font(egui::TextStyle::Monospace),
            );

            let commit =
                edit_response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter));
            let cancel = ui.input(|i| i.key_pressed(egui::Key::Escape));

            if commit {
                if text.trim().is_empty() {
                    *self.value = None;
                } else if let Some(result) =
                    crate::math_interp::eval_inner(&text, self.vars, self.degrees)
                {
                    let rounded = result.round().max(0.0) as u32;
                    *self.value = Some(rounded);
                    text = format!("{rounded}");
                }
                editing = false;
                let mut r = edit_response;
                r.mark_changed();
                r
            } else if cancel {
                text = match self.value {
                    Some(v) => format!("{v}"),
                    None => String::new(),
                };
                editing = false;
                edit_response
            } else {
                edit_response
            }
        } else {
            let v = self.value.as_mut().unwrap();
            let prev = *v;

            let display = match self.suffix {
                Some(s) => format!("{v}{s}"),
                None => format!("{v}"),
            };

            let drag = egui::DragValue::new(v)
                .speed(self.speed)
                .custom_formatter(move |_, _| display.clone())
                .custom_parser(|s| s.parse::<f64>().ok());

            let mut drag_response = ui.add(drag);

            if *v != prev {
                drag_response.mark_changed();
            }

            drag_response
        };

        let wants_focus = editing
            && !ui.memory(|m| {
                m.data
                    .get_temp::<bool>(id.with("had_focus"))
                    .unwrap_or(false)
            });

        ui.memory_mut(|m| {
            m.data.insert_temp(id, editing);
            m.data.insert_temp(id, text);
            m.data.insert_temp(id.with("had_focus"), editing);
        });

        if wants_focus {
            response.request_focus();
        }

        response
    }
}

pub struct TextInput<'a, 'b> {
    hint_text: &'a str,
    value: &'b mut Option<String>,
}

impl<'a, 'b> TextInput<'a, 'b> {
    pub fn new(hint_text: &'a str, value: &'b mut Option<String>) -> Self {
        Self { hint_text, value }
    }
}

impl<'a, 'b> egui::Widget for TextInput<'a, 'b> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let id = ui.next_auto_id();

        let mut text = ui.memory_mut(|m| m.data.get_temp::<String>(id).unwrap_or_else(String::new));

        let edit_response = ui.add(
            egui::TextEdit::singleline(&mut text)
                .desired_width(ui.available_width())
                .clip_text(true)
                .hint_text(self.hint_text),
        );

        let commit = ui.input(|i| i.key_pressed(egui::Key::Enter));
        let cancel = ui.input(|i| i.key_pressed(egui::Key::Escape));

        if commit && !text.trim().is_empty() {
            *self.value = Some(text.clone());
            text.clear();
        } else if cancel {
            text.clear();
            *self.value = None;
        }

        ui.memory_mut(|m| m.data.insert_temp(id, text));

        edit_response
    }
}


pub struct ImageCard<'a> {
    image: ImageSource<'a>,
    hover_image: Option<ImageSource<'a>>,
    size: Vec2,
    hover_scale: f32,
    corner_radius: u8,
    animation_time: f32,
    show_border: bool,
}

impl<'a> ImageCard<'a> {
    pub fn new(image: impl Into<ImageSource<'a>>, size: impl Into<Vec2>) -> Self {
        Self {
            image: image.into(),
            hover_image: None,
            size: size.into(),
            hover_scale: 1.05,
            corner_radius: 6,
            animation_time: 0.11,
            show_border: true,
        }
    }

    /// Image that fades in on top of the base image while hovered.
    pub fn hover_image(mut self, image: impl Into<ImageSource<'a>>) -> Self {
        self.hover_image = Some(image.into());
        self
    }

    /// Scale factor reached when fully hovered (default `1.05`).
    pub fn hover_scale(mut self, scale: f32) -> Self {
        self.hover_scale = scale;
        self
    }

    pub fn corner_radius(mut self, radius: u8) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Seconds for the hover animation to complete (default `0.11`).
    pub fn animation_time(mut self, seconds: f32) -> Self {
        self.animation_time = seconds;
        self
    }

    pub fn show_border(mut self, show: bool) -> Self {
        self.show_border = show;
        self
    }

}

impl Widget for ImageCard<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            image,
            hover_image,
            size,
            hover_scale,
            corner_radius,
            animation_time,
            show_border,
        } = self;

        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let hovered = response.hovered();

        // 0.0 -> 1.0 while hovered, back to 0.0 when not. egui requests
        // repaints for us while the animation is in flight.
        let t = ui.ctx().animate_bool_with_time(
            response.id.with("image_card_anim"),
            hovered,
            animation_time,
        );
        let scale = 1.0 + (hover_scale - 1.0) * t;

        if !ui.is_rect_visible(rect) {
            return response;
        }

        // Scale about the center of the card.
        let translation = rect.center().to_vec2() * (1.0 - scale);
        let visuals = *ui.style().interact(&response);

        ui.with_visual_transform(TSTransform::new(translation, scale), |ui| {
            ui.painter().rect(
                rect.expand(visuals.expansion * 2.),
                corner_radius,
                Color32::TRANSPARENT,
                Stroke::new(
                    if show_border { 1.0f32 } else { 0f32 },
                    if hovered {
                        Color32::from_white_alpha(127)
                    } else {
                        visuals.bg_fill
                    },
                ),
                StrokeKind::Outside,
            );

            Image::new(image)
                .corner_radius(corner_radius)
                .paint_at(ui, rect);

            if let Some(hover_image) = hover_image && t > 0.0 {
                Image::new(hover_image)
                    .corner_radius(corner_radius)
                    .tint(Color32::WHITE.gamma_multiply(t))
                    .paint_at(ui, rect);
            }
        });

        response
    }
}
