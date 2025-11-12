#![allow(non_upper_case_globals)]

/// RGBA defined color values
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[inline]
    pub const fn as_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

impl Color {
    /// Red color
    pub const red: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };

    /// White color
    pub const white: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    /// Black color
    pub const black: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };

    /// DodgerBlue - a nice color
    pub const dodger_blue: Color = Color {
        r: 30,
        g: 144,
        b: 255,
        a: 255,
    };

    /// RISDBlue - I like this one a lot
    pub const risd_blue: Color = Color::Hex(0x4455eeFF);

    /// Transform raw hex into RGBA componnent
    /// **FORMAT: RRGGBBAA**
    #[allow(non_snake_case)]
    pub const fn Hex(hex: u32) -> Color {
        let r = ((hex >> (8 * 3)) & 0xFF) as u8;
        let g = ((hex >> (8 * 2)) & 0xFF) as u8;
        let b = ((hex >> (8 * 1)) & 0xFF) as u8;
        let a = ((hex >> (8 * 0)) & 0xFF) as u8;

        Color { r, g, b, a }
    }

    /// Set an alpha value for the color
    pub fn with_alpha(mut self, value: u8) -> Self {
        self.a = value;
        self
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::Hex(0xFFFFFFFF)
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        [
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        ]
    }
}
