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

impl Color {
    /// Creates a new Color from HSL values.
    ///
    /// * `h` - Hue in degrees (0.0 - 360.0)
    /// * `s` - Saturation (0.0 - 1.0)
    /// * `l` - Lightness (0.0 - 1.0)
    pub const fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let r;
        let g;
        let b;

        if s == 0.0 {
            // Achromatic (Grey)
            r = l;
            g = l;
            b = l;
        } else {
            let q = if l < 0.5 {
                l * (1.0 + s)
            } else {
                l + s - l * s
            };
            let p = 2.0 * l - q;

            // Normalize Hue to 0.0 - 1.0
            let h_norm = h / 360.0;

            r = hue_to_rgb(p, q, h_norm + 1.0 / 3.0);
            g = hue_to_rgb(p, q, h_norm);
            b = hue_to_rgb(p, q, h_norm - 1.0 / 3.0);
        }

        Self {
            r: (r * 255.0).round() as u8,
            g: (g * 255.0).round() as u8,
            b: (b * 255.0).round() as u8,
            a: 255, // Default opaque
        }
    }

    /// Same as from_hsl, but with an alpha channel (0.0 - 1.0)
    pub const fn from_hsla(h: f32, s: f32, l: f32, a: f32) -> Self {
        let mut color = Self::from_hsl(h, s, l);
        color.a = (a * 255.0).round() as u8;
        color
    }
}

// Helper function for HSL conversion
const fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }

    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    return p;
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
