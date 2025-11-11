use heka::color;

use cosmic_text::{Align, Attrs, FamilyOwned, Metrics, Style as FontStyle, Weight};

#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font_family: FamilyOwned,
    pub color: color::Color,
    pub font_size: f32,
    pub line_height_px: f32,
    pub weight: Weight,
    pub style: FontStyle,
    pub align: Align,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: FamilyOwned::SansSerif,
            color: color::Color::black,
            font_size: 14.0,
            line_height_px: 18.0,
            weight: Weight::NORMAL,
            style: FontStyle::Normal,
            align: Align::Left,
        }
    }
}

pub trait AsCosmicColor {
    fn into_cosmic(&self) -> cosmic_text::Color;
}

impl AsCosmicColor for color::Color {
    fn into_cosmic(&self) -> cosmic_text::Color {
        cosmic_text::Color(self.as_u32())
    }
}

impl TextStyle {
    pub fn as_cosmic_attrs<'a>(&self) -> Attrs<'a> {
        Attrs {
            color_opt: Some(self.color.into_cosmic()),
            weight: self.weight,
            style: self.style,
            ..Attrs::new()
        }
    }

    pub fn as_cosmic_metrics(&self) -> Metrics {
        Metrics::new(self.font_size, self.line_height_px)
    }
}
