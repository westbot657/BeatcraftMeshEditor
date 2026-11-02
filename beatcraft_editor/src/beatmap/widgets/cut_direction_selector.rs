use egui::{Color32, ImageSource, Response, Ui, Widget};
use beatmap_core::CutDirectionExt as _;
use bs_mapping_data::CutDirection;
use crate::beatmap::widgets::angle_dial::AngleDial;
use crate::beatmap::widgets::toggle::ToggleSwitch;

pub static ARROW: ImageSource =
    egui::include_image!("../../assets/textures/svg/cut_dir_arrow.svg");
pub static DOT: ImageSource =
    egui::include_image!("../../assets/textures/svg/cut_dir_dot.svg");


trait CutDirectionExt {
    fn iter_grid() -> impl Iterator<Item = CutDirection>;
    fn tex(&self) -> (ImageSource<'_>, f32);
}

impl CutDirectionExt for CutDirection {
    fn iter_grid() -> impl Iterator<Item=CutDirection> {
        [
            Self::UpLeft,   Self::Up,   Self::UpRight,
            Self::Left,     Self::Dot,  Self::Right,
            Self::DownLeft, Self::Down, Self::DownRight,
        ].into_iter()
    }
    fn tex(&self) -> (ImageSource<'_>, f32) {
        match self {
            Self::Up => (ARROW.clone(), 180f32.to_radians()),
            Self::Down => (ARROW.clone(), 0f32.to_radians()),
            Self::Left => (ARROW.clone(), 90f32.to_radians()),
            Self::Right => (ARROW.clone(), -90f32.to_radians()),
            Self::UpLeft => (ARROW.clone(), 135f32.to_radians()),
            Self::UpRight => (ARROW.clone(), -135f32.to_radians()),
            Self::DownLeft => (ARROW.clone(), 45f32.to_radians()),
            Self::DownRight => (ARROW.clone(), -45f32.to_radians()),
            Self::Dot => (DOT.clone(), 0.),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SelectorLayout {
    Horizontal,
    Vertical,
}

pub struct CutDirectionSelector<'c> {
    cut_direction: &'c mut CutDirection,
    cut_offset: Option<&'c mut f32>,
    use_cut_offset: &'c mut bool,
    allow_dot: bool,

    layout: SelectorLayout,
    image: Option<ImageSource<'c>>,
}

impl<'c> CutDirectionSelector<'c> {
    pub fn new(
        cut_direction: &'c mut CutDirection,
        use_cut_offset: &'c mut bool,
        layout: SelectorLayout,
    ) -> Self {
        Self {
            cut_direction,
            cut_offset: None,
            use_cut_offset,
            allow_dot: true,
            layout,
            image: None,
        }
    }
    
    pub fn cut_offset(mut self, offset: &'c mut f32) -> Self {
        self.cut_offset = Some(offset);
        self
    }
    
    pub fn allow_dot(mut self, dot: bool) -> Self {
        self.allow_dot = dot;
        self
    }

    pub fn image(mut self, image: ImageSource<'c>) -> Self {
        self.image = Some(image);
        self
    }

    fn draw_button_panel(&mut self, ui: &mut Ui) -> Option<CutDirection> {
        let padding = ui.style().spacing.item_spacing.x;
        let size = ui.available_size();
        let width = size.x.min(size.y);
        let w3 = (width - padding * 2.) / 3.;

        let origin = ui.max_rect().min;

        let mut out = None;

        for (i, dir) in CutDirection::iter_grid().enumerate() {
            if !self.allow_dot && dir == CutDirection::Dot {
                continue;
            }
            let row = i / 3;
            let col = i % 3;

            let pos = origin
                + egui::vec2(
                col as f32 * (w3 + padding),
                row as f32 * (w3 + padding),
            );

            let rect = egui::Rect::from_min_size(pos, egui::Vec2::splat(w3));

            let response = ui.allocate_rect(rect, egui::Sense::click());

            let bg = if *self.cut_direction == dir {
                egui::Color32::from_gray(60)
            } else if response.hovered() {
                egui::Color32::from_gray(50)
            } else {
                egui::Color32::from_gray(0x20)
            };
            ui.painter().rect_filled(rect, 4.0, bg);

            let (img, angle) = dir.tex();

            egui::Image::new(img)
                .rotate(angle, egui::vec2(0.5, 0.5))
                .paint_at(ui, rect);

            if response.clicked() {
                out = Some(dir);
            }

        }

        out

    }

    fn draw_rotation_panel(&mut self, ui: &mut Ui) {
        if let Some(cut_offset) = &mut self.cut_offset {
            let padding = ui.style().spacing.item_spacing.x;
            let size = ui.available_size();
            let width = size.x.min(size.y);

            ui.allocate_ui_with_layout(
                [width, width].into(),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.allocate_ui_with_layout(
                        [width, 20.].into(),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.add_sized(
                                [20., 20.],
                                ToggleSwitch::new(self.use_cut_offset),
                            );

                            ui.add_enabled_ui(
                                *self.use_cut_offset,
                                |ui| {
                                    ui.add_sized(
                                        [width - padding - 20., 20.],
                                        egui::DragValue::new(*cut_offset)
                                            .range(-45f32..=45.)
                                            .speed(0.5).suffix("\u{00B0}"),
                                    )
                                },
                            );
                        },
                    );

                    ui.add_enabled_ui(
                        *self.use_cut_offset,
                        |ui| {
                            let mut dial = AngleDial::new(cut_offset, &[
                                (-22.5, 10., Color32::LIGHT_GRAY),
                                (-20., 5., Color32::GRAY),
                                (-15., 5., Color32::GRAY),
                                (-10., 5., Color32::GRAY),
                                (-5., 5., Color32::GRAY),
                                (0., 10., Color32::LIGHT_GRAY),
                                (5., 5., Color32::GRAY),
                                (10., 5., Color32::GRAY),
                                (15., 5., Color32::GRAY),
                                (20., 5., Color32::GRAY),
                                (22.5, 10., Color32::LIGHT_GRAY),
                                (25., 5., Color32::GRAY),
                                (-25., 5., Color32::GRAY),
                                (30., 5., Color32::GRAY),
                                (-30., 5., Color32::GRAY),
                                (35., 5., Color32::GRAY),
                                (-35., 5., Color32::GRAY),
                                (40., 5., Color32::GRAY),
                                (-40., 5., Color32::GRAY),
                                (45., 10., Color32::LIGHT_GRAY),
                                (-45., 10., Color32::LIGHT_GRAY),
                                (90., 10., Color32::GRAY),
                                (-90., 10., Color32::GRAY),
                                (135., 10., Color32::GRAY),
                                (-135., 10., Color32::GRAY),
                                (180., 10., Color32::GRAY),
                                (45. + 22.5, 5., Color32::DARK_GRAY),
                                (90. + 22.5, 5., Color32::DARK_GRAY),
                                (-90. + 22.5, 5., Color32::DARK_GRAY),
                                (135. + 22.5, 5., Color32::DARK_GRAY),
                                (-135. + 22.5, 5., Color32::DARK_GRAY),
                                (180. + 22.5, 5., Color32::DARK_GRAY),
                            ])
                                .zero_angle(-90.)
                                .line_length(0.5)
                                .line(true)
                                .arc(-45., 45.);

                            if let Some(img) = self.image.clone() {
                                dial = dial.image(img);
                                dial = dial.image_angle(self.cut_direction.angle_degrees());
                            };

                            ui.add_sized(
                                [width, width - padding - 20.],
                                dial,
                            )
                        },
                    );
                },
            );
        }

    }

}


impl<'c> Widget for CutDirectionSelector<'c> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let mut set = None;
        let res = match self.layout {
            SelectorLayout::Horizontal => {
                let size = ui.available_size();
                let width = size.x.min(size.y);
                let height = if self.cut_offset.is_some() {
                    width / 2.
                } else { width };
                ui.allocate_ui_with_layout(
                    [width, height].into(),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        set = self.draw_button_panel(ui);
                        if self.cut_offset.is_some() {
                            self.draw_rotation_panel(ui);
                        }
                    }
                ).response
            },
            SelectorLayout::Vertical => {
                let size = ui.available_size();
                let width = size.x.min(size.y);
                let height = if self.cut_offset.is_some() {
                    width / 2.
                } else { width };
                ui.allocate_ui_with_layout(
                    [height, width].into(),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        set = self.draw_button_panel(ui);
                        if self.cut_offset.is_some() {
                            self.draw_rotation_panel(ui);
                        }
                    }
                ).response
            },
        };
        if let Some(set) = set {
            *self.cut_direction = set;
        };
        res
    }
}




