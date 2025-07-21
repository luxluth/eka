#![doc = include_str!("../README.md")]

pub mod macros;

#[derive(Debug, Clone, Copy)]
struct Space {
    pub id: usize,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Space {
    pub fn zero(id: usize) -> Self {
        Self {
            id,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }
}

#[derive(Debug, Clone, Copy)]
struct Capsule {
    space_ref: usize,
    parent_ref: usize,
    style_ref: usize,
    is_dirty: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    capsule_ref: usize,
}

impl<'a> Frame {
    pub fn style_mut(&'a self, root: &'a mut Root) -> &'a mut Style {
        root.styles
            .get_mut(root.capsules[self.capsule_ref].style_ref)
            .unwrap()
    }

    pub fn set_dirty(&self, root: &mut Root) {
        root.capsules[self.capsule_ref].is_dirty = true;
    }

    pub fn get_ref(&self) -> usize {
        self.capsule_ref
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const DODGER_BLUE: Color = Color {
        r: 30,
        g: 144,
        b: 255,
        a: 255,
    };

    /// FORMAT: RRGGBBAA
    #[allow(non_snake_case)]
    pub const fn Hex(hex: u32) -> Color {
        let r = ((hex >> (8 * 3)) & 0xFF) as u8;
        let g = ((hex >> (8 * 2)) & 0xFF) as u8;
        let b = ((hex >> (8 * 1)) & 0xFF) as u8;
        let a = ((hex >> (8 * 0)) & 0xFF) as u8;

        Color { r, g, b, a }
    }

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

#[derive(Debug, Clone, Copy)]
pub enum SizeSpec {
    Fill,
    Fit,
    Pixel(u32),
    Percent(f32),
}

impl SizeSpec {
    pub(crate) fn resolve_size(&self, parent_value: u32) -> u32 {
        match self {
            SizeSpec::Pixel(px) => *px,
            SizeSpec::Percent(pct) => (*pct * parent_value as f32) as u32,
            SizeSpec::Fill => parent_value, // basic fallback
            SizeSpec::Fit => 0,             // TODO: content-based later
        }
    }
}

impl Default for SizeSpec {
    fn default() -> Self {
        return Self::Pixel(0);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Style {
    pub background_color: Color,
    pub width: SizeSpec,
    pub height: SizeSpec,

    pub padding: u32,
}

#[derive(Debug)]
pub struct Root {
    pub(crate) spaces: Vec<Space>,
    pub(crate) styles: Vec<Style>,
    pub(crate) capsules: Vec<Capsule>,
}

impl Root {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            spaces: vec![Space::zero(0).with_width(width).with_height(height)],
            styles: vec![],
            capsules: vec![],
        }
    }

    pub fn dbg(&self, caps_ref: usize, name: Option<&str>) {
        if let Some(name) = name {
            eprintln!("NAME  :: {name}");
        }

        let Some(capsule) = self.capsules.get(caps_ref) else {
            return;
        };

        let style = self.styles[capsule.style_ref];
        let space = self.spaces[capsule.space_ref];

        eprintln!("STYLE :: {style:#?}");
        eprintln!("SPACE :: {space:#?}");
    }

    pub fn compute(&mut self) {
        for capsule in self.capsules.iter_mut() {
            if capsule.is_dirty {
                let style = self.styles[capsule.style_ref];
                let parent = self.spaces[capsule.parent_ref];
                let space = &mut self.spaces[capsule.space_ref];

                space.width = style.width.resolve_size(parent.width);
                space.height = style.height.resolve_size(parent.height);

                space.x = parent.x + style.padding as i32;
                space.y = parent.y + style.padding as i32;

                capsule.is_dirty = false;
            }
        }
    }

    fn children_of(&self, parent_id: usize) -> impl Iterator<Item = usize> + '_ {
        self.capsules
            .iter()
            .enumerate()
            .filter_map(move |(i, cap)| {
                if cap.parent_ref == parent_id {
                    Some(i)
                } else {
                    None
                }
            })
    }

    pub fn add_frame(&mut self) -> Frame {
        let new_id = self.spaces.len();
        let space = Space::zero(new_id);
        self.spaces.push(space);

        let new_style_idx = self.styles.len();
        self.styles.push(Style::default());

        let caps_ref = self.capsules.len();
        let caps = Capsule {
            space_ref: new_id,
            parent_ref: self.spaces[0].id,
            style_ref: new_style_idx,
            is_dirty: false,
        };

        self.capsules.push(caps);

        Frame {
            capsule_ref: caps_ref,
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
