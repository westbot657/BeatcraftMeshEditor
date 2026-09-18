use std::fmt::Debug;

#[derive(Debug)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Debug)]
pub enum TextLayout {
    Left,
    Right,
    Centered,
    Justified,
}

#[derive(Debug)]
pub struct ObstacleFontData {
    pub font: ObstacleFont,
    pub layout: TextLayout,
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
    pub letter_spacing: f32,
    pub word_spacing: f32,
}

#[derive(Debug)]
pub enum ObstacleFont {
    Simple,
}
