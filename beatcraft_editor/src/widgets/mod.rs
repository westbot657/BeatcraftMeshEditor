use egui::emath::TSTransform;
use egui::{Color32, Image, ImageSource, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget};

pub mod toggle;
pub mod keybind_hint;
pub mod bar_slider;
pub mod math_values;

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
