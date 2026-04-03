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

            let mut drag_response = ui.add(drag);

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

pub struct MathDragValueOpt<'a, T: MapIndexable> {
    value: &'a mut Option<f32>,
    vars: &'a mut T,
    speed: f64,
    max_decimals: usize,
    suffix: Option<&'a str>,
    degrees: bool,
}

impl<'a, T: MapIndexable> MathDragValueOpt<'a, T> {
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

impl<'a, T: MapIndexable> egui::Widget for MathDragValueOpt<'a, T> {
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
